use nalgebra_glm::Vec3;

use crate::color::Color;

pub struct Light {
    pub position: Vec3,
    pub intensity: f32,
    pub color: Color,
}

impl Light {
    pub fn new(position: Vec3, intensity: f32, color: Color) -> Self {
        Light { position, intensity, color }
    }
}
