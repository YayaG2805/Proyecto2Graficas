use nalgebra_glm::Vec3;

use crate::color::Color;

pub struct Light {
    pub position: Vec3,
    pub intensity: f32,
    pub color: Color,
    // Distancia a la que la luz ya no ilumina nada. Infinita para el sol y el
    // relleno (estan tan lejos que su intensidad casi no cambia en el diorama).
    pub range: f32,
}

impl Light {
    // Luz lejana (sol): misma intensidad en todo el diorama.
    pub fn new(position: Vec3, intensity: f32, color: Color) -> Self {
        Light { position, intensity, color, range: f32::INFINITY }
    }

    // Luz puntual cercana (fuego, farol): se debilita con la distancia y se
    // apaga del todo al llegar a `range`.
    pub fn point(position: Vec3, intensity: f32, color: Color, range: f32) -> Self {
        Light { position, intensity, color, range }
    }

    // Atenuacion segun la distancia: 1 junto a la luz y 0 al llegar a `range`.
    // Se usa (1 - (d/r)^2)^2: parecido a 1/d^2 (la luz se reparte en una
    // esfera cada vez mas grande) pero llega a 0 exacto en el alcance, asi
    // que mas alla podemos saltarnos el rayo de sombra.
    pub fn attenuation(&self, distance: f32) -> f32 {
        if self.range.is_infinite() {
            return 1.0;
        }
        let x = (1.0 - (distance / self.range).powi(2)).max(0.0);
        x * x
    }
}
