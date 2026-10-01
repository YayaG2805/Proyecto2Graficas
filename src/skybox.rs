use std::f32::consts::PI;

use nalgebra_glm::{dot, Vec3};

use crate::color::Color;
use crate::procedural::{fractal_noise, hash, mix};

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

const CLOUD_COLOR: Color = Color { r: 255.0, g: 205.0, b: 200.0 };
const FAR_MOUNTAINS: Color = Color { r: 185.0, g: 125.0, b: 160.0 };
const NEAR_MOUNTAINS: Color = Color { r: 105.0, g: 70.0, b: 120.0 };

// Nubes altas: cara iluminada por el sol del atardecer y base en sombra.
const HIGH_CLOUD_LIT: Color = Color { r: 255.0, g: 195.0, b: 165.0 };
const HIGH_CLOUD_SHADE: Color = Color { r: 140.0, g: 85.0, b: 135.0 };

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

// Altura (como elevacion) de una cordillera en cada azimut: suma de senos
// con frecuencias enteras, asi el perfil da la vuelta completa sin cortes.
fn far_ridge(azimuth: f32) -> f32 {
    0.045 + 0.03 * (3.0 * azimuth + 0.5).sin() + 0.018 * (7.0 * azimuth + 2.0).sin() + 0.008 * (17.0 * azimuth + 1.0).sin()
}

fn near_ridge(azimuth: f32) -> f32 {
    0.02 + 0.025 * (5.0 * azimuth + 4.0).sin() + 0.012 * (11.0 * azimuth + 0.3).sin() + 0.006 * (23.0 * azimuth).sin()
}

impl Skybox {
    // Color de la bruma (el aire entre la camara y un objeto) en una
    // direccion: el degradado del cielo mas el resplandor del sol, sin
    // nubes, estrellas ni montanas. Mas barato que sample().
    pub fn haze(&self, direction: &Vec3) -> Color {
        let elevation = direction.y.max(0.0);
        let cos_angle = dot(direction, &self.sun_direction).max(0.0);
        gradient(&SKY_STOPS, elevation) + SUN_GLOW * (cos_angle.powf(8.0) * 0.35)
    }

    pub fn sample(&self, direction: &Vec3) -> Color {
        let elevation = direction.y; // seno del angulo sobre el horizonte
        let azimuth = direction.z.atan2(direction.x); // angulo horizontal, -PI..PI

        let mut color = if elevation >= 0.0 {
            gradient(&SKY_STOPS, elevation)
        } else {
            gradient(&BELOW_STOPS, -elevation)
        };

        // Estrellas: dividimos el cielo en celdas pequenas (por azimut y
        // elevacion) y encendemos unas pocas al azar con el hash. Aparecen
        // solo en la parte alta y se desvanecen hacia el horizonte.
        if elevation > 0.25 {
            let cell_x = (azimuth / PI * 900.0).floor() as i32;
            let cell_y = (elevation * 600.0).floor() as i32;
            if hash(cell_x, cell_y, 71) > 0.997 {
                let fade = ((elevation - 0.25) / 0.35).min(1.0);
                let brightness = 0.5 + 0.5 * hash(cell_x, cell_y, 72);
                color = mix(color, Color::new(255.0, 250.0, 240.0), fade * brightness);
            }
        }

        // Nubes sobre el horizonte: igual que el mar de nubes, la direccion
        // se proyecta sobre un plano (ahora arriba) y ahi se evalua el ruido.
        // Para sombrearlas se vuelve a leer el ruido un poco mas hacia el sol:
        // si ahi hay mas nube, este punto queda "detras" y en sombra; si hay
        // menos, es el borde que mira al sol y se ilumina.
        if elevation > 0.02 {
            let plane_x = direction.x / elevation;
            let plane_z = direction.z / elevation;
            let density = fractal_noise(plane_x * 0.45 + 10.0, plane_z * 0.45, 301);
            let cloud = ((density - 0.5) / 0.2).clamp(0.0, 1.0);
            if cloud > 0.0 {
                let toward_sun = fractal_noise(
                    (plane_x + self.sun_direction.x * 0.6) * 0.45 + 10.0,
                    (plane_z + self.sun_direction.z * 0.6) * 0.45,
                    301,
                );
                let light = (0.6 + (density - toward_sun) * 4.0).clamp(0.0, 1.0);
                let shade = mix(HIGH_CLOUD_SHADE, HIGH_CLOUD_LIT, light);
                // Se desvanecen al subir (cerca del cenit el cielo esta
                // despejado y se ven las estrellas) y muy cerca del horizonte.
                let band = ((elevation - 0.02) / 0.06).min(1.0) * (1.0 - ((elevation - 0.2) / 0.35).clamp(0.0, 1.0));
                color = mix(color, shade, cloud * band * 0.9);
            }
        }

        // Montanas lejanas en silueta, asomando sobre el mar de nubes. La mas
        // lejana es mas clara porque la bruma la aclara (perspectiva atmosferica).
        if elevation > -0.01 {
            if elevation < near_ridge(azimuth) {
                color = NEAR_MOUNTAINS;
            } else if elevation < far_ridge(azimuth) {
                color = FAR_MOUNTAINS;
            }
        }

        // Mar de nubes debajo del horizonte: la direccion se proyecta sobre un
        // plano de nubes imaginario (dividir entre la elevacion: cerca del
        // horizonte los puntos quedan lejos y las nubes se ven mas pequenas) y
        // ahi se evalua el ruido fractal. Cerca del horizonte se funden con la
        // bruma para que no se vea ruido comprimido.
        if elevation < 0.0 {
            let depth = -elevation;
            let plane_x = direction.x / depth;
            let plane_z = direction.z / depth;
            let density = fractal_noise(plane_x * 0.6, plane_z * 0.6, 81);
            let cloud = ((density - 0.42) / 0.25).clamp(0.0, 1.0);
            let horizon_fade = (depth / 0.12).min(1.0);
            color = mix(color, CLOUD_COLOR * (0.75 + 0.25 * cloud), cloud * horizon_fade * 0.85);
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
