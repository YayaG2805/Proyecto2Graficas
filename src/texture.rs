use std::sync::Arc;

use raylib::color::Color as RColor;
use raylib::prelude::*;

use crate::color::Color;

#[derive(Clone)]
pub struct Texture {
    width: i32,
    height: i32,
    pixels: Arc<Vec<Color>>,
}

impl Texture {
    fn from_image(image: &Image) -> Self {
        let width = image.width();
        let height = image.height();
        let mut pixels = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let c = image.get_color(x, y);
                pixels.push(Color::new(c.r as f32, c.g as f32, c.b as f32));
            }
        }

        Texture { width, height, pixels: Arc::new(pixels) }
    }

    pub fn from_file(path: &str) -> Self {
        let image = Image::load_image(path).expect("no se pudo cargar la textura");
        Texture::from_image(&image)
    }

    // Textura a partir de un arreglo de colores (fila por fila), por ejemplo
    // el resultado de un generador procedural.
    pub fn from_pixels(width: i32, height: i32, pixels: Vec<Color>) -> Self {
        assert_eq!(pixels.len(), (width * height) as usize);
        Texture { width, height, pixels: Arc::new(pixels) }
    }

    // Exporta la textura como PNG usando Image de raylib (igual que en clase
    // se exportaba el tablero), para tener archivos de imagen reales.
    pub fn save_png(&self, path: &str) {
        let mut image = Image::gen_image_color(self.width, self.height, RColor::BLACK);
        for y in 0..self.height {
            for x in 0..self.width {
                let c = self.pixels[(y * self.width + x) as usize];
                image.draw_pixel(x, y, c.to_raylib());
            }
        }
        image.export_image(path);
    }

    // Si el PNG ya existe lo carga; si no, lo genera con `generate`, lo guarda
    // y lo usa. Asi las texturas quedan como archivos en assets/ y se pueden
    // reemplazar por cualquier otra imagen sin tocar el codigo.
    // (Para regenerarlas despues de cambiar un generador, borrar el PNG.)
    pub fn load_or_generate(path: &str, generate: fn() -> Texture) -> Self {
        if std::path::Path::new(path).exists() {
            return Texture::from_file(path);
        }
        if let Some(dir) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(dir).expect("no se pudo crear la carpeta de texturas");
        }
        let texture = generate();
        texture.save_png(path);
        texture
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let u = u.rem_euclid(1.0);
        let v = v.rem_euclid(1.0);

        let x = ((u * self.width as f32) as i32).clamp(0, self.width - 1);
        let y = (((1.0 - v) * self.height as f32) as i32).clamp(0, self.height - 1);

        self.pixels[(y * self.width + x) as usize]
    }
}
