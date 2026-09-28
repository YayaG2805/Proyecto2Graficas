mod camera;
mod color;
mod cube;
mod framebuffer;
mod light;
mod ray_intersect;
mod scene;
mod texture;

use nalgebra_glm::{dot, normalize, Vec3};
use raylib::color::Color as RColor;
use raylib::prelude::*;

use camera::OrbitCamera;
use color::Color;
use cube::Cube;
use framebuffer::Framebuffer;
use light::Light;
use ray_intersect::{Intersect, RayIntersect};

const SKY_COLOR: Color = Color { r: 4.0, g: 12.0, b: 36.0 };

// Luz ambiental minima para que las caras sin luz directa no queden negras
// y se lea la forma del diorama. Se reemplaza por iluminacion completa en la Fase 6.
const AMBIENT: f32 = 0.25;

// Velocidades de la camara (por segundo, se multiplican por dt).
const ORBIT_SPEED: f32 = 1.5; // radianes/s
const ZOOM_SPEED: f32 = 8.0; // unidades/s
const WHEEL_ZOOM_STEP: f32 = 1.0; // unidades por "clic" de rueda

fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, objects: &[Cube], light: &Light) -> Color {
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let tmp = object.ray_intersect(ray_origin, ray_direction);
        if tmp.is_intersecting && tmp.distance < zbuffer {
            zbuffer = tmp.distance;
            intersect = tmp;
        }
    }

    if !intersect.is_intersecting {
        return SKY_COLOR;
    }

    let light_dir = normalize(&(light.position - intersect.point));
    let diffuse_intensity = dot(&intersect.normal, &light_dir).max(0.0);
    let diffuse_color = intersect.material.diffuse_at(intersect.u, intersect.v);

    diffuse_color * light.color * (diffuse_intensity * light.intensity) + diffuse_color * AMBIENT
}

fn render(framebuffer: &mut Framebuffer, objects: &[Cube], light: &Light, camera: &OrbitCamera) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;

    // La base de la camara es igual para todos los pixeles: se calcula una vez.
    let basis = camera.basis();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            // Pixel -> espacio de pantalla [-1, 1] (y invertida porque en
            // pantalla crece hacia abajo), con aspect ratio en x.
            let screen_x = ((2.0 * x as f32) / width - 1.0) * aspect_ratio;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let ray_direction = basis.ray_direction(screen_x, screen_y);
            let pixel_color = cast_ray(&basis.eye, &ray_direction, objects, light);

            framebuffer.set_pixel(x, y, pixel_color);
        }
    }
}

// Lee el teclado/rueda y mueve la camara. Devuelve true si la camara cambio,
// para solo volver a renderizar cuando hace falta.
fn handle_camera_input(rl: &RaylibHandle, camera: &mut OrbitCamera, dt: f32) -> bool {
    let mut delta_yaw = 0.0;
    let mut delta_pitch = 0.0;
    let mut delta_zoom = 0.0;

    if rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT) {
        delta_yaw -= ORBIT_SPEED * dt;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        delta_yaw += ORBIT_SPEED * dt;
    }
    if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
        delta_pitch += ORBIT_SPEED * dt;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN) {
        delta_pitch -= ORBIT_SPEED * dt;
    }
    if rl.is_key_down(KeyboardKey::KEY_Q) {
        delta_zoom -= ZOOM_SPEED * dt;
    }
    if rl.is_key_down(KeyboardKey::KEY_E) {
        delta_zoom += ZOOM_SPEED * dt;
    }
    // Rueda hacia adelante = acercarse.
    delta_zoom -= rl.get_mouse_wheel_move() * WHEEL_ZOOM_STEP;

    if delta_yaw == 0.0 && delta_pitch == 0.0 && delta_zoom == 0.0 {
        return false;
    }

    camera.orbit(delta_yaw, delta_pitch);
    camera.zoom(delta_zoom);
    true
}

fn main() {
    let window_width = 800;
    let window_height = 600;

    let (mut rl, thread) = raylib::init()
        .size(window_width, window_height)
        .title("Proyecto 2 - Raytracing: Santuario flotante")
        .build();
    rl.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(window_width, window_height);

    let objects = scene::build_scene();
    println!("Escena: {} cajas", objects.len());

    let light = Light::new(Vec3::new(-8.0, 12.0, 6.0), 1.0, Color::new(255.0, 255.0, 255.0));

    let mut camera = OrbitCamera::new(
        Vec3::new(2.5, 0.5, -0.5), // target: centro del diorama (isla + satelite)
        0.7,                       // yaw
        0.35,                      // pitch
        20.0,                      // distance
        std::f32::consts::PI / 3.0,
    );

    // Textura de pantalla donde subimos el framebuffer cada vez que cambia.
    let image = Image::gen_image_color(window_width, window_height, RColor::BLACK);
    let mut screen_texture = rl
        .load_texture_from_image(&thread, &image)
        .expect("no se pudo crear la textura del framebuffer");

    let mut needs_render = true;
    let mut render_ms = 0.0;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        if handle_camera_input(&rl, &mut camera, dt) {
            needs_render = true;
        }

        if needs_render {
            let start = std::time::Instant::now();
            render(&mut framebuffer, &objects, &light, &camera);
            render_ms = start.elapsed().as_secs_f32() * 1000.0;
            screen_texture
                .update_texture(&framebuffer.to_rgba_bytes())
                .expect("no se pudo actualizar la textura del framebuffer");
            needs_render = false;
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(RColor::BLACK);
        d.draw_texture(&screen_texture, 0, 0, RColor::WHITE);
        d.draw_fps(10, 10);
        d.draw_text(&format!("render: {:.0} ms", render_ms), 10, 32, 20, RColor::WHITE);
    }
}
