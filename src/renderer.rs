use std::sync::Mutex;

use nalgebra_glm::{dot, length, normalize, Vec3};

use crate::camera::OrbitCamera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::material::Material;
use crate::postprocess::{tone_map, vignette};
use crate::ray_intersect::Intersect;
use crate::scene::Scene;

// Luz ambiental hemisferica: la luz indirecta depende de hacia donde mira la
// superficie. Las caras que miran arriba reciben el azul del cielo (sombras
// frias que contrastan con el sol calido); las que miran abajo, la luz rosada
// que rebota en el mar de nubes iluminado por el atardecer. Las caras
// laterales reciben una mezcla. Se multiplica por el color base.
const AMBIENT_SKY: Color = Color { r: 60.0, g: 70.0, b: 100.0 };
const AMBIENT_GROUND: Color = Color { r: 135.0, g: 98.0, b: 130.0 };

fn ambient_light(normal: &Vec3) -> Color {
    let up = (normal.y + 1.0) * 0.5; // 1 mirando arriba, 0 mirando abajo
    AMBIENT_GROUND * (1.0 - up) + AMBIENT_SKY * up
}

// Separacion del origen de los rayos secundarios (sombra, reflexion,
// refraccion) respecto a la superficie, para no volver a chocar con la
// misma cara por errores de redondeo.
const BIAS: f32 = 1e-3;

// Maximo de rebotes de rayos secundarios. Sin limite, dos superficies
// reflectantes enfrentadas harian una recursion infinita. Con 4 un rayo
// puede entrar a un cristal, salir y todavia ver algo con su reflejo.
const MAX_DEPTH: u32 = 4;

// Aporte minimo (fraccion del pixel final) para que valga la pena lanzar un
// rayo secundario. Por debajo de 6% el cambio casi no se nota en pantalla.
const MIN_CONTRIBUTION: f32 = 0.06;

// Refleja la direccion `incident` sobre la superficie con normal `normal`:
// R = I - 2(I.N)N (angulo de salida igual al de entrada).
fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - normal * (2.0 * dot(incident, normal))
}

// Refraccion con la ley de Snell (n1 sen(t1) = n2 sen(t2)).
// `normal` apunta hacia afuera del objeto e `ior` es su indice de refraccion.
// Devuelve None si hay reflexion interna total (el rayo no puede salir).
fn refract(incident: &Vec3, normal: &Vec3, ior: f32) -> Option<Vec3> {
    let mut cos_i = dot(incident, normal).clamp(-1.0, 1.0);
    let (n, eta) = if cos_i < 0.0 {
        // Entrando (aire -> material): el rayo va contra la normal.
        cos_i = -cos_i;
        (*normal, 1.0 / ior)
    } else {
        // Saliendo (material -> aire): se usa la normal invertida.
        (-normal, ior)
    };

    // k < 0: el angulo es demasiado inclinado para salir -> reflexion
    // interna total.
    let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
    if k < 0.0 {
        return None;
    }
    Some(normalize(&(incident * eta + n * (eta * cos_i - k.sqrt()))))
}

// Fresnel (aproximacion de Schlick): fraccion de la luz que se REFLEJA en
// la superficie de un material transparente segun el angulo. De frente se
// refleja poco (R0: 2% en agua, 4% en vidrio); casi a ras de la superficie
// se refleja casi todo. Por eso un lago visto de lado refleja el cielo.
fn schlick(incident: &Vec3, normal: &Vec3, ior: f32) -> f32 {
    let r0 = ((1.0 - ior) / (1.0 + ior)).powi(2);
    let cos_theta = dot(incident, normal).abs();
    r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
}

// Ondas en el agua (como un normal map, pero calculado): la caja sigue
// siendo plana, pero la normal se inclina un poco segun la posicion con una
// suma de senos en distintas direcciones. Asi el reflejo del cielo y la
// refraccion del fondo se ondulan como en agua real. Se quita la parte de la
// perturbacion que va en la direccion de la normal para que solo la incline.
fn ripple_normal(normal: &Vec3, point: &Vec3, amplitude: f32) -> Vec3 {
    let (x, y, z) = (point.x, point.y, point.z);
    let wobble = Vec3::new(
        (z * 7.0 + x * 2.0).sin() + 0.5 * (x * 13.0 - y * 5.0).sin(),
        (x * 6.0 + z * 4.0).sin() * 0.5,
        (x * 8.0 - z * 3.0).sin() + 0.5 * (z * 11.0 + y * 6.0).sin(),
    ) * amplitude;
    let tangent_wobble = wobble - normal * dot(&wobble, normal);
    normalize(&(normal + tangent_wobble))
}

// Bump mapping: la textura se usa tambien como mapa de alturas. Con
// diferencias finitas (altura un poco a la derecha menos un poco a la
// izquierda) se obtiene la pendiente en u y en v; la normal se inclina en
// contra de la pendiente, sobre la tangente y la bitangente de la cara. Asi
// las juntas de la piedra o los surcos de la corteza reciben la luz como si
// estuvieran hundidos, aunque la caja sea plana.
fn bump_normal(intersect: &Intersect, material: &Material) -> Vec3 {
    let texture = &material.texture;
    let (u, v) = (intersect.u, intersect.v);
    let e = texture.texel_size() * 0.5;
    // Pendiente = cambio de altura / distancia recorrida (2e), escalada por
    // la profundidad del relieve.
    let slope_u = (texture.height(u + e, v) - texture.height(u - e, v)) / (2.0 * e) * material.bump;
    let slope_v = (texture.height(u, v + e) - texture.height(u, v - e)) / (2.0 * e) * material.bump;
    normalize(&(intersect.normal - intersect.tangent * slope_u - intersect.bitangent * slope_v))
}

// Origen de un rayo secundario: el punto desplazado un poco hacia el lado
// de la superficie al que va el rayo (afuera si se refleja, adentro si se
// refracta hacia el interior).
fn offset_origin(point: &Vec3, normal: &Vec3, direction: &Vec3) -> Vec3 {
    if dot(direction, normal) < 0.0 {
        point - normal * BIAS
    } else {
        point + normal * BIAS
    }
}

// `weight` es cuanto aporta este rayo al color final del pixel: 1.0 para el
// rayo primario y, en cada rebote, se multiplica por la fraccion que se
// refleja o refracta (un reflejo dentro de otro reflejo pesa cada vez menos).
pub fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, scene: &Scene, depth: u32, weight: f32) -> Color {
    if depth > MAX_DEPTH {
        return scene.skybox.sample(ray_direction);
    }

    let intersect = scene.closest_hit(ray_origin, ray_direction);

    // El rayo no golpeo nada: se ve el cielo en esa direccion.
    if !intersect.is_intersecting {
        return scene.skybox.sample(ray_direction);
    }

    let material = &scene.materials[intersect.material_id];
    let base_color = material.base_color(intersect.u, intersect.v);

    // Superficie emisiva (fuego): brilla con su propio color. No depende de
    // las luces, asi que no se calcula difuso, sombras ni reflejos.
    if material.emission > 0.0 {
        return base_color * material.emission;
    }

    let normal = if material.ripple > 0.0 {
        ripple_normal(&intersect.normal, &intersect.point, material.ripple)
    } else if material.bump > 0.0 {
        bump_normal(&intersect, material)
    } else {
        intersect.normal
    };
    let view_dir = -ray_direction; // V: direccion hacia la camara

    // Color propio de la superficie (ambiente + difuso) y brillo especular
    // se acumulan por separado: el especular es luz reflejada, asi que no se
    // atenua con la reflectividad mas abajo.
    let mut surface = base_color * ambient_light(&normal);
    let mut specular = Color::new(0.0, 0.0, 0.0);

    // Cada luz suma su aporte difuso y especular.
    for light in &scene.lights {
        // L: direccion hacia la luz.
        let to_light = light.position - intersect.point;
        let light_distance = length(&to_light);
        let light_dir = to_light / light_distance;

        // Atenuacion: las luces cercanas (fuego) pierden fuerza con la
        // distancia. Fuera de su alcance no aportan nada y nos ahorramos el
        // rayo de sombra.
        let attenuation = light.attenuation(light_distance);
        if attenuation <= 0.0 {
            continue;
        }

        // Difuso (Lambert): mas luz cuanto mas de frente llega a la superficie.
        let diffuse_intensity = dot(&normal, &light_dir).max(0.0);
        if diffuse_intensity <= 0.0 {
            continue; // la luz llega por detras de esta cara
        }

        // Sombra: rayo desde el punto hacia la luz. Si choca con algo antes de
        // llegar a ella, a este punto llega solo la fraccion que ese algo deja
        // pasar (0 si es opaco). El origen se separa un poco de la superficie
        // (bias) para no chocar con la misma cara por errores de redondeo
        // ("shadow acne").
        let shadow_origin = intersect.point + normal * BIAS;
        let transmission = scene.shadow_transmission(&shadow_origin, &light_dir, light_distance);
        if transmission <= 0.0 {
            continue;
        }
        let light_intensity = light.intensity * attenuation * transmission;
        surface = surface + base_color * light.color * (diffuse_intensity * light_intensity);

        // Specular (Phong): R es el reflejo de la luz sobre la normal. Si R
        // apunta a la camara se ve un brillo; shininess controla que tan
        // concentrado es.
        let light_reflect = reflect(&-light_dir, &normal);
        let specular_intensity = dot(&light_reflect, &view_dir).max(0.0).powf(material.shininess);
        specular = specular + light.color * (material.specular * specular_intensity * light_intensity);
    }

    let transparency = material.transparency;

    // Fresnel en superficies opacas (Schlick): de frente se refleja solo
    // `reflectivity`; al mirar de lado el reflejo crece hacia el maximo que
    // permite el material (lo que no es transparente). `fresnel` dice cuanto
    // se acerca a ese maximo. Asi el oro de los capiteles o las tejas
    // brillan con el cielo en los bordes, como en la vida real.
    let reflectivity = if material.fresnel > 0.0 {
        let edge = (1.0 - dot(&view_dir, &normal).abs()).powi(5);
        let headroom = (1.0 - material.reflectivity - transparency).max(0.0);
        material.reflectivity + headroom * edge * material.fresnel
    } else {
        material.reflectivity
    };
    let mut surface_weight = (1.0 - reflectivity - transparency).max(0.0);
    let mut reflect_weight = reflectivity;
    let mut refract_weight = 0.0;

    // Refraccion: rayo secundario que atraviesa el material doblandose segun
    // la ley de Snell. Lo que se ve a traves se filtra por el albedo (el color
    // del medio, como un vidrio tintado): el agua tine de azul el fondo. La
    // textura se sigue viendo en la parte de color propio de la superficie.
    let mut refract_dir = None;
    if transparency > 0.0 {
        match refract(ray_direction, &normal, material.refractive_index) {
            Some(dir) => {
                // Fresnel: de la parte transparente, una fraccion se refleja
                // (mas cuanto mas de lado se mira) y el resto se refracta.
                let fresnel = schlick(ray_direction, &normal, material.refractive_index);
                reflect_weight += transparency * fresnel;
                refract_weight = transparency * (1.0 - fresnel);
                refract_dir = Some(dir);
            }
            // Reflexion interna total: la parte transparente se refleja.
            None => reflect_weight += transparency,
        }
    }

    // Corte por importancia: un rebote que aportaria muy poco al pixel final
    // no se lanza (un rayo secundario cuesta tanto como uno primario, con sus
    // sombras y rebotes). Su peso pasa al color propio para que la superficie
    // no se oscurezca. Asi la piedra (2% de reflejo) no paga un rayo extra
    // por pixel, pero el metal y el agua siguen reflejando.
    if weight * reflect_weight < MIN_CONTRIBUTION {
        surface_weight += reflect_weight;
        reflect_weight = 0.0;
    }
    if weight * refract_weight < MIN_CONTRIBUTION {
        surface_weight += refract_weight;
        refract_weight = 0.0;
    }

    let mut refracted = Color::new(0.0, 0.0, 0.0);
    if let Some(dir) = refract_dir.filter(|_| refract_weight > 0.0) {
        let refract_origin = offset_origin(&intersect.point, &normal, &dir);
        refracted = cast_ray(&refract_origin, &dir, scene, depth + 1, weight * refract_weight) * material.albedo;
    }

    // Reflexion: rayo secundario en la direccion reflejada; su color (lo que
    // "ve" el reflejo) se mezcla con el color propio segun la reflectividad.
    // En un metal el reflejo se filtra por el color de la superficie (el oro
    // refleja en tonos dorados); en los demas materiales se ve el color real
    // de lo reflejado, como el cielo en una teja esmaltada o en la obsidiana.
    let mut reflected = Color::new(0.0, 0.0, 0.0);
    if reflect_weight > 0.0 {
        let reflect_dir = normalize(&reflect(ray_direction, &normal));
        let reflect_origin = offset_origin(&intersect.point, &normal, &reflect_dir);
        reflected = cast_ray(&reflect_origin, &reflect_dir, scene, depth + 1, weight * reflect_weight);
        if material.metallic {
            reflected = reflected * base_color;
        }
    }

    // Reparto de la luz: color propio + reflejo + refraccion + brillo.
    let color = surface * surface_weight + reflected * reflect_weight + refracted * refract_weight + specular;
    apply_fog(color, intersect.distance, ray_direction, scene)
}

// Perspectiva atmosferica: el aire no es perfectamente transparente, asi
// que lo lejano se funde con el color del cielo. Da sensacion de
// profundidad (las islas del fondo se ven mas claras y "lejanas"). Hasta
// FOG_START no hay bruma; despues crece como 1 - e^(-densidad * distancia)
// (la fraccion de luz que el aire absorbe/dispersa en ese tramo).
const FOG_START: f32 = 10.0;
const FOG_DENSITY: f32 = 0.018;

fn apply_fog(color: Color, distance: f32, ray_direction: &Vec3, scene: &Scene) -> Color {
    let fog = 1.0 - (-(distance - FOG_START).max(0.0) * FOG_DENSITY).exp();
    if fog <= 0.0 {
        return color;
    }
    color * (1.0 - fog) + scene.skybox.haze(ray_direction) * fog
}

// Posiciones de las muestras dentro de un pixel (0..1 en x e y) para el
// antialiasing: grilla rotada de 4 muestras. Comparada con una grilla 2x2
// alineada, cada muestra tiene una x y una y distintas, asi los bordes casi
// horizontales o casi verticales (muy comunes en cajas) se suavizan mejor.
const AA_SAMPLES: [(f32, f32); 4] = [(0.375, 0.125), (0.875, 0.375), (0.125, 0.625), (0.625, 0.875)];
const CENTER_SAMPLE: [(f32, f32); 1] = [(0.5, 0.5)];

// Renderiza la escena. Con `antialias`, cada pixel promedia 4 rayos (4 veces
// mas lento): se usa cuando la camara esta quieta y en las capturas.
pub fn render(framebuffer: &mut Framebuffer, scene: &Scene, camera: &OrbitCamera, antialias: bool) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;

    // La base de la camara es igual para todos los pixeles: se calcula una vez.
    let basis = camera.basis();

    // Render en paralelo con hilos de la libreria estandar (sin crates).
    // Cada pixel es independiente de los demas, asi que repartimos las filas
    // entre los nucleos del CPU. El reparto es dinamico: las filas estan en
    // una "cola" compartida y cada hilo, al terminar una, toma la siguiente
    // libre. Asi los nucleos rapidos (o los que tocan filas de cielo, que son
    // baratas) procesan mas filas y ninguno se queda esperando al final. En
    // un CPU hibrido (nucleos de rendimiento + de eficiencia) un reparto fijo
    // obligaria a todos a esperar a los nucleos lentos.
    let thread_count = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let row_width = framebuffer.width as usize;

    // El Mutex protege el iterador de filas: solo un hilo a la vez saca la
    // siguiente. Se bloquea apenas un instante por fila (600 veces por cuadro).
    let rows = Mutex::new(framebuffer.pixels_mut().chunks_mut(row_width).enumerate());

    let samples: &[(f32, f32)] = if antialias { &AA_SAMPLES } else { &CENTER_SAMPLE };

    // thread::scope garantiza que todos los hilos terminan antes de salir,
    // por eso pueden usar referencias prestadas (scene, basis, rows).
    std::thread::scope(|scope| {
        for _ in 0..thread_count {
            let basis = &basis;
            let rows = &rows;
            scope.spawn(move || {
                loop {
                    // El candado se suelta al terminar esta linea.
                    let next = rows.lock().unwrap().next();
                    let Some((y, row)) = next else { break };

                    for (x, pixel) in row.iter_mut().enumerate() {
                        // Promedio de las muestras del pixel, cada una ya con
                        // tone mapping (si se promedia antes, un borde con
                        // fuego muy brillante dejaria un halo).
                        let mut sum = Color::new(0.0, 0.0, 0.0);
                        for (dx, dy) in samples {
                            // Pixel -> espacio de pantalla [-1, 1] (y invertida porque en
                            // pantalla crece hacia abajo), con aspect ratio en x.
                            let screen_x = ((2.0 * (x as f32 + dx)) / width - 1.0) * aspect_ratio;
                            let screen_y = -(2.0 * (y as f32 + dy)) / height + 1.0;

                            let ray_direction = basis.ray_direction(screen_x, screen_y);
                            sum = sum + tone_map(cast_ray(&basis.eye, &ray_direction, scene, 0, 1.0));
                        }
                        let pixel_color = sum * (1.0 / samples.len() as f32);

                        let ndc_x = (2.0 * (x as f32 + 0.5)) / width - 1.0;
                        let ndc_y = -(2.0 * (y as f32 + 0.5)) / height + 1.0;
                        *pixel = vignette(pixel_color, ndc_x, ndc_y).to_raylib();
                    }
                }
            });
        }
    });
}
