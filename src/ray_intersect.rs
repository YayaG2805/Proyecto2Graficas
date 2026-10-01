use nalgebra_glm::Vec3;

use crate::material::MaterialId;

#[derive(Clone)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub is_intersecting: bool,
    pub material_id: MaterialId,
    pub u: f32,
    pub v: f32,
    // Direcciones del mundo en las que crecen u y v sobre la cara golpeada.
    // Las usa el bump mapping para saber hacia donde inclinar la normal.
    pub tangent: Vec3,
    pub bitangent: Vec3,
}

impl Intersect {
    pub fn new(point: Vec3, normal: Vec3, distance: f32, material_id: MaterialId, u: f32, v: f32) -> Self {
        Intersect {
            point,
            normal,
            distance,
            is_intersecting: true,
            material_id,
            u,
            v,
            tangent: Vec3::new(0.0, 0.0, 0.0),
            bitangent: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    // Agrega las direcciones de u y v (ver `tangent`).
    pub fn with_tangents(mut self, tangent: Vec3, bitangent: Vec3) -> Self {
        self.tangent = tangent;
        self.bitangent = bitangent;
        self
    }

    pub fn empty() -> Self {
        Intersect {
            point: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 0.0),
            distance: 0.0,
            is_intersecting: false,
            material_id: 0,
            u: 0.0,
            v: 0.0,
            tangent: Vec3::new(0.0, 0.0, 0.0),
            bitangent: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect;
}
