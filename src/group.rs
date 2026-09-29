use nalgebra_glm::Vec3;

use crate::cube::{slab_intersect, Cube};

// Grupo de cajas con una caja envolvente (bounding box) que las contiene a
// todas. Antes de probar el rayo contra cada pieza del grupo, se prueba
// contra la envolvente: si el rayo no la toca, ninguna pieza de adentro
// puede ser golpeada y nos saltamos el grupo completo.
pub struct Group {
    pub min: Vec3,
    pub max: Vec3,
    pub cubes: Vec<Cube>,
}

impl Group {
    // La envolvente es el minimo y maximo, eje por eje, de todas las cajas.
    pub fn new(cubes: Vec<Cube>) -> Self {
        let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        for cube in &cubes {
            min = min.inf(&cube.min);
            max = max.sup(&cube.max);
        }
        Group { min, max, cubes }
    }

    // Distancia a la que el rayo entra a la envolvente (0 si el origen ya
    // esta adentro), o None si no la toca.
    pub fn entry_distance(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<f32> {
        slab_intersect(&self.min, &self.max, ray_origin, ray_direction).map(|(tmin, _)| tmin.max(0.0))
    }
}
