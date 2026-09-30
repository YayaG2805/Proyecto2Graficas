mod camera;
mod color;
mod cube;
mod framebuffer;
mod group;
mod light;
mod material;
mod procedural;
mod ray_intersect;
mod renderer;
mod scene;
mod skybox;
mod texture;

use nalgebra_glm::Vec3;
use raylib::color::Color as RColor;
use raylib::prelude::*;

use camera::OrbitCamera;
use framebuffer::Framebuffer;
use renderer::render;
use scene::Scene;

// Velocidades de la camara (por segundo, se multiplican por dt).
const ORBIT_SPEED: f32 = 1.5; // radianes/s
const ZOOM_SPEED: f32 = 8.0; // unidades/s
const WHEEL_ZOOM_STEP: f32 = 1.0; // unidades por "clic" de rueda

// Factor de reduccion de la vista previa mientras la camara se mueve.
const PREVIEW_SCALE: i32 = 2;

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

// Imprime los parametros de cada material al arrancar (evidencia de que cada
// uno tiene su propia textura, albedo, specular, transparencia, etc.).
fn print_material_table(scene: &Scene) {
    println!(
        "{:<8} {:>15} {:>8} {:>9} {:>8} {:>8} {:>5}",
        "Material", "albedo (RGB)", "specular", "shininess", "reflect", "transp", "n"
    );
    for m in &scene.materials {
        println!(
            "{:<8} {:>5.0},{:>4.0},{:>4.0} {:>8.2} {:>9.0} {:>8.2} {:>8.2} {:>5.2}",
            m.name, m.albedo.r, m.albedo.g, m.albedo.b, m.specular, m.shininess, m.reflectivity,
            m.transparency, m.refractive_index
        );
    }
}

// Guarda el contenido del framebuffer como PNG (usando Image de raylib, que
// ya usabamos en clase para exportar la textura del tablero).
fn save_screenshot(framebuffer: &Framebuffer, path: &str) {
    let mut image = Image::gen_image_color(framebuffer.width, framebuffer.height, RColor::BLACK);
    for (i, color) in framebuffer.pixels().iter().enumerate() {
        let x = i as i32 % framebuffer.width;
        let y = i as i32 / framebuffer.width;
        image.draw_pixel(x, y, *color);
    }
    image.export_image(path);
    println!("Captura guardada en {}", path);
}

fn main() {
    let window_width = 800;
    let window_height = 600;

    let scene = scene::build_scene();
    println!("Escena: {} cajas en {} grupos", scene.cube_count(), scene.groups.len());
    print_material_table(&scene);

    let mut camera = OrbitCamera::new(
        Vec3::new(2.5, 0.5, -0.5), // target: centro del diorama (isla + satelite)
        0.7,                       // yaw
        0.35,                      // pitch
        20.0,                      // distance
        std::f32::consts::PI / 3.0,
    );

    let mut framebuffer = Framebuffer::new(window_width, window_height);

    // Modo captura sin ventana:
    //   cargo run -- --screenshot archivo.png [yaw pitch distance [tx ty tz]]
    // Renderiza un solo cuadro, lo guarda y termina.
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--screenshot" {
        let parse = |s: &String| s.parse::<f32>().expect("los parametros de camara deben ser numeros");
        let target = if args.len() >= 9 {
            Vec3::new(parse(&args[6]), parse(&args[7]), parse(&args[8]))
        } else {
            camera.target
        };
        if args.len() >= 6 {
            camera = OrbitCamera::new(target, parse(&args[3]), parse(&args[4]), parse(&args[5]), camera.fov);
        }
        let start = std::time::Instant::now();
        render(&mut framebuffer, &scene, &camera);
        println!("render: {:.0} ms", start.elapsed().as_secs_f32() * 1000.0);
        save_screenshot(&framebuffer, &args[2]);
        return;
    }

    let (mut rl, thread) = raylib::init()
        .size(window_width, window_height)
        .title("Proyecto 2 - Raytracing: Santuario flotante")
        .build();
    rl.set_target_fps(60);

    // Textura de pantalla donde subimos el framebuffer cada vez que cambia.
    let image = Image::gen_image_color(window_width, window_height, RColor::BLACK);
    let mut screen_texture = rl
        .load_texture_from_image(&thread, &image)
        .expect("no se pudo crear la textura del framebuffer");

    // Vista previa: mientras la camara se mueve renderizamos a media
    // resolucion (4 veces menos pixeles) y la mostramos estirada; al soltar
    // las teclas se renderiza una vez a resolucion completa.
    let preview_width = window_width / PREVIEW_SCALE;
    let preview_height = window_height / PREVIEW_SCALE;
    let mut preview_framebuffer = Framebuffer::new(preview_width, preview_height);
    let preview_image = Image::gen_image_color(preview_width, preview_height, RColor::BLACK);
    let mut preview_texture = rl
        .load_texture_from_image(&thread, &preview_image)
        .expect("no se pudo crear la textura de vista previa");

    let mut needs_full_render = true;
    let mut showing_preview = false;
    let mut render_ms = 0.0;
    let mut screenshot_count = 0;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        let start = std::time::Instant::now();
        if handle_camera_input(&rl, &mut camera, dt) {
            // En movimiento: vista previa rapida.
            render(&mut preview_framebuffer, &scene, &camera);
            preview_texture
                .update_texture(&preview_framebuffer.to_rgba_bytes())
                .expect("no se pudo actualizar la textura de vista previa");
            render_ms = start.elapsed().as_secs_f32() * 1000.0;
            showing_preview = true;
            needs_full_render = true;
        } else if needs_full_render {
            // Quieta: render a resolucion completa (una sola vez).
            render(&mut framebuffer, &scene, &camera);
            screen_texture
                .update_texture(&framebuffer.to_rgba_bytes())
                .expect("no se pudo actualizar la textura del framebuffer");
            render_ms = start.elapsed().as_secs_f32() * 1000.0;
            showing_preview = false;
            needs_full_render = false;
        }

        // F12: guardar captura del render actual (para el README).
        if rl.is_key_pressed(KeyboardKey::KEY_F12) {
            std::fs::create_dir_all("screenshots").expect("no se pudo crear la carpeta screenshots");
            screenshot_count += 1;
            save_screenshot(&framebuffer, &format!("screenshots/captura_{}.png", screenshot_count));
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(RColor::BLACK);
        if showing_preview {
            d.draw_texture_ex(&preview_texture, Vector2::new(0.0, 0.0), 0.0, PREVIEW_SCALE as f32, RColor::WHITE);
        } else {
            d.draw_texture(&screen_texture, 0, 0, RColor::WHITE);
        }
        d.draw_fps(10, 10);
        d.draw_text(&format!("render: {:.0} ms", render_ms), 10, 32, 20, RColor::WHITE);
    }
}
