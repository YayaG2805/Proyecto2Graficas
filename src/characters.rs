use nalgebra_glm::Vec3;

use crate::cube::Cube;
use crate::placement::{Heading, Placement};
use crate::procedural;
use crate::scene::Palette;

// ============================================================
// Personajes voxel: All Might (gigante) y Asta
// ============================================================
// Igual que los dragones, cada personaje se describe en coordenadas locales
// (f = adelante, y = arriba, s = su derecha) con los pies en y = 0, y
// `Placement` lo lleva al mundo con su direccion y escala. Las medidas estan
// pensadas como una figura de accion: proporciones exageradas (torso enorme
// y cintura chica en All Might, cabeza grande en Asta) para que se
// reconozcan como en el anime aunque esten hechos de cajas.

// All Might en su pose de victoria: el puno izquierdo levantado al cielo y
// el brazo derecho relajado. Traje azul marino con paneles rojos en el pecho
// y el abdomen, blanco junto al cuello, cinturon amarillo, franjas blancas y
// rojas a los lados de las piernas y botas rojas. La cara queda en sombra
// (como en el anime) y solo brillan los ojos; arriba, sus dos mechones en V.
pub fn all_might(o: &mut Vec<Cube>, p: &Palette, origin: Vec3, heading: Heading, scale: f32) {
    let d = Placement::new(origin, heading, scale);

    // ---- Piernas ----
    for side in [-1.0f32, 1.0] {
        let c = 0.55 * side;
        // Bota roja con una franja blanca y la punta amarilla.
        d.place(o, (-0.38, 0.0, c - 0.34), (0.42, 0.85, c + 0.34), p.suit_red);
        d.place(o, (-0.4, 0.55, c - 0.36), (0.44, 0.7, c + 0.36), p.suit_white);
        d.place(o, (-0.4, 0.0, c - 0.36), (0.5, 0.15, c + 0.36), p.suit_yellow);
        // Espinilla, rodilla y muslo (mas grueso arriba).
        d.place(o, (-0.3, 0.85, c - 0.3), (0.32, 2.2, c + 0.3), p.suit_blue);
        d.place(o, (-0.33, 2.0, c - 0.33), (0.36, 2.4, c + 0.33), p.suit_blue);
        d.place(o, (-0.45, 2.3, c - 0.45), (0.45, 3.5, c + 0.45), p.suit_blue);
        // Franja blanca por fuera de la pierna, con bordes rojos. Sobresale
        // apenas de la cara exterior para no compartir la cara con el muslo.
        let outer = c + side * 0.45;
        let (s0, s1) = (outer.min(outer + side * 0.03), outer.max(outer + side * 0.03));
        d.place(o, (-0.18, 2.3, s0), (0.18, 3.5, s1), p.suit_white);
        d.place(o, (0.18, 2.3, s0), (0.27, 3.5, s1), p.suit_red);
        d.place(o, (-0.27, 2.3, s0), (-0.18, 3.5, s1), p.suit_red);
        // En la espinilla la franja sigue sobre la cara exterior mas angosta.
        let shin = c + side * 0.3;
        let (s0, s1) = (shin.min(shin + side * 0.03), shin.max(shin + side * 0.03));
        d.place(o, (-0.14, 0.85, s0), (0.14, 2.3, s1), p.suit_white);
        d.place(o, (0.14, 0.85, s0), (0.21, 2.3, s1), p.suit_red);
        d.place(o, (-0.21, 0.85, s0), (-0.14, 2.3, s1), p.suit_red);
    }

    // ---- Cadera, cinturon y abdomen ----
    d.place(o, (-0.45, 3.3, -0.95), (0.45, 3.85, 0.95), p.suit_blue);
    d.place(o, (-0.48, 3.75, -0.92), (0.48, 4.05, 0.92), p.suit_yellow);
    d.place(o, (0.48, 3.72, -0.2), (0.53, 4.08, 0.2), p.metal); // hebilla
    // Solapa blanca en V bajo el cinturon.
    d.place(o, (0.45, 3.35, -0.35), (0.47, 3.75, 0.35), p.suit_white);
    d.place(o, (0.45, 3.1, -0.18), (0.47, 3.35, 0.18), p.suit_white);
    // Cintura (angosta) con el panel rojo del abdomen y las lineas de los
    // abdominales marcadas en azul.
    d.place(o, (-0.42, 4.05, -0.78), (0.44, 4.7, 0.78), p.suit_blue);
    d.place(o, (0.44, 4.1, -0.42), (0.48, 5.0, 0.42), p.suit_red);
    for y in [4.38, 4.68] {
        d.place(o, (0.48, y, -0.42), (0.49, y + 0.03, 0.42), p.suit_blue);
    }
    d.place(o, (0.48, 4.1, -0.02), (0.49, 5.0, 0.02), p.suit_blue);

    // ---- Torso en V ----
    d.place(o, (-0.55, 4.7, -1.1), (0.6, 5.4, 1.1), p.suit_blue);
    d.place(o, (0.6, 4.75, -0.5), (0.64, 5.35, 0.5), p.suit_red);
    d.place(o, (-0.6, 5.4, -1.4), (0.66, 6.35, 1.4), p.suit_blue);
    for side in [-1.0f32, 1.0] {
        let (a, b) = (side * 0.05, side * 1.05);
        let (s0, s1) = (a.min(b), a.max(b));
        // Pectoral: rojo abajo, azul en medio y blanco junto al cuello.
        d.place(o, (0.66, 5.45, s0), (0.82, 5.85, s1), p.suit_red);
        d.place(o, (0.66, 5.85, s0), (0.8, 6.05, s1), p.suit_blue);
        let (a, b) = (side * 0.35, side * 1.15);
        d.place(o, (0.66, 6.05, a.min(b)), (0.78, 6.35, a.max(b)), p.suit_white);
        // Franja roja en el costado (dorsales).
        let edge = side * 1.4;
        d.place(o, (-0.1, 5.0, edge.min(edge + side * 0.03)), (0.3, 5.9, edge.max(edge + side * 0.03)), p.suit_red);
        // Trapecios y deltoides (hombros enormes) con el borde blanco arriba.
        let (a, b) = (side * 0.4, side * 1.25);
        d.place(o, (-0.45, 6.25, a.min(b)), (0.35, 6.55, a.max(b)), p.suit_blue);
        let (a, b) = (side * 1.3, side * 2.15);
        d.place(o, (-0.6, 5.5, a.min(b)), (0.6, 6.55, a.max(b)), p.suit_blue);
        let (a, b) = (side * 1.35, side * 2.1);
        d.place(o, (-0.4, 6.55, a.min(b)), (0.4, 6.62, a.max(b)), p.suit_white);
    }

    // ---- Cuello y cabeza ----
    d.place(o, (-0.38, 6.3, -0.48), (0.42, 6.45, 0.48), p.suit_white); // cuello del traje
    d.place(o, (-0.28, 6.35, -0.36), (0.3, 6.75, 0.36), p.skin);
    d.place(o, (-0.45, 6.75, -0.45), (0.5, 7.8, 0.45), p.skin);
    d.place(o, (-0.3, 6.65, -0.4), (0.56, 7.0, 0.4), p.skin); // mandibula cuadrada
    // Ojos hundidos en sombra (cuencas oscuras bajo la ceja) con un punto
    // que brilla en cada una, como en el anime.
    for side in [-1.0f32, 1.0] {
        let (a, b) = (side * 0.06, side * 0.34);
        d.place(o, (0.5, 7.2, a.min(b)), (0.52, 7.46, a.max(b)), p.black_cloth);
        let (a, b) = (side * 0.15, side * 0.25);
        d.place(o, (0.52, 7.28, a.min(b)), (0.54, 7.36, a.max(b)), p.magic);
    }
    d.place(o, (0.5, 7.48, -0.42), (0.58, 7.58, 0.42), p.skin); // ceja
    d.place(o, (0.5, 7.05, -0.07), (0.63, 7.3, 0.07), p.skin); // nariz
    // La sonrisa enorme: dientes blancos con separaciones y labio oscuro.
    d.place(o, (0.56, 6.82, -0.32), (0.59, 6.98, 0.32), p.suit_white);
    for s in [-0.16, 0.0, 0.16] {
        d.place(o, (0.59, 6.82, s - 0.008), (0.595, 6.98, s + 0.008), p.black_cloth);
    }
    d.place(o, (0.56, 6.98, -0.34), (0.6, 7.01, 0.34), p.black_cloth);

    // ---- Cabello ----
    d.place(o, (-0.55, 7.7, -0.5), (0.45, 8.0, 0.5), p.hair_gold); // casco
    d.place(o, (-0.58, 6.9, -0.5), (-0.42, 7.9, 0.5), p.hair_gold); // nuca
    for side in [-1.0f32, 1.0] {
        let (a, b) = (side * 0.45, side * 0.52);
        d.place(o, (-0.5, 7.2, a.min(b)), (0.2, 7.9, a.max(b)), p.hair_gold); // patillas
        let (a, b) = (side * 0.2, side * 0.45);
        d.place(o, (0.45, 7.45, a.min(b)), (0.55, 7.85, a.max(b)), p.hair_gold); // flequillo
        // Los dos mechones en V: escalones que suben y se abren hacia
        // afuera, cada uno mas delgado, hasta una punta fina.
        for i in 0..7 {
            let t = i as f32;
            let half = 0.26 - 0.03 * t;
            let s = side * (0.12 + 0.17 * t);
            let y = 7.85 + 0.32 * t;
            let f = 0.3 - 0.04 * t;
            d.place(o, (f - half, y, s - half * 0.8), (f + half, y + 0.42, s + half * 0.8), p.hair_gold);
        }
        // Mechones cortos y en punta alrededor de la coronilla.
        for (f, s_off, h) in [(-0.35, 0.3, 0.35), (-0.05, 0.4, 0.3), (-0.45, 0.05, 0.4)] {
            let s = side * s_off;
            d.place(o, (f - 0.14, 7.95, s - 0.12), (f + 0.14, 7.95 + h, s + 0.12), p.hair_gold);
        }
    }

    // ---- Brazo derecho, relajado (piel, sin mangas) ----
    d.place(o, (-0.42, 4.4, 1.45), (0.42, 5.6, 2.2), p.skin);
    d.place(o, (0.42, 4.7, 1.55), (0.54, 5.4, 2.1), p.skin); // biceps
    d.place(o, (-0.36, 3.1, 1.5), (0.38, 4.4, 2.12), p.skin);
    d.place(o, (-0.4, 2.55, 1.45), (0.42, 3.1, 2.15), p.skin); // puno

    // ---- Brazo izquierdo, levantado al cielo ----
    d.place(o, (-0.42, 6.3, -2.2), (0.42, 7.7, -1.45), p.skin);
    d.place(o, (0.42, 6.6, -2.1), (0.54, 7.5, -1.55), p.skin); // biceps
    d.place(o, (-0.37, 7.7, -2.12), (0.37, 8.9, -1.5), p.skin);
    d.place(o, (-0.45, 8.9, -2.25), (0.47, 9.65, -1.4), p.skin); // puno
    d.place(o, (0.47, 9.3, -2.2), (0.53, 9.6, -1.45), p.skin); // nudillos
    d.place(o, (0.47, 9.02, -1.65), (0.56, 9.25, -1.42), p.skin); // pulgar
}

// Isla de All Might: una isla grande para que el gigante quede de pie, con
// roca escalonada debajo y unas grietas en el pasto alrededor de sus pies
// (como si acabara de aterrizar).
pub fn all_might_island(o: &mut Vec<Cube>, p: &Palette, cx: f32, top: f32, cz: f32) {
    let add = |o: &mut Vec<Cube>, min: (f32, f32, f32), max: (f32, f32, f32), m: usize| {
        o.push(Cube::from_min_max(Vec3::new(min.0, min.1, min.2), Vec3::new(max.0, max.1, max.2), m));
    };
    add(o, (cx - 3.5, top - 0.4, cz - 3.0), (cx + 3.5, top, cz + 3.0), p.grass);
    add(o, (cx - 3.2, top - 1.6, cz - 2.7), (cx + 3.2, top - 0.4, cz + 2.7), p.rock);
    add(o, (cx - 2.3, top - 2.8, cz - 1.9), (cx + 2.2, top - 1.6, cz + 2.0), p.rock);
    add(o, (cx - 1.2, top - 3.9, cz - 1.0), (cx + 1.3, top - 2.8, cz + 1.1), p.rock);
    add(o, (cx - 0.4, top - 4.8, cz - 0.3), (cx + 0.5, top - 3.9, cz + 0.4), p.rock);
    // Grietas: tiras de roca hundidas en el pasto, en estrella.
    for i in 0..7 {
        let angle = i as f32 / 7.0 * std::f32::consts::TAU + procedural::hash(i, 0, 351);
        let length = 1.2 + 1.2 * procedural::hash(i, 1, 352);
        for k in 0..4 {
            let r = 0.9 + length * k as f32 / 4.0;
            let (x, z) = (cx + angle.cos() * r, cz + angle.sin() * r);
            add(o, (x - 0.1, top - 0.05, z - 0.1), (x + 0.1, top + 0.01, z + 0.1), p.rock);
        }
    }
}

// Asta en su forma de antimagia, sosteniendo la espada mata demonios en
// diagonal frente a el (con la mano izquierda, la hoja sube hacia su
// derecha). Media cara cubierta de antimagia negra con el ojo rojo
// encendido, el otro ojo verde, sonrisa desafiante, bandana con emblema,
// cabello blanco en picos y el cuerno negro; la capa de los Toros Negros con
// borde dorado. Alrededor, un aura roja y negra de antimagia.
pub fn asta(o: &mut Vec<Cube>, p: &Palette, origin: Vec3, heading: Heading, scale: f32) {
    let d = Placement::new(origin, heading, scale);

    // ---- Piernas abiertas (postura de combate) ----
    for side in [-1.0f32, 1.0] {
        let c = 0.16 * side;
        d.place(o, (-0.09, 0.0, c - 0.09), (0.14, 0.28, c + 0.09), p.leather); // bota
        d.place(o, (-0.08, 0.28, c - 0.085), (0.09, 0.95, c + 0.085), p.black_cloth); // pantalon
    }

    // ---- Torso, cinturon y capa de los Toros Negros ----
    d.place(o, (-0.11, 0.95, -0.21), (0.11, 1.32, 0.21), p.black_cloth);
    d.place(o, (-0.12, 1.3, -0.23), (0.12, 1.5, 0.23), p.black_cloth); // pecho
    d.place(o, (-0.12, 0.9, -0.22), (0.12, 0.98, 0.22), p.leather); // cinturon
    d.place(o, (0.12, 0.91, -0.04), (0.13, 0.97, 0.04), p.metal); // hebilla
    d.place(o, (-0.16, 0.92, -0.25), (-0.12, 1.53, 0.25), p.black_cloth); // capa
    d.place(o, (-0.165, 0.9, -0.255), (-0.115, 0.94, 0.255), p.metal); // borde dorado
    d.place(o, (-0.16, 1.47, -0.28), (0.14, 1.56, 0.28), p.black_cloth); // hombrera
    d.place(o, (-0.19, 1.5, -0.13), (-0.06, 1.63, 0.13), p.black_cloth); // capucha recogida
    // Emblema del toro en la espalda: cabeza dorada con dos cuernos.
    d.place(o, (-0.17, 1.15, -0.05), (-0.16, 1.27, 0.05), p.metal);
    for side in [-1.0f32, 1.0] {
        let (a, b) = (side * 0.05, side * 0.11);
        d.place(o, (-0.17, 1.25, a.min(b)), (-0.16, 1.31, a.max(b)), p.metal);
    }

    // ---- Cabeza ----
    d.place(o, (-0.06, 1.5, -0.07), (0.07, 1.6, 0.07), p.skin); // cuello
    d.place(o, (-0.14, 1.6, -0.14), (0.15, 1.95, 0.14), p.skin);
    // Mitad derecha de la cara cubierta de antimagia, con el ojo rojo.
    d.place(o, (0.15, 1.68, 0.0), (0.16, 1.86, 0.15), p.black_cloth);
    d.place(o, (0.16, 1.76, 0.04), (0.17, 1.81, 0.1), p.antimagic);
    // Ojo izquierdo verde (brilla con luz propia, como en la imagen).
    d.place(o, (0.15, 1.76, -0.1), (0.16, 1.81, -0.04), p.firefly);
    // Ceja fruncida y sonrisa con los dientes apretados.
    d.place(o, (0.15, 1.82, -0.12), (0.165, 1.84, -0.02), p.black_cloth);
    d.place(o, (0.15, 1.64, -0.08), (0.165, 1.68, 0.07), p.suit_white);
    d.place(o, (0.165, 1.655, -0.08), (0.168, 1.665, 0.07), p.black_cloth);
    // Bandana negra con el emblema rojo al frente.
    d.place(o, (-0.15, 1.82, -0.15), (0.16, 1.89, 0.15), p.black_cloth);
    d.place(o, (0.16, 1.83, -0.06), (0.17, 1.88, 0.03), p.suit_red);
    d.place(o, (-0.22, 1.8, -0.03), (-0.15, 1.86, 0.03), p.black_cloth); // nudo atras

    // ---- Cabello blanco en picos ----
    d.place(o, (-0.16, 1.88, -0.16), (0.15, 2.0, 0.16), p.hair_white);
    // Picos: (f, s, alto, medio ancho). Suben y la punta se corre hacia atras.
    let spikes = [
        (0.08, -0.1, 0.16, 0.05),
        (0.05, 0.02, 0.2, 0.05),
        (-0.02, -0.14, 0.18, 0.05),
        (-0.1, 0.0, 0.22, 0.06),
        (-0.12, -0.12, 0.17, 0.05),
        (-0.16, 0.12, 0.14, 0.05),
        (0.1, 0.12, 0.12, 0.04),
        (-0.05, 0.16, 0.13, 0.04),
        (0.02, -0.18, 0.12, 0.04),
    ];
    for (f, s, h, w) in spikes {
        d.place(o, (f - w, 1.98, s - w), (f + w, 1.98 + h, s + w), p.hair_white);
        let (tf, tw) = (f - 0.03, w * 0.5);
        d.place(o, (tf - tw, 1.98 + h, s - tw), (tf + tw, 1.98 + h + 0.06, s + tw), p.hair_white);
    }
    // Mechon cayendo sobre la frente.
    d.place(o, (0.13, 1.88, -0.14), (0.17, 1.95, -0.02), p.hair_white);
    // Cuerno negro que sale del lado derecho de la cabeza y se curva arriba.
    let horn = [(0.0, 0.15, 1.95), (-0.01, 0.19, 2.07), (-0.03, 0.22, 2.19), (-0.06, 0.23, 2.31), (-0.09, 0.22, 2.43), (-0.12, 0.2, 2.53)];
    for (i, (f, s, y)) in horn.iter().enumerate() {
        let w = 0.055 - 0.008 * i as f32;
        d.place(o, (f - w, *y, s - w), (f + w, y + 0.11, s + w), p.obsidian);
    }

    // ---- Brazo izquierdo: sostiene la espada frente al cuerpo ----
    d.place(o, (-0.06, 1.2, -0.31), (0.06, 1.5, -0.21), p.black_cloth);
    d.place(o, (0.0, 1.08, -0.34), (0.3, 1.18, -0.25), p.skin); // antebrazo
    let hand = (0.36, 1.12, -0.3); // centro del puno (f, y, s)
    d.place(o, (hand.0 - 0.06, hand.1 - 0.08, hand.2 - 0.07), (hand.0 + 0.06, hand.1 + 0.08, hand.2 + 0.07), p.skin);

    // ---- Brazo derecho: cubierto de antimagia (negro y rojo) ----
    d.place(o, (-0.06, 1.2, 0.21), (0.06, 1.5, 0.31), p.black_cloth);
    d.place(o, (-0.06, 0.92, 0.22), (0.06, 1.2, 0.32), p.antimagic);
    d.place(o, (-0.07, 0.82, 0.21), (0.07, 0.94, 0.33), p.black_cloth); // puno

    // ---- Espada mata demonios ----
    // La hoja sube en diagonal (en el plano frente a Asta) desde el puno
    // hacia su derecha. Las cajas no se pueden inclinar, asi que la hoja es
    // una escalera de cajas superpuestas a lo largo de la direccion (como la
    // cuerda de la guirnalda); de lejos se lee como una hoja inclinada.
    let (dir_s, dir_y) = (0.82f32, 0.572f32); // direccion de la hoja (normalizada, ~35 grados)
    let along = |t: f32| (hand.2 + dir_s * t, hand.1 + dir_y * t); // (s, y) a distancia t
    let thickness = 0.03; // medio grosor de la hoja (en f)
    let step = 0.09;
    // Mango vendado hacia abajo del puno y pomo de hierro.
    for k in 1..=3 {
        let (s, y) = along(-(k as f32) * step * 0.8);
        d.place(o, (hand.0 - 0.035, y - 0.05, s - 0.04), (hand.0 + 0.035, y + 0.05, s + 0.04), p.wrap);
    }
    let (s, y) = along(-4.0 * step * 0.8);
    d.place(o, (hand.0 - 0.05, y - 0.05, s - 0.05), (hand.0 + 0.05, y + 0.05, s + 0.05), p.demon_iron);
    // Guarda: bloque ancho justo sobre el puno.
    let (s, y) = along(0.1);
    d.place(o, (hand.0 - 0.07, y - 0.08, s - 0.12), (hand.0 + 0.07, y + 0.08, s + 0.12), p.demon_iron);
    // Hoja: ancha y pesada; la punta es mas angosta y tiene mellas.
    let blade_steps = 18;
    for k in 0..blade_steps {
        let (s, y) = along(0.18 + k as f32 * step);
        let width = if k + 2 >= blade_steps { 0.09 } else { 0.15 };
        d.place(o, (hand.0 - thickness, y - width, s - width * 0.7), (hand.0 + thickness, y + width, s + width * 0.7), p.demon_iron);
        // Mellas: picos sueltos en el filo superior, cada tanto.
        if k % 4 == 2 {
            let (es, ey) = (s - dir_y * 0.17, y + dir_s * 0.17);
            d.place(o, (hand.0 - thickness * 0.8, ey - 0.03, es - 0.03), (hand.0 + thickness * 0.8, ey + 0.03, es + 0.03), p.demon_iron);
        }
    }

    // ---- Aura de antimagia ----
    // Chispas y lenguas rojas y negras alrededor del cuerpo, en posiciones
    // fijas sacadas del hash. Las lenguas son cajas altas y delgadas (llamas
    // que suben); las chispas, cubitos sueltos.
    for i in 0..36 {
        let angle = procedural::hash(i, 0, 361) * std::f32::consts::TAU;
        let radius = 0.35 + 0.45 * procedural::hash(i, 1, 362);
        let f = angle.cos() * radius - 0.1;
        let s = angle.sin() * radius;
        let y = 0.1 + 2.1 * procedural::hash(i, 2, 363);
        let size = 0.02 + 0.03 * procedural::hash(i, 3, 364);
        let tall = if i % 3 == 0 { 4.0 } else { 1.0 };
        let material = if i % 4 == 0 { p.black_cloth } else { p.antimagic };
        d.place(o, (f - size, y - size, s - size), (f + size, y + size * tall, s + size), material);
    }
    // Chispas que se desprenden de la hoja, delante de ella.
    for i in 0..10 {
        let t = 0.3 + 1.5 * procedural::hash(i, 4, 365);
        let off = (procedural::hash(i, 5, 366) - 0.5) * 0.5;
        let (s, y) = along(t);
        let (s, y) = (s - dir_y * off, y + dir_s * off);
        let size = 0.02 + 0.02 * procedural::hash(i, 6, 367);
        d.place(o, (hand.0 + 0.05, y - size, s - size), (hand.0 + 0.05 + 2.0 * size, y + size, s + size), p.antimagic);
    }
}
