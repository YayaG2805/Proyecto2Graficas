use std::f32::consts::TAU;

use crate::color::Color;
use crate::texture::Texture;

// ============================================================
// Generadores de texturas procedurales (estilo pixel-art)
// ============================================================
// Cada textura es de SIZE x SIZE pixeles y representa 1 unidad de mundo
// (la UV esta en unidades de mundo y se repite cada 1.0). Todos los patrones
// usan periodos que dividen SIZE, asi la textura se repite sin costuras.
// No usamos librerias de ruido: el "azar" sale de una funcion hash propia.

const SIZE: i32 = 32;

// Hash entero -> numero en [0, 1). Determinista: la misma entrada siempre da
// el mismo valor, asi que la textura sale identica en cada ejecucion. La
// semilla permite tener ruido distinto para cada textura o capa.
pub fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ seed.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0xffff) as f32 / 65_535.0
}

// Interpola entre dos colores (t = 0 -> a, t = 1 -> b).
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    a * (1.0 - t) + b * t
}

// Ruido suave ("value noise"): valores del hash en los puntos enteros de una
// cuadricula, interpolados suavemente en medio. A diferencia del hash, que
// cambia bruscamente de un pixel a otro, este varia de forma continua.
pub fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (xf, yf) = (x - x.floor(), y - y.floor());
    // Curva suave (smoothstep) para que no se noten las lineas de la cuadricula.
    let (u, v) = (xf * xf * (3.0 - 2.0 * xf), yf * yf * (3.0 - 2.0 * yf));

    let a = hash(xi, yi, seed);
    let b = hash(xi + 1, yi, seed);
    let c = hash(xi, yi + 1, seed);
    let d = hash(xi + 1, yi + 1, seed);
    let top = a + (b - a) * u;
    let bottom = c + (d - c) * u;
    top + (bottom - top) * v
}

// Suma de varias capas ("octavas") de value noise, cada una con el doble de
// detalle y la mitad de fuerza: formas grandes con detalle pequeno encima,
// como las nubes. Resultado entre 0 y 1.
pub fn fractal_noise(x: f32, y: f32, seed: u32) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    for octave in 0..4 {
        total += value_noise(x * frequency, y * frequency, seed + octave) * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    total / 0.9375 // suma de las amplitudes (0.5 + 0.25 + 0.125 + 0.0625)
}

// Recorre todos los pixeles y arma la textura con la funcion `pixel(x, y)`.
fn generate(pixel: impl Fn(i32, i32) -> Color) -> Texture {
    let mut pixels = Vec::with_capacity((SIZE * SIZE) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            pixels.push(pixel(x, y));
        }
    }
    Texture::from_pixels(SIZE, SIZE, pixels)
}

// PIEDRA: bloques de 16x8 px (0.5 x 0.25 unidades) en hileras alternadas,
// con juntas oscuras, un tono distinto por bloque y bisel claro/oscuro.
pub fn stone() -> Texture {
    generate(|x, y| {
        let row = y / 8;
        let offset = if row % 2 == 0 { 0 } else { 8 };
        let shifted_x = (x + offset) % SIZE;
        let brick = shifted_x / 16;
        let (bx, by) = (shifted_x % 16, y % 8);

        if bx == 0 || by == 0 {
            return Color::new(72.0, 68.0, 62.0); // junta
        }

        let base = Color::new(150.0, 142.0, 130.0);
        let brick_tone = 0.82 + 0.28 * hash(brick, row, 11);
        let grain = 0.92 + 0.14 * hash(x, y, 12);
        let bevel = if by == 1 || bx == 1 {
            1.12 // borde superior/izquierdo iluminado
        } else if by == 7 || bx == 15 {
            0.85 // borde inferior/derecho en sombra
        } else {
            1.0
        };
        base * (brick_tone * grain * bevel)
    })
}

// MADERA: tablones horizontales de 8 px con vetas onduladas, una union por
// tablon y algunos nudos.
pub fn wood() -> Texture {
    generate(|x, y| {
        let plank = y / 8;
        let py = y % 8;
        if py == 0 {
            return Color::new(60.0, 36.0, 18.0); // separacion entre tablones
        }

        // Union entre dos tablas a lo largo del tablon.
        let joint_x = (hash(plank, 0, 21) * SIZE as f32) as i32;
        if x == joint_x {
            return Color::new(70.0, 42.0, 22.0);
        }

        let base = Color::new(140.0, 90.0, 50.0);
        let plank_tone = 0.85 + 0.25 * hash(plank, 1, 22);

        // Vetas: bandas a lo largo de x que ondulan un poco.
        let wave = (x as f32 * TAU / 16.0).sin() * 0.8;
        let grain = ((py as f32 + wave + plank as f32 * 2.3) * 1.9).sin();
        let grain_tone = 0.9 + 0.1 * grain;

        // Nudo: un circulo oscuro en la mitad de los tablones.
        let knot_x = (hash(plank, 2, 23) * SIZE as f32) as i32;
        let has_knot = hash(plank, 3, 24) > 0.5;
        let dx = (x - knot_x) as f32;
        let dy = (py - 4) as f32;
        let knot = if has_knot && dx * dx + dy * dy < 3.0 { 0.65 } else { 1.0 };

        let noise = 0.94 + 0.1 * hash(x, y, 25);
        base * (plank_tone * grain_tone * knot * noise)
    })
}

// METAL (oro envejecido): placa cepillada con lineas horizontales, un marco
// grabado y remaches en las esquinas.
pub fn metal() -> Texture {
    generate(|x, y| {
        let base = Color::new(225.0, 175.0, 70.0);

        // Remaches de 2x2 px con sombra abajo a la derecha.
        for (rx, ry) in [(4, 4), (26, 4), (4, 26), (26, 26)] {
            if (x == rx || x == rx + 1) && (y == ry || y == ry + 1) {
                return base * 1.25;
            }
            if (x == rx + 2 && (y == ry || y == ry + 1)) || (y == ry + 2 && (x == rx || x == rx + 1)) {
                return base * 0.6;
            }
        }

        // Marco grabado: linea oscura con un filo brillante al lado.
        let edge = x.min(y).min(SIZE - 1 - x).min(SIZE - 1 - y);
        if edge == 1 {
            return base * 0.62;
        }
        if edge == 2 {
            return base * 1.15;
        }

        // Cepillado: cada fila con un tono ligeramente distinto.
        let brushed = 0.9 + 0.12 * hash(0, y, 31) + 0.05 * hash(x / 4, y, 32);
        base * brushed
    })
}

// CRISTAL: tono cian muy claro con reflejos diagonales y destellos.
pub fn glass() -> Texture {
    generate(|x, y| {
        let base = Color::new(170.0, 225.0, 245.0);
        let highlight = Color::new(240.0, 252.0, 255.0);

        let diagonal = (x + y) % 16;
        if diagonal < 2 {
            return mix(base, highlight, 0.7);
        }
        if hash(x, y, 41) > 0.97 {
            return highlight; // destello
        }
        base * (0.95 + 0.08 * hash(x / 2, y / 2, 42))
    })
}

// AGUA: azul con crestas de ola onduladas y algo de ruido.
pub fn water() -> Texture {
    generate(|x, y| {
        let deep = Color::new(35.0, 105.0, 160.0);
        let crest = Color::new(130.0, 200.0, 235.0);

        // Olas: bandas en x (periodo 16 px) desplazadas por un seno en y
        // (periodo 32 px), asi las crestas serpentean.
        let fx = x as f32 / 16.0 + 0.25 * (y as f32 * TAU / SIZE as f32).sin();
        let wave = (fx * TAU).sin();

        let noise = 0.93 + 0.12 * hash(x, y, 51);
        if wave > 0.9 {
            mix(deep, crest, 0.75)
        } else if wave > 0.6 {
            mix(deep, crest, 0.35) * noise
        } else {
            deep * noise
        }
    })
}

// PASTO: verde con variacion en manchas de 2x2 px, briznas oscuras y
// algunas flores amarillas y blancas.
pub fn grass() -> Texture {
    generate(|x, y| {
        let flower = hash(x, y, 61);
        if flower > 0.988 {
            return if hash(x, y, 62) > 0.5 {
                Color::new(240.0, 215.0, 90.0)
            } else {
                Color::new(235.0, 235.0, 225.0)
            };
        }

        let base = Color::new(95.0, 155.0, 70.0);
        let patch = 0.85 + 0.25 * hash(x / 2, y / 2, 63);
        let blade = if hash(x, y, 64) > 0.85 { 0.78 } else { 1.0 };
        base * (patch * blade)
    })
}

// FUEGO: lenguas de llama verticales que ondulan, de rojo anaranjado en los
// bordes a amarillo casi blanco en el centro, con chispas sueltas. Se usan
// senos con periodos que dividen SIZE para que se repita sin costuras.
pub fn fire() -> Texture {
    generate(|x, y| {
        let ember = Color::new(220.0, 70.0, 20.0);
        let flame = Color::new(255.0, 160.0, 40.0);
        let core = Color::new(255.0, 240.0, 170.0);

        if hash(x, y, 71) > 0.97 {
            return core; // chispa
        }

        // Lenguas: bandas en x (periodo 16 px) que serpentean con y.
        let fx = x as f32 / 16.0 + 0.2 * (y as f32 * TAU / SIZE as f32).sin();
        let tongue = 0.5 + 0.5 * (fx * TAU).sin(); // 0 borde, 1 centro
        let flicker = 0.9 + 0.15 * hash(x / 2, y / 2, 72);
        let t = (tongue * flicker).min(1.0);

        if t > 0.7 {
            mix(flame, core, (t - 0.7) / 0.3)
        } else {
            mix(ember, flame, t / 0.7)
        }
    })
}

// CORTEZA: surcos verticales oscuros (la v sigue el eje y, asi que en un
// tronco quedan a lo largo de el), placas con tonos distintos y musgo verde
// en algunos pixeles.
pub fn bark() -> Texture {
    generate(|x, y| {
        let base = Color::new(95.0, 62.0, 38.0);

        // Surcos: cada 4 px en x, ondulando un poco con y.
        let wobble = ((y as f32 * TAU / 16.0).sin() * 0.8).round() as i32;
        if (x + wobble).rem_euclid(4) == 0 {
            return base * 0.55;
        }

        if hash(x, y / 2, 81) > 0.94 {
            return Color::new(80.0, 105.0, 50.0); // musgo
        }

        // Placas: cada tira entre surcos se parte en tramos de 6 px de alto.
        let strip = (x + wobble).div_euclid(4);
        let plate_tone = 0.85 + 0.3 * hash(strip, y / 6, 82);
        let noise = 0.92 + 0.12 * hash(x, y, 83);
        base * (plate_tone * noise)
    })
}

// ROCA natural (bajo las islas): sin ladrillos. Manchas de 4x4 px con tonos
// distintos, grano fino, grietas oscuras diagonales y vetas de tierra.
pub fn rock() -> Texture {
    generate(|x, y| {
        let base = Color::new(118.0, 108.0, 100.0);
        let earth = Color::new(110.0, 82.0, 60.0);

        // Grietas: dos familias de lineas diagonales que ondulan (los
        // periodos de 32 px mantienen la textura sin costuras). Solo se
        // dibujan en algunas celdas de 8x8 px, asi quedan tramos sueltos en
        // vez de una rejilla regular.
        let crack_a = (x + y + (3.0 * (y as f32 * TAU / 32.0).sin()) as i32).rem_euclid(16);
        let crack_b = (x - 2 * y + (2.0 * (x as f32 * TAU / 32.0).sin()) as i32).rem_euclid(32);
        let show_a = hash(x / 8, y / 8, 94) > 0.5;
        let show_b = hash(x / 8, y / 8, 95) > 0.6;
        if (crack_a == 0 && show_a) || (crack_b == 0 && show_b) {
            return base * 0.5;
        }

        let patch = 0.8 + 0.3 * hash(x / 4, y / 4, 91);
        let grain = 0.9 + 0.15 * hash(x, y, 92);
        let color = base * (patch * grain);
        // Vetas de tierra: algunas filas tiran a cafe.
        if hash(0, y / 3, 93) > 0.75 {
            mix(color, earth * grain, 0.5)
        } else {
            color
        }
    })
}

// HOJAS (copas de arboles y arbustos): racimos de 4x4 px desplazados en
// hileras, cada uno con luz arriba a la izquierda y sombra abajo a la
// derecha, y huecos oscuros entre racimos que dan volumen.
pub fn leaves() -> Texture {
    generate(|x, y| {
        let row = y / 4;
        let shifted_x = x + if row % 2 == 0 { 0 } else { 2 };
        let (cx, cy) = (shifted_x.rem_euclid(4), y % 4);
        let cluster = (shifted_x.div_euclid(4), row);

        let base = Color::new(60.0, 120.0, 50.0);
        if hash(cluster.0, cluster.1, 101) > 0.85 && cx == 3 && cy == 3 {
            return base * 0.45; // hueco entre racimos
        }

        let tone = 0.8 + 0.35 * hash(cluster.0, cluster.1, 102);
        let shade = if cx + cy <= 1 {
            1.25 // borde iluminado
        } else if cx + cy >= 5 {
            0.75 // borde en sombra
        } else {
            1.0
        };
        let noise = 0.92 + 0.12 * hash(x, y, 103);
        base * (tone * shade * noise)
    })
}

// FLOR DE CEREZO: racimos rosados como los de las hojas, con petalos casi
// blancos sueltos y algun centro magenta.
pub fn blossom() -> Texture {
    generate(|x, y| {
        let row = y / 4;
        let shifted_x = x + if row % 2 == 0 { 0 } else { 2 };
        let (cx, cy) = (shifted_x.rem_euclid(4), y % 4);
        let cluster = (shifted_x.div_euclid(4), row);

        let petal = hash(x, y, 111);
        if petal > 0.94 {
            return Color::new(255.0, 240.0, 245.0); // petalo claro
        }
        if petal < 0.03 {
            return Color::new(200.0, 70.0, 130.0); // centro de la flor
        }

        let base = Color::new(240.0, 150.0, 185.0);
        let tone = 0.85 + 0.25 * hash(cluster.0, cluster.1, 112);
        let shade = if cx + cy <= 1 {
            1.12
        } else if cx + cy >= 5 {
            0.78
        } else {
            1.0
        };
        base * (tone * shade)
    })
}

// TELA de los estandartes: rojo profundo con trama de hilos, franjas
// doradas cada 16 px y un rombo dorado entre las franjas.
pub fn cloth() -> Texture {
    generate(|x, y| {
        let red = Color::new(170.0, 30.0, 40.0);
        let gold = Color::new(230.0, 180.0, 70.0);

        let band = y % 16;
        if band == 0 || band == 2 {
            return gold;
        }
        if band == 1 {
            return gold * 0.7;
        }

        // Rombo: |dx| + |dy| pequeno alrededor del centro de cada tramo.
        let dx = ((x % 16) - 8).abs();
        let dy = (band - 9).abs();
        if dx + dy == 3 {
            return gold;
        }
        if dx + dy < 3 {
            return red * 1.2;
        }

        // Trama: hilos verticales y horizontales alternados.
        let weave = if (x + y) % 2 == 0 { 1.05 } else { 0.92 };
        red * (weave * (0.94 + 0.1 * hash(x, y, 131)))
    })
}

// ESCAMAS de dragon: marfil con escamas en forma de arco, en hileras
// desplazadas (como tejas), con borde oscuro abajo y brillo arriba.
pub fn scales() -> Texture {
    generate(|x, y| {
        let row = y / 4;
        let shifted_x = x + if row % 2 == 0 { 0 } else { 2 };
        let (cx, cy) = (shifted_x.rem_euclid(4), y % 4);

        let base = Color::new(235.0, 225.0, 200.0);
        // Borde inferior de cada escama: un arco (mas bajo en el centro).
        let arc = if cx == 0 || cx == 3 { 2 } else { 3 };
        if cy == arc {
            return base * 0.62;
        }
        let shine = if cy == 0 && (cx == 1 || cx == 2) { 1.1 } else { 1.0 };
        let tone = 0.9 + 0.12 * hash(shifted_x.div_euclid(4), row, 141);
        base * (shine * tone)
    })
}

// MEMBRANA de ala: verde azulado que se aclara hacia un lado, con
// nervaduras oscuras diagonales cada 8 px y bordes de hueso claros.
pub fn wing() -> Texture {
    generate(|x, y| {
        let dark = Color::new(25.0, 110.0, 105.0);
        let light = Color::new(110.0, 215.0, 190.0);

        if (x + y / 2) % 8 == 0 {
            return dark * 0.7; // nervadura
        }
        let t = (y as f32 / SIZE as f32 + 0.15 * (x as f32 * TAU / 32.0).sin()).clamp(0.0, 1.0);
        mix(dark, light, t) * (0.94 + 0.1 * hash(x, y, 151))
    })
}

// TEJAS vidriadas: filas de tejas curvas de 4 px de ancho, cada fila tapa
// la parte de arriba de la siguiente; brillo en el lomo de cada teja y
// sombra en la union entre tejas.
pub fn tiles() -> Texture {
    generate(|x, y| {
        let base = Color::new(40.0, 140.0, 125.0);
        let row = y / 6;
        let (cx, cy) = ((x + if row % 2 == 0 { 0 } else { 2 }).rem_euclid(4), y % 6);

        if cy == 5 {
            return base * 0.45; // sombra bajo el borde de la fila de arriba
        }
        if cx == 0 {
            return base * 0.7; // union entre tejas
        }
        // Lomo de la teja: mas claro en el centro (cx = 2).
        let curve = if cx == 2 { 1.25 } else { 1.0 };
        let glaze = 0.92 + 0.12 * hash(x / 4, row, 161);
        base * (curve * glaze)
    })
}
