use raylib::color::Color as RColor;

use crate::color::Color;

// Framebuffer: un arreglo de colores (uno por pixel) sobre el que dibujamos
// manualmente. Cada frame lo convertimos a bytes RGBA y lo subimos como
// textura a raylib (igual que en el Proyecto 1), porque llamar draw_pixel
// cientos de miles de veces por frame es demasiado lento para una camara
// interactiva.
pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    pixels: Vec<RColor>,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32) -> Self {
        Framebuffer {
            width,
            height,
            pixels: vec![RColor::BLACK; (width * height) as usize],
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let index = (y * self.width + x) as usize;
        self.pixels[index] = color.to_raylib();
    }

    // Convierte el buffer a bytes RGBA, el formato que espera update_texture.
    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.pixels.len() * 4);
        for color in &self.pixels {
            bytes.push(color.r);
            bytes.push(color.g);
            bytes.push(color.b);
            bytes.push(color.a);
        }
        bytes
    }
}
