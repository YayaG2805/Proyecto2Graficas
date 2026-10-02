use nalgebra_glm::Vec3;

use crate::cube::Cube;

// ============================================================
// Coordenadas locales para modelos voxel (dragones, personajes)
// ============================================================
// Las cajas de la escena estan alineadas a los ejes (no se pueden rotar),
// asi que los modelos se describen en COORDENADAS LOCALES:
//   f = adelante (hacia donde mira), y = arriba, s = lado (su derecha)
// y `place` las lleva al mundo segun hacia donde mira el modelo (una de las
// 4 direcciones horizontales) y su escala. Asi el mismo modelo sirve en
// cualquier posicion, direccion y tamano.

// Hacia donde mira el modelo: un eje horizontal del mundo.
#[derive(Clone, Copy)]
pub enum Heading {
    PlusX,
    MinusX,
    PlusZ,
    MinusZ,
}

pub struct Placement {
    origin: Vec3,
    forward: Vec3, // f local en el mundo
    side: Vec3,    // s local en el mundo
    scale: f32,
}

impl Placement {
    pub fn new(origin: Vec3, heading: Heading, scale: f32) -> Self {
        let forward = match heading {
            Heading::PlusX => Vec3::new(1.0, 0.0, 0.0),
            Heading::MinusX => Vec3::new(-1.0, 0.0, 0.0),
            Heading::PlusZ => Vec3::new(0.0, 0.0, 1.0),
            Heading::MinusZ => Vec3::new(0.0, 0.0, -1.0),
        };
        // El lado es el adelante girado 90 grados en el plano horizontal.
        let side = Vec3::new(-forward.z, 0.0, forward.x);
        Placement { origin, forward, side, scale }
    }

    // Punto local (f, y, s) -> punto del mundo.
    pub fn point(&self, f: f32, y: f32, s: f32) -> Vec3 {
        self.origin + (self.forward * f + Vec3::new(0.0, y, 0.0) + self.side * s) * self.scale
    }

    // Caja local -> caja del mundo. Como los ejes locales coinciden con ejes
    // del mundo (solo cambian de nombre o de signo), las dos esquinas
    // transformadas siguen formando una caja alineada; solo hay que volver
    // a ordenar minimo y maximo eje por eje.
    pub fn place(&self, o: &mut Vec<Cube>, min: (f32, f32, f32), max: (f32, f32, f32), material: usize) {
        let a = self.point(min.0, min.1, min.2);
        let b = self.point(max.0, max.1, max.2);
        o.push(Cube::from_min_max(a.inf(&b), a.sup(&b), material));
    }
}
