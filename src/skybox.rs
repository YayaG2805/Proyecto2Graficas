use std::f32::consts::PI;

use nalgebra_glm::{dot, Vec3};

use crate::color::Color;
use crate::procedural::{hash, mix};

// ============================================================
// Skybox procedural: cielo de atardecer
// ============================================================
// Cuando un rayo no golpea nada, en vez de un color fijo se consulta el
// skybox con la DIRECCION del rayo. Como solo depende de la direccion (no de
// la posicion), el cielo se ve infinitamente lejano: no cambia al hacer
// zoom, solo al rotar la camara. Tambien es lo que ven los rayos reflejados
// y refractados que escapan de la escena.

pub struct Skybox {
    // Direccion (normalizada) hacia el sol; debe coincidir con la luz del sol
    // para que las sombras sean coherentes con el disco que se ve en el cielo.
    pub sun_direction: Vec3,
}

// Degradado del cielo segun la elevacion (0 = horizonte, 1 = cenit).
const SKY_STOPS: [(f32, Color); 6] = [
    (0.00, Color { r: 255.0, g: 180.0, b: 120.0 }), // durazno en el horizonte
    (0.08, Color { r: 250.0, g: 140.0, b: 120.0 }), // coral
    (0.20, Color { r: 200.0, g: 100.0, b: 150.0 }), // magenta
    (0.40, Color { r: 100.0, g: 70.0, b: 150.0 }),  // purpura
    (0.70, Color { r: 35.0, g: 40.0, b: 100.0 }),   // azul profundo
    (1.00, Color { r: 15.0, g: 20.0, b: 60.0 }),    // azul noche (cenit)
];

// Debajo del horizonte (0 = horizonte, 1 = directamente abajo).
const BELOW_STOPS: [(f32, Color); 4] = [
    (0.00, Color { r: 240.0, g: 170.0, b: 150.0 }),
    (0.10, Color { r: 170.0, g: 120.0, b: 160.0 }),
    (0.35, Color { r: 90.0, g: 70.0, b: 130.0 }),
    (1.00, Color { r: 40.0, g: 35.0, b: 80.0 }),
];

const SUN_COLOR: Color = Color { r: 255.0, g: 245.0, b: 220.0 };
const SUN_GLOW: Color = Color { r: 255.0, g: 190.0, b: 130.0 };

// Interpola entre las "paradas" de un degradado (t entre 0 y 1).
fn gradient(stops: &[(f32, Color)], t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    for pair in stops.windows(2) {
        let (t0, c0) = pair[0];
        let (t1, c1) = pair[1];
        if t <= t1 {
            return mix(c0, c1, (t - t0) / (t1 - t0));
        }
    }
    stops[stops.len() - 1].1
}

impl Skybox {
    pub fn sample(&self, direction: &Vec3) -> Color {
        let elevation = direction.y; // seno del angulo sobre el horizonte

        let mut color = if elevation >= 0.0 {
            gradient(&SKY_STOPS, elevation)
        } else {
            gradient(&BELOW_STOPS, -elevation)
        };

        // Estrellas: dividimos el cielo en celdas pequenas (por azimut y
        // elevacion) y encendemos unas pocas al azar con el hash. Aparecen
        // solo en la parte alta y se desvanecen hacia el horizonte.
        if elevation > 0.25 {
            let azimuth = direction.z.atan2(direction.x); // -PI..PI
            let cell_x = (azimuth / PI * 900.0).floor() as i32;
            let cell_y = (elevation * 600.0).floor() as i32;
            if hash(cell_x, cell_y, 71) > 0.997 {
                let fade = ((elevation - 0.25) / 0.35).min(1.0);
                let brightness = 0.5 + 0.5 * hash(cell_x, cell_y, 72);
                color = mix(color, Color::new(255.0, 250.0, 240.0), fade * brightness);
            }
        }

        // Sol: resplandor amplio + halo cercano + disco. cos_angle es 1 justo
        // en la direccion del sol y baja al alejarse.
        let cos_angle = dot(direction, &self.sun_direction).max(0.0);
        let glow = cos_angle.powf(8.0) * 0.35 + cos_angle.powf(64.0) * 0.6;
        color = color + SUN_GLOW * glow;
        if cos_angle > 0.9994 {
            color = SUN_COLOR;
        }

        color
    }
}
