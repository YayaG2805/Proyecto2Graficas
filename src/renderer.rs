use nalgebra_glm::{dot, length, normalize, Vec3};
use raylib::color::Color as RColor;

use crate::camera::OrbitCamera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::scene::Scene;

const SKY_COLOR: Color = Color { r: 4.0, g: 12.0, b: 36.0 };

// Luz ambiental: la luz indirecta que llega desde todo el cielo. Tenida de
// azul (como el cielo) para que las sombras se vean frias y contrasten con
// el sol calido. Se multiplica por el color base (Color * Color / 255).
const AMBIENT_LIGHT: Color = Color { r: 60.0, g: 70.0, b: 100.0 };

// Separacion del origen de los rayos secundarios respecto a la superficie.
const SHADOW_BIAS: f32 = 1e-3;

pub fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, scene: &Scene) -> Color {
    let intersect = scene.closest_hit(ray_origin, ray_direction);

    if !intersect.is_intersecting {
        return SKY_COLOR;
    }

    let material = &scene.materials[intersect.material_id];
    let base_color = material.base_color(intersect.u, intersect.v);
    let normal = intersect.normal;
    let view_dir = -ray_direction; // V: direccion hacia la camara

    let mut color = base_color * AMBIENT_LIGHT;

    // Cada luz suma su aporte difuso y especular.
    for light in &scene.lights {
        // L: direccion hacia la luz.
        let light_dir = normalize(&(light.position - intersect.point));

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
        let shadow_origin = intersect.point + normal * SHADOW_BIAS;
        let light_distance = length(&(light.position - intersect.point));
        let transmission = scene.shadow_transmission(&shadow_origin, &light_dir, light_distance);
        if transmission <= 0.0 {
            continue;
        }
        let light_intensity = light.intensity * transmission;
        color = color + base_color * light.color * (diffuse_intensity * light_intensity);

        // Specular (Phong): R es el reflejo de L sobre la normal. Si R apunta
        // a la camara se ve un brillo; shininess controla que tan concentrado es.
        let reflect_dir = normal * (2.0 * dot(&normal, &light_dir)) - light_dir;
        let specular_intensity = dot(&reflect_dir, &view_dir).max(0.0).powf(material.shininess);
        color = color + light.color * (material.specular * specular_intensity * light_intensity);
    }

    color
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
                        let pixel_color = cast_ray(&basis.eye, &ray_direction, scene);

                        *pixel = pixel_color.to_raylib();
                    }
                }
            });
        }
    });
}
