use raylib::color::Color as RColor;

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

    pub fn pixels(&self) -> &[RColor] {
        &self.pixels
    }

    // Acceso directo a todos los pixeles (fila por fila), para que el render
    // en paralelo pueda repartir filas entre hilos.
    pub fn pixels_mut(&mut self) -> &mut [RColor] {
        &mut self.pixels
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
