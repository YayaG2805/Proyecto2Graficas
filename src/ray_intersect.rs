use nalgebra_glm::Vec3;

use crate::color::Color;
use crate::texture::Texture;

#[derive(Clone)]
pub struct Material {
    pub diffuse: Color,
    pub texture: Option<Texture>,
}

impl Material {
    pub fn new(diffuse: Color) -> Self {
        Material { diffuse, texture: None }
    }

    pub fn with_texture(texture: Texture) -> Self {
        Material { diffuse: Color::new(255.0, 255.0, 255.0), texture: Some(texture) }
    }

    pub fn diffuse_at(&self, u: f32, v: f32) -> Color {
        match &self.texture {
            Some(texture) => texture.sample(u, v),
            None => self.diffuse,
        }
    }
}

#[derive(Clone)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub is_intersecting: bool,
    pub material: Material,
    pub u: f32,
    pub v: f32,
}

impl Intersect {
    pub fn new(point: Vec3, normal: Vec3, distance: f32, material: Material, u: f32, v: f32) -> Self {
        Intersect {
            point,
            normal,
            distance,
            is_intersecting: true,
            material,
            u,
            v,
        }
    }

    pub fn empty() -> Self {
        Intersect {
            point: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 0.0),
            distance: 0.0,
            is_intersecting: false,
            material: Material::new(Color::black()),
            u: 0.0,
            v: 0.0,
        }
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect;
}
