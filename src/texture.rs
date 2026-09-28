use std::rc::Rc;

use raylib::color::Color as RColor;
use raylib::prelude::*;

use crate::color::Color;

#[derive(Clone)]
pub struct Texture {
    width: i32,
    height: i32,
    pixels: Rc<Vec<Color>>,
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

        Texture { width, height, pixels: Rc::new(pixels) }
    }

    pub fn from_file(path: &str) -> Self {
        let image = Image::load_image(path).expect("no se pudo cargar la textura");
        Texture::from_image(&image)
    }

    /// Genera una textura de tablero de ajedrez y la guarda en `path`, para tener
    /// un archivo de imagen real en el repositorio en lugar de solo datos en memoria.
    pub fn generate_checkerboard(path: &str, size: i32, squares: i32) -> Self {
        let image = Image::gen_image_checked(
            size,
            size,
            size / squares,
            size / squares,
            RColor::new(200, 170, 110, 255),
            RColor::new(90, 60, 30, 255),
        );
        image.export_image(path);
        Texture::from_image(&image)
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let u = u.rem_euclid(1.0);
        let v = v.rem_euclid(1.0);

        let x = ((u * self.width as f32) as i32).clamp(0, self.width - 1);
        let y = (((1.0 - v) * self.height as f32) as i32).clamp(0, self.height - 1);

        self.pixels[(y * self.width + x) as usize]
    }
}
