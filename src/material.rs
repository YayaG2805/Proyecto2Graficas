use crate::color::Color;
use crate::texture::Texture;

// Indice de un material dentro de Scene::materials. Las cajas guardan solo
// este numero en vez de una copia del material completo.
pub type MaterialId = usize;

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
