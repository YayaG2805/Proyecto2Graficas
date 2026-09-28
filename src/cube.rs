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

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        // Slab method: intersect the ray against each pair of axis-aligned planes.
        let inv_dir = Vec3::new(1.0 / ray_direction.x, 1.0 / ray_direction.y, 1.0 / ray_direction.z);

        let mut t1 = (self.min.x - ray_origin.x) * inv_dir.x;
        let mut t2 = (self.max.x - ray_origin.x) * inv_dir.x;
        let mut tmin = t1.min(t2);
        let mut tmax = t1.max(t2);

        t1 = (self.min.y - ray_origin.y) * inv_dir.y;
        t2 = (self.max.y - ray_origin.y) * inv_dir.y;
        tmin = tmin.max(t1.min(t2));
        tmax = tmax.min(t1.max(t2));

        t1 = (self.min.z - ray_origin.z) * inv_dir.z;
        t2 = (self.max.z - ray_origin.z) * inv_dir.z;
        tmin = tmin.max(t1.min(t2));
        tmax = tmax.min(t1.max(t2));

        if tmax < 0.0 || tmin > tmax {
            return Intersect::empty();
        }

        let distance = if tmin > 0.0 { tmin } else { tmax };
        if distance < 0.0 {
            return Intersect::empty();
        }

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
