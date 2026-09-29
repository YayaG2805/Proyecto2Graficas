use nalgebra_glm::Vec3;

use crate::ray_intersect::{Intersect, Material, RayIntersect};

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
}

impl Cube {
    pub fn new(center: Vec3, size: f32, material: Material) -> Self {
        let half = size / 2.0;
        Cube {
            min: Vec3::new(center.x - half, center.y - half, center.z - half),
            max: Vec3::new(center.x + half, center.y + half, center.z + half),
            material,
        }
    }

    // Caja de cualquier tamaño definida por sus dos esquinas opuestas.
    // Es la pieza basica del diorama: un piso, una columna o un tablon son
    // una sola caja estirada en vez de muchos cubos de 1x1.
    pub fn from_min_max(min: Vec3, max: Vec3, material: Material) -> Self {
        Cube { min, max, material }
    }
}

// Metodo slab: intersecta el rayo con los tres pares de planos alineados a
// los ejes que forman la caja. Devuelve (t_entrada, t_salida), o None si el
// rayo no toca la caja o la caja queda completamente detras del origen.
// Se usa tanto para las cajas del diorama como para las cajas envolventes de
// los grupos.
pub fn slab_intersect(min: &Vec3, max: &Vec3, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<(f32, f32)> {
    let inv_dir = Vec3::new(1.0 / ray_direction.x, 1.0 / ray_direction.y, 1.0 / ray_direction.z);

    let mut t1 = (min.x - ray_origin.x) * inv_dir.x;
    let mut t2 = (max.x - ray_origin.x) * inv_dir.x;
    let mut tmin = t1.min(t2);
    let mut tmax = t1.max(t2);

    t1 = (min.y - ray_origin.y) * inv_dir.y;
    t2 = (max.y - ray_origin.y) * inv_dir.y;
    tmin = tmin.max(t1.min(t2));
    tmax = tmax.min(t1.max(t2));

    t1 = (min.z - ray_origin.z) * inv_dir.z;
    t2 = (max.z - ray_origin.z) * inv_dir.z;
    tmin = tmin.max(t1.min(t2));
    tmax = tmax.min(t1.max(t2));

    if tmax < 0.0 || tmin > tmax {
        return None;
    }
    Some((tmin, tmax))
}

impl Cube {
    // Solo la distancia al impacto (sin normal, UV ni material). Es la
    // version barata que usamos para buscar la caja mas cercana; el
    // Intersect completo se calcula despues, una sola vez, para la ganadora.
    pub fn hit_distance(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<f32> {
        let (tmin, tmax) = slab_intersect(&self.min, &self.max, ray_origin, ray_direction)?;
        // Si el origen esta dentro de la caja, tmin es negativo: el impacto
        // visible es la salida (tmax).
        Some(if tmin > 0.0 { tmin } else { tmax })
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let distance = match self.hit_distance(ray_origin, ray_direction) {
            Some(distance) => distance,
            None => return Intersect::empty(),
        };

        let point = ray_origin + ray_direction * distance;
        let size = self.max - self.min;

        // Figure out which face was hit by checking which bound the point sits on,
        // and derive the UV coordinates from the other two axes of that face.
        let epsilon = 1e-4;
        let (normal, u, v) = if (point.x - self.min.x).abs() < epsilon {
            (Vec3::new(-1.0, 0.0, 0.0), (self.max.z - point.z) / size.z, (point.y - self.min.y) / size.y)
        } else if (point.x - self.max.x).abs() < epsilon {
            (Vec3::new(1.0, 0.0, 0.0), (point.z - self.min.z) / size.z, (point.y - self.min.y) / size.y)
        } else if (point.y - self.min.y).abs() < epsilon {
            (Vec3::new(0.0, -1.0, 0.0), (point.x - self.min.x) / size.x, (point.z - self.min.z) / size.z)
        } else if (point.y - self.max.y).abs() < epsilon {
            (Vec3::new(0.0, 1.0, 0.0), (point.x - self.min.x) / size.x, (self.max.z - point.z) / size.z)
        } else if (point.z - self.min.z).abs() < epsilon {
            (Vec3::new(0.0, 0.0, -1.0), (self.max.x - point.x) / size.x, (point.y - self.min.y) / size.y)
        } else {
            (Vec3::new(0.0, 0.0, 1.0), (point.x - self.min.x) / size.x, (point.y - self.min.y) / size.y)
        };

        Intersect::new(point, normal, distance, self.material.clone(), u, v)
    }
}
