use crate::color::Color;

// ============================================================
// Post-proceso: ajustes al color final de cada pixel
// ============================================================
// Se aplican despues de cast_ray, justo antes de escribir el pixel.

// A partir de este valor (0-255) los canales se comprimen en vez de cortarse.
const KNEE: f32 = 200.0;

// Tone mapping con "rodilla" suave. La luz calculada puede pasar de 255
// (fuego emisivo, el sol, brillos especulares sumados). Si solo se corta en
// 255, esas zonas quedan como manchas planas y el color se vuelve blanco o
// amarillo puro. Debajo de KNEE el canal no cambia; arriba se acerca a 255
// con una curva exponencial que nunca lo alcanza, asi se conservan el
// degradado y el tono del fuego y del sol.
fn soft_clip(channel: f32) -> f32 {
    if channel <= KNEE {
        return channel;
    }
    let range = 255.0 - KNEE;
    KNEE + range * (1.0 - (-(channel - KNEE) / range).exp())
}

pub fn tone_map(color: Color) -> Color {
    Color::new(soft_clip(color.r), soft_clip(color.g), soft_clip(color.b))
}
