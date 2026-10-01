use nalgebra_glm::{dot, length, normalize, Vec3};
use raylib::color::Color as RColor;

use crate::camera::OrbitCamera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::scene::Scene;

// Luz ambiental: la luz indirecta que llega desde todo el cielo. Tenida de
// azul (como el cielo) para que las sombras se vean frias y contrasten con
// el sol calido. Se multiplica por el color base (Color * Color / 255).
const AMBIENT_LIGHT: Color = Color { r: 60.0, g: 70.0, b: 100.0 };

// Separacion del origen de los rayos secundarios (sombra, reflexion,
// refraccion) respecto a la superficie, para no volver a chocar con la
// misma cara por errores de redondeo.
const BIAS: f32 = 1e-3;

// Maximo de rebotes de rayos secundarios. Sin limite, dos superficies
// reflectantes enfrentadas harian una recursion infinita. Con 4 un rayo
// puede entrar a un cristal, salir y todavia ver algo con su reflejo.
const MAX_DEPTH: u32 = 4;

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

pub fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, scene: &Scene, depth: u32) -> Color {
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
    let normal = intersect.normal;
    let view_dir = -ray_direction; // V: direccion hacia la camara

    // Color propio de la superficie (ambiente + difuso) y brillo especular
    // se acumulan por separado: el especular es luz reflejada, asi que no se
    // atenua con la reflectividad mas abajo.
    let mut surface = base_color * AMBIENT_LIGHT;
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

    let reflectivity = material.reflectivity;
    let transparency = material.transparency;
    let mut reflect_weight = reflectivity;
    let mut refract_weight = 0.0;

    // Refraccion: rayo secundario que atraviesa el material doblandose segun
    // la ley de Snell. Lo que se ve a traves se filtra por el albedo (el color
    // del medio, como un vidrio tintado): el agua tine de azul el fondo. La
    // textura se sigue viendo en la parte de color propio de la superficie.
    let mut refracted = Color::new(0.0, 0.0, 0.0);
    if transparency > 0.0 {
        match refract(ray_direction, &normal, material.refractive_index) {
            Some(refract_dir) => {
                let refract_origin = offset_origin(&intersect.point, &normal, &refract_dir);
                refracted = cast_ray(&refract_origin, &refract_dir, scene, depth + 1) * material.albedo;
                // Fresnel: de la parte transparente, una fraccion se refleja
                // (mas cuanto mas de lado se mira) y el resto se refracta.
                let fresnel = schlick(ray_direction, &normal, material.refractive_index);
                reflect_weight += transparency * fresnel;
                refract_weight = transparency * (1.0 - fresnel);
            }
            // Reflexion interna total: la parte transparente se refleja.
            None => reflect_weight += transparency,
        }
    }

    // Reflexion: rayo secundario en la direccion reflejada; su color (lo que
    // "ve" el reflejo) se mezcla con el color propio segun la reflectividad.
    // El reflejo se filtra por el color de la superficie, como en un metal
    // real: el oro refleja en tonos dorados. En materiales casi blancos
    // (cristal) practicamente no cambia.
    let mut reflected = Color::new(0.0, 0.0, 0.0);
    if reflect_weight > 0.0 {
        let reflect_dir = normalize(&reflect(ray_direction, &normal));
        let reflect_origin = offset_origin(&intersect.point, &normal, &reflect_dir);
        reflected = cast_ray(&reflect_origin, &reflect_dir, scene, depth + 1) * base_color;
    }

    // Reparto de la luz: color propio + reflejo + refraccion + brillo.
    let surface_weight = (1.0 - reflectivity - transparency).max(0.0);
    surface * surface_weight + reflected * reflect_weight + refracted * refract_weight + specular
}

pub fn render(framebuffer: &mut Framebuffer, scene: &Scene, camera: &OrbitCamera) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;

    // La base de la camara es igual para todos los pixeles: se calcula una vez.
    let basis = camera.basis();

    // Render en paralelo con hilos de la libreria estandar (sin crates).
    // Cada pixel es independiente de los demas, asi que repartimos las filas
    // entre los nucleos del CPU. El reparto es intercalado (hilo i toma las
    // filas i, i+N, i+2N...) para que todos reciban una mezcla parecida de
    // cielo (barato) y escena (caro) y terminen al mismo tiempo.
    let thread_count = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let row_width = framebuffer.width as usize;

    let mut rows_per_thread: Vec<Vec<(usize, &mut [RColor])>> =
        (0..thread_count).map(|_| Vec::new()).collect();
    for (y, row) in framebuffer.pixels_mut().chunks_mut(row_width).enumerate() {
        rows_per_thread[y % thread_count].push((y, row));
    }

    // thread::scope garantiza que todos los hilos terminan antes de salir,
    // por eso pueden usar referencias prestadas (scene, basis).
    std::thread::scope(|scope| {
        for rows in rows_per_thread {
            let basis = &basis;
            scope.spawn(move || {
                for (y, row) in rows {
                    for (x, pixel) in row.iter_mut().enumerate() {
                        // Pixel -> espacio de pantalla [-1, 1] (y invertida porque en
                        // pantalla crece hacia abajo), con aspect ratio en x.
                        let screen_x = ((2.0 * x as f32) / width - 1.0) * aspect_ratio;
                        let screen_y = -(2.0 * y as f32) / height + 1.0;

                        let ray_direction = basis.ray_direction(screen_x, screen_y);
                        let pixel_color = cast_ray(&basis.eye, &ray_direction, scene, 0);

                        *pixel = pixel_color.to_raylib();
                    }
                }
            });
        }
    });
}
