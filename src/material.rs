use crate::color::Color;
use crate::texture::Texture;

// Indice de un material dentro de Scene::materials. Las cajas guardan solo
// este numero en vez de una copia del material completo.
pub type MaterialId = usize;

// Todas las propiedades que definen como una superficie interactua con la luz.
//
// La luz que llega a la superficie se reparte en tres partes:
//   reflectivity           -> se refleja como espejo (rayo reflejado)
//   transparency           -> atraviesa el material (rayo refractado)
//   1 - reflectivity - transparency -> se ve con el color propio (difuso)
// Por eso reflectivity + transparency debe ser <= 1.
#[derive(Clone)]
pub struct Material {
    pub name: &'static str,
    // Detalle de la superficie (se muestrea con la UV del impacto).
    pub texture: Texture,
    // Color base del material (0-255 por canal): tine la textura.
    pub albedo: Color,
    // Intensidad del brillo especular (Phong), de 0 a 1.
    pub specular: f32,
    // Exponente de Phong: mas alto = brillo mas pequeno y concentrado.
    pub shininess: f32,
    // Fraccion de luz reflejada como espejo, de 0 a 1.
    pub reflectivity: f32,
    // Fraccion de luz que atraviesa el material, de 0 a 1.
    pub transparency: f32,
    // Indice de refraccion n (aire = 1.0). Solo importa si transparency > 0.
    pub refractive_index: f32,
}

impl Material {
    // Color propio de la superficie en (u, v): textura tenida por el albedo.
    // (Color * Color divide entre 255, asi que un albedo blanco no cambia nada.)
    pub fn base_color(&self, u: f32, v: f32) -> Color {
        self.texture.sample(u, v) * self.albedo
    }

    // ---- Los 5 materiales de la rubrica ----

    // Piedra: estructura del templo e isla. Mate: casi sin brillo ni reflejo.
    pub fn stone(texture: Texture) -> Self {
        Material {
            name: "Piedra",
            texture,
            albedo: Color::new(255.0, 248.0, 235.0),
            specular: 0.1,
            shininess: 8.0,
            reflectivity: 0.02,
            transparency: 0.0,
            refractive_index: 1.0,
        }
    }

    // Madera: puente, portal, cajas. Barniz leve: brillo suave.
    pub fn wood(texture: Texture) -> Self {
        Material {
            name: "Madera",
            texture,
            albedo: Color::new(255.0, 230.0, 205.0),
            specular: 0.25,
            shininess: 16.0,
            reflectivity: 0.05,
            transparency: 0.0,
            refractive_index: 1.0,
        }
    }

    // Metal (oro): capiteles, campana, braseros. Brillo intenso y reflejo alto.
    pub fn metal(texture: Texture) -> Self {
        Material {
            name: "Metal",
            texture,
            albedo: Color::new(255.0, 235.0, 190.0),
            specular: 0.9,
            shininess: 128.0,
            reflectivity: 0.65,
            transparency: 0.0,
            refractive_index: 1.0,
        }
    }

    // Cristal: la reliquia del altar y los cristales. Casi transparente,
    // refracta con n = 1.5 (vidrio) y refleja un poco.
    pub fn glass(texture: Texture) -> Self {
        Material {
            name: "Cristal",
            texture,
            albedo: Color::new(225.0, 245.0, 255.0),
            specular: 1.0,
            shininess: 256.0,
            reflectivity: 0.1,
            transparency: 0.85,
            refractive_index: 1.5,
        }
    }

    // Agua: estanque y cascada. Medio transparente, refracta con n = 1.33 y
    // refleja el entorno.
    pub fn water(texture: Texture) -> Self {
        Material {
            name: "Agua",
            texture,
            albedo: Color::new(170.0, 215.0, 255.0),
            specular: 0.7,
            shininess: 64.0,
            reflectivity: 0.3,
            transparency: 0.5,
            refractive_index: 1.33,
        }
    }

    // ---- Material decorativo extra (no cuenta para la rubrica) ----

    // Pasto y hojas: mate, sin reflejo.
    pub fn grass(texture: Texture) -> Self {
        Material {
            name: "Pasto",
            texture,
            albedo: Color::new(240.0, 255.0, 230.0),
            specular: 0.05,
            shininess: 4.0,
            reflectivity: 0.0,
            transparency: 0.0,
            refractive_index: 1.0,
        }
    }
}
