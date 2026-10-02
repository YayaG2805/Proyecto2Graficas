use nalgebra_glm::Vec3;

use crate::cube::Cube;
use crate::placement::{Heading, Placement};
use crate::scene::Palette;

// ============================================================
// Dragones voxel volando alrededor de la isla
// ============================================================
// Las cajas de la escena estan alineadas a los ejes (no se pueden rotar),
// asi que el dragon se describe en COORDENADAS LOCALES (ver placement.rs):
//   f = adelante (hacia donde vuela), y = arriba, s = lado (derecha)
// Asi el mismo modelo sirve para dragones que vuelan en distintas
// direcciones.

// Un dragon completo. `wings_up` elige la pose: alas levantadas (aleteando)
// o extendidas y un poco caidas (planeando).
pub fn dragon(o: &mut Vec<Cube>, p: &Palette, origin: Vec3, heading: Heading, scale: f32, wings_up: bool) {
    let d = Placement::new(origin, heading, scale);
    let body = p.scales;

    // ---- Cabeza ----
    d.place(o, (2.6, 0.0, -0.22), (3.2, 0.45, 0.22), body); // craneo
    d.place(o, (3.2, 0.02, -0.15), (3.65, 0.27, 0.15), body); // hocico
    d.place(o, (3.0, -0.14, -0.13), (3.5, 0.02, 0.13), body); // mandibula
    d.place(o, (3.55, 0.27, -0.12), (3.65, 0.32, -0.06), body); // fosas nasales
    d.place(o, (3.55, 0.27, 0.06), (3.65, 0.32, 0.12), body);
    // Ojos brillantes (fuego: emiten luz propia).
    d.place(o, (3.0, 0.27, -0.24), (3.12, 0.36, -0.21), p.fire);
    d.place(o, (3.0, 0.27, 0.21), (3.12, 0.36, 0.24), p.fire);
    // Cuernos hacia atras, de metal dorado.
    for side in [-1.0, 1.0] {
        let (s0, s1) = if side < 0.0 { (-0.2, -0.1) } else { (0.1, 0.2) };
        d.place(o, (2.65, 0.45, s0), (2.85, 0.6, s1), p.metal);
        d.place(o, (2.35, 0.55, s0), (2.65, 0.65, s1), p.metal);
        d.place(o, (2.15, 0.62, s0 * 0.9), (2.35, 0.7, s1 * 0.9), p.metal);
    }
    // Cresta de la membrana detras de la cabeza.
    d.place(o, (2.4, 0.45, -0.03), (2.7, 0.6, 0.03), p.wing);

    // ---- Cuello: baja desde la cabeza hasta el pecho ----
    d.place(o, (2.15, -0.1, -0.18), (2.7, 0.32, 0.18), body);
    d.place(o, (1.65, -0.28, -0.21), (2.2, 0.15, 0.21), body);

    // ---- Cuerpo ----
    d.place(o, (0.2, -0.5, -0.32), (1.75, 0.12, 0.32), body); // torso
    d.place(o, (0.4, -0.6, -0.22), (1.6, -0.5, 0.22), body); // vientre

    // Puas en el lomo (membrana), de mas grandes a mas chicas.
    for (i, f) in [1.9, 1.5, 1.1, 0.7, 0.3].iter().enumerate() {
        let h = 0.2 - i as f32 * 0.025;
        d.place(o, (*f, 0.12, -0.03), (*f + 0.18, 0.12 + h, 0.03), p.wing);
    }

    // Patas recogidas bajo el cuerpo, con garras oscuras.
    for side in [-1.0f32, 1.0] {
        let (s0, s1) = if side < 0.0 { (-0.32, -0.16) } else { (0.16, 0.32) };
        d.place(o, (1.3, -0.85, s0), (1.5, -0.5, s1), body); // delantera
        d.place(o, (1.35, -0.92, s0), (1.6, -0.85, s1), p.rock);
        d.place(o, (0.35, -0.9, s0), (0.65, -0.5, s1), body); // trasera
        d.place(o, (0.15, -0.98, s0), (0.45, -0.9, s1), p.rock);
    }

    // ---- Cola: baja, se adelgaza y termina curvada hacia arriba ----
    let tail = [
        ((-0.5, -0.42, -0.24), (0.25, 0.02, 0.24)),
        ((-1.2, -0.48, -0.19), (-0.45, -0.08, 0.19)),
        ((-1.9, -0.5, -0.15), (-1.15, -0.18, 0.15)),
        ((-2.5, -0.45, -0.11), (-1.85, -0.2, 0.11)),
        ((-3.0, -0.3, -0.08), (-2.45, -0.1, 0.08)),
        ((-3.4, -0.12, -0.06), (-2.95, 0.05, 0.06)),
    ];
    for (min, max) in tail {
        d.place(o, min, max, body);
    }
    // Aleta de la punta (membrana) en forma de flecha.
    d.place(o, (-3.75, -0.05, -0.04), (-3.3, 0.35, 0.04), p.wing);
    d.place(o, (-3.6, 0.35, -0.04), (-3.4, 0.45, 0.04), p.wing);

    // ---- Alas ----
    // Cada ala es una escalera de paneles delgados de membrana que suben (o
    // bajan) al alejarse del cuerpo, con huesos de escamas en el borde de
    // ataque y "dedos" que llegan al borde de salida. `lift` es cuanto sube
    // cada tramo.
    let lift = if wings_up { 0.42 } else { -0.12 };
    // (s inicial, s final, f desde donde empieza el borde de salida)
    let spans = [(0.3, 1.3, -0.3), (1.3, 2.3, -0.05), (2.3, 3.1, 0.35), (3.1, 3.6, 0.8)];
    for side in [-1.0f32, 1.0] {
        for (i, (s0, s1, trailing)) in spans.iter().enumerate() {
            let y = 0.08 + lift * i as f32;
            let (a, b) = (s0 * side, s1 * side);
            let (smin, smax) = (a.min(b), a.max(b));

            // Hueso del borde de ataque.
            d.place(o, (1.35, y, smin), (1.6, y + 0.14, smax), body);
            // Panel de membrana hacia atras.
            d.place(o, (*trailing, y + 0.02, smin), (1.35, y + 0.08, smax), p.wing);
            // Escalon que une este panel con el siguiente (sin huecos).
            if i + 1 < spans.len() {
                let edge = s1 * side;
                let next_y = 0.08 + lift * (i + 1) as f32;
                let (ylo, yhi) = (y.min(next_y), y.max(next_y) + 0.08);
                d.place(o, (*trailing, ylo, edge - 0.03), (1.6, yhi, edge + 0.03), p.wing);
            }
            // Dedo oseo que cruza la membrana hasta el borde de salida.
            let mid = (s0 + s1) * 0.5 * side;
            d.place(o, (*trailing, y + 0.08, mid - 0.04), (1.35, y + 0.12, mid + 0.04), body);
            // Festones del borde de salida: puntas que cuelgan hacia atras.
            d.place(o, (trailing - 0.25, y + 0.02, mid - 0.15), (*trailing, y + 0.08, mid + 0.15), p.wing);
        }
        // Garra en la punta del ala.
        let tip_y = 0.08 + lift * (spans.len() - 1) as f32;
        let tip = 3.6 * side;
        d.place(o, (1.5, tip_y, tip.min(tip + 0.25 * side)), (1.85, tip_y + 0.12, tip.max(tip + 0.25 * side)), p.metal);
    }
}
