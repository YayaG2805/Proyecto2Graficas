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

// Fuerza de la vineta: cuanto se oscurecen las esquinas (0 = nada).
const VIGNETTE_STRENGTH: f32 = 0.35;

// Vineta: oscurece suavemente los bordes de la imagen, como una lente de
// camara real, para llevar la mirada al centro del diorama. `x` y `y` son la
// posicion del pixel normalizada a [-1, 1] (0 en el centro).
pub fn vignette(color: Color, x: f32, y: f32) -> Color {
    // Distancia al centro al cuadrado: 0 en el centro y 1 en las esquinas.
    // Se eleva otra vez al cuadrado para que el centro quede casi intacto y
    // solo las esquinas se oscurezcan.
    let d2 = (x * x + y * y) * 0.5;
    color * (1.0 - VIGNETTE_STRENGTH * d2 * d2)
}
