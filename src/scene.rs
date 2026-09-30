use nalgebra_glm::Vec3;

use crate::color::Color;
use crate::cube::Cube;
use crate::group::Group;
use crate::light::Light;
use crate::procedural;
use crate::material::{Material, MaterialId};
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::texture::Texture;

// ============================================================
// SANTUARIO FLOTANTE
// ============================================================
// Convencion: Y hacia arriba, 1 unidad = 1 bloque. El pasto de la isla
// principal esta en y = 0. El templo queda al fondo (z negativo), el
// estanque adelante a la izquierda y el puente sale hacia +x, a una isla
// satelite.
//
//        (fondo, z-)
//   arbol   [ TEMPLO + altar/cristal ]   columna rota + andamio
//           [  terraza + escalinata  ]
//   ESTANQUE        camino ----------------- PUENTE ===> ISLA SATELITE
//                          cajas    arbol                 (farol, cristales)
//        (frente, z+)
//
// Los parametros de cada material (albedo, specular, reflectividad,
// transparencia, indice de refraccion) estan en material.rs.

pub struct Palette {
    pub stone: MaterialId,
    pub wood: MaterialId,
    pub metal: MaterialId,
    pub glass: MaterialId,
    pub water: MaterialId,
    pub grass: MaterialId,
}

impl Palette {
    // Registra cada material en la lista de la escena y guarda su indice.
    fn new(materials: &mut Vec<Material>) -> Self {
        // Cada material usa su propia textura, guardada como PNG en
        // assets/textures/ (se genera la primera vez que se ejecuta).
        let mut register = |name: &str, generate: fn() -> Texture, make: fn(Texture) -> Material| {
            let texture = Texture::load_or_generate(&format!("assets/textures/{}.png", name), generate);
            materials.push(make(texture));
            materials.len() - 1
        };
        Palette {
            stone: register("stone", procedural::stone, Material::stone),
            wood: register("wood", procedural::wood, Material::wood),
            metal: register("metal", procedural::metal, Material::metal),
            glass: register("glass", procedural::glass, Material::glass),
            water: register("water", procedural::water, Material::water),
            grass: register("grass", procedural::grass, Material::grass),
        }
    }
}

// Atajo para no escribir Vec3::new dos veces por caja:
// agrega una caja con esquina minima (x0,y0,z0) y maxima (x1,y1,z1).
fn add(objects: &mut Vec<Cube>, min: (f32, f32, f32), max: (f32, f32, f32), material: MaterialId) {
    objects.push(Cube::from_min_max(
        Vec3::new(min.0, min.1, min.2),
        Vec3::new(max.0, max.1, max.2),
        material,
    ));
}

// La escena completa: los materiales (cada caja los referencia por indice) y
// una lista de grupos, cada uno con su caja envolvente.
pub struct Scene {
    pub materials: Vec<Material>,
    pub groups: Vec<Group>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn cube_count(&self) -> usize {
        self.groups.iter().map(|g| g.cubes.len()).sum()
    }

    // Impacto mas cercano del rayo contra toda la escena.
    pub fn closest_hit(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let mut zbuffer = f32::INFINITY;
        let mut closest: Option<&Cube> = None;

        for group in &self.groups {
            // Si el rayo no toca la envolvente, o la toca mas lejos que el
            // impacto que ya tenemos, ninguna pieza del grupo puede ganar.
            match group.entry_distance(ray_origin, ray_direction) {
                Some(entry) if entry < zbuffer => {}
                _ => continue,
            }

            for cube in &group.cubes {
                if let Some(distance) = cube.hit_distance(ray_origin, ray_direction) {
                    if distance < zbuffer {
                        zbuffer = distance;
                        closest = Some(cube);
                    }
                }
            }
        }

        // Intersect completo (punto, normal, UV, material) solo para la ganadora.
        match closest {
            Some(cube) => cube.ray_intersect(ray_origin, ray_direction),
            None => Intersect::empty(),
        }
    }

    // Rayo de sombra: que fraccion de la luz llega desde el origen hasta
    // max_distance (1.0 = toda, 0.0 = nada). Cada caja en el camino deja
    // pasar solo su `transparency`: la piedra (0) bloquea todo, el cristal
    // (0.85) deja pasar casi todo. No importa cual caja es la mas cercana, asi
    // que en cuanto algo opaco bloquea la luz se detiene (mas barato que
    // closest_hit).
    pub fn shadow_transmission(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> f32 {
        let mut transmission = 1.0;
        for group in &self.groups {
            match group.entry_distance(ray_origin, ray_direction) {
                Some(entry) if entry < max_distance => {}
                _ => continue,
            }

            for cube in &group.cubes {
                if let Some(distance) = cube.hit_distance(ray_origin, ray_direction) {
                    if distance < max_distance {
                        transmission *= self.materials[cube.material].transparency;
                        if transmission <= 0.0 {
                            return 0.0;
                        }
                    }
                }
            }
        }
        transmission
    }
}

// Cada funcion de construccion arma una parte del diorama; cada parte se
// convierte en un grupo con su propia caja envolvente.
type PartBuilder = fn(&mut Vec<Cube>, &Palette);

pub fn build_scene() -> Scene {
    let mut materials = Vec::new();
    let p = Palette::new(&mut materials);

    let parts: [PartBuilder; 15] = [
        floating_island,
        floating_rocks,
        pond,
        waterfall,
        temple,
        temple_details,
        altar_and_crystal,
        bridge_and_satellite,
        sky_shrine,
        under_crystals,
        props,
        gate,
        ruined_walls,
        trees,
        vegetation,
    ];

    let groups = parts
        .iter()
        .map(|build_part| {
            let mut cubes = Vec::new();
            build_part(&mut cubes, &p);
            Group::new(cubes)
        })
        .collect();

    Scene { materials, groups, lights: build_lights() }
}

// Iluminacion: una luz principal calida y una de relleno fria (el contraste
// calido/frio da volumen y atractivo). Las luces estan lejos para que su
// direccion casi no cambie de un extremo del diorama al otro, como el sol.
fn build_lights() -> Vec<Light> {
    vec![
        // Sol de la tarde: arriba a la izquierda, calido y fuerte.
        Light::new(Vec3::new(-20.0, 30.0, 15.0), 1.0, Color::new(255.0, 230.0, 190.0)),
        // Relleno: desde el lado contrario, frio y debil, como el cielo.
        Light::new(Vec3::new(25.0, 18.0, -20.0), 0.35, Color::new(150.0, 175.0, 255.0)),
    ]
}

// Isla principal: capas de roca cada vez mas pequenas hacia abajo (piramide
// invertida), con salientes y estalactitas para romper la simetria.
fn floating_island(o: &mut Vec<Cube>, p: &Palette) {
    // Capa superior de pasto + salientes irregulares del borde.
    add(o, (-7.0, -0.4, -6.0), (7.0, 0.0, 6.0), p.grass);
    add(o, (-7.8, -0.4, -2.5), (-7.0, 0.0, 1.5), p.grass);
    add(o, (2.0, -0.4, 6.0), (6.0, 0.0, 6.8), p.grass);

    // Cuerpo de roca, escalonado hacia abajo.
    add(o, (-6.6, -1.5, -5.6), (6.6, -0.4, 5.6), p.stone);
    add(o, (-5.0, -2.5, -4.5), (5.0, -1.5, 4.5), p.stone);
    add(o, (-3.5, -3.5, -3.0), (3.0, -2.5, 3.0), p.stone);
    add(o, (-2.0, -4.5, -1.5), (1.5, -3.5, 2.0), p.stone);
    add(o, (-0.8, -5.6, -0.5), (0.5, -4.5, 0.8), p.stone);

    // Estalactitas colgando bajo cada capa.
    add(o, (5.4, -2.3, 4.0), (6.0, -1.5, 4.8), p.stone);
    add(o, (-6.0, -2.0, -3.0), (-5.4, -1.5, -2.2), p.stone);
    add(o, (3.6, -3.3, -4.0), (4.2, -2.5, -3.4), p.stone);
    add(o, (-4.6, -3.0, 2.8), (-4.0, -2.5, 3.4), p.stone);
    add(o, (1.8, -4.2, 2.2), (2.4, -3.5, 2.8), p.stone);
}

// Rocas pequenas flotando alrededor (dan escala y profundidad).
fn floating_rocks(o: &mut Vec<Cube>, p: &Palette) {
    o.push(Cube::new(Vec3::new(-9.5, -0.8, 3.5), 0.8, p.stone));
    o.push(Cube::new(Vec3::new(-8.8, 1.2, -5.0), 0.5, p.stone));
    o.push(Cube::new(Vec3::new(8.2, 2.5, -5.2), 0.6, p.stone));
}

// Estanque elevado adelante a la izquierda: borde de piedra, fondo y agua,
// con un pilar en el centro (la futura fuente).
fn pond(o: &mut Vec<Cube>, p: &Palette) {
    add(o, (-5.2, 0.0, 4.4), (-1.3, 0.6, 4.7), p.stone); // borde frente
    add(o, (-5.2, 0.0, 1.3), (-1.3, 0.6, 1.6), p.stone); // borde fondo
    // Borde izquierdo partido en dos: por el hueco sale el canal de la cascada.
    add(o, (-5.2, 0.0, 1.6), (-4.9, 0.6, 2.7), p.stone);
    add(o, (-5.2, 0.0, 3.3), (-4.9, 0.6, 4.4), p.stone);
    add(o, (-1.6, 0.0, 1.6), (-1.3, 0.6, 4.4), p.stone); // borde der
    add(o, (-4.9, 0.0, 1.6), (-1.6, 0.05, 4.4), p.stone); // fondo
    add(o, (-4.9, 0.05, 1.6), (-1.6, 0.45, 4.4), p.water); // agua
    add(o, (-3.45, 0.05, 2.75), (-2.95, 1.1, 3.25), p.stone); // pilar fuente
    add(o, (-3.6, 1.1, 2.6), (-2.8, 1.25, 3.4), p.metal); // cuenco
}

// Cascada: el estanque desborda por un canal de piedra hasta el borde de la
// isla y el agua cae al vacio, deshaciendose en gotas.
fn waterfall(o: &mut Vec<Cube>, p: &Palette) {
    // Canal con paredes de piedra.
    add(o, (-7.0, 0.0, 2.5), (-5.2, 0.5, 2.7), p.stone);
    add(o, (-7.0, 0.0, 3.3), (-5.2, 0.5, 3.5), p.stone);
    add(o, (-7.0, 0.0, 2.7), (-4.9, 0.4, 3.3), p.water);

    // Caida de agua pegada al costado de la isla.
    add(o, (-7.25, -3.2, 2.7), (-7.0, 0.4, 3.3), p.water);

    // Gotas que se separan al final de la caida.
    add(o, (-7.22, -3.8, 2.8), (-7.05, -3.5, 2.97), p.water);
    add(o, (-7.2, -4.4, 3.02), (-7.08, -4.2, 3.14), p.water);
    add(o, (-7.18, -4.9, 2.85), (-7.1, -4.78, 2.93), p.water);
}

// Templo: terraza, escalinata, estilobato, columnas (una rota), dinteles y
// un frontón escalonado.
fn temple(o: &mut Vec<Cube>, p: &Palette) {
    // Terraza y estilobato (plataforma donde se apoyan las columnas).
    add(o, (-4.0, 0.0, -5.0), (4.0, 1.0, -0.5), p.stone);
    add(o, (-3.5, 1.0, -4.5), (3.5, 1.3, -1.0), p.stone);

    // Escalinata de 3 escalones bajando hacia el frente (+z).
    add(o, (-1.5, 0.0, -0.5), (1.5, 0.33, 0.7), p.stone);
    add(o, (-1.5, 0.33, -0.5), (1.5, 0.66, 0.3), p.stone);
    add(o, (-1.5, 0.66, -0.5), (1.5, 1.0, -0.1), p.stone);

    // Columnas: (x, z, altura del fuste). La de atras a la derecha esta rota.
    let columns = [(-2.6, -1.6, 3.0), (2.6, -1.6, 3.0), (-2.6, -3.9, 3.0), (2.6, -3.9, 1.6)];
    for (x, z, height) in columns {
        column(o, p, x, z, 1.3, height, height >= 3.0);
    }

    // Dinteles (vigas horizontales sobre las columnas). El de atras solo
    // cubre la mitad porque la columna derecha se derrumbo.
    add(o, (-3.1, 4.55, -1.9), (3.1, 5.0, -1.3), p.stone); // frente
    add(o, (-3.1, 4.55, -4.2), (0.3, 5.0, -3.6), p.stone); // fondo (roto)
    add(o, (-2.9, 4.55, -3.6), (-2.3, 5.0, -1.9), p.stone); // lado izq

    // Fronton escalonado sobre el dintel frontal.
    add(o, (-2.4, 5.0, -1.85), (2.4, 5.3, -1.35), p.stone);
    add(o, (-1.4, 5.3, -1.85), (1.4, 5.6, -1.35), p.stone);
    add(o, (-0.5, 5.6, -1.85), (0.5, 5.85, -1.35), p.metal); // ornamento

    // Trozos caidos de la columna rota, en el pasto junto al templo.
    add(o, (4.4, 0.0, -3.2), (5.6, 0.5, -2.6), p.stone);
    add(o, (4.9, 0.0, -4.4), (5.4, 0.5, -3.9), p.stone);
}

// Detalles que dan vida al templo: estatuas guardianas a los lados de la
// escalinata, braseros en la terraza y una campana colgando del dintel.
fn temple_details(o: &mut Vec<Cube>, p: &Palette) {
    // Estatuas: la de la izquierda sostiene la lanza con la mano izquierda,
    // la de la derecha con la derecha (simetria en espejo).
    guardian_statue(o, p, -2.4, 0.1, -1.0);
    guardian_statue(o, p, 2.4, 0.1, 1.0);

    // Braseros en las esquinas delanteras de la terraza.
    for x in [-3.6, 3.6] {
        let z = -0.8;
        add(o, (x - 0.08, 1.0, z - 0.08), (x + 0.08, 1.8, z + 0.08), p.metal);
        add(o, (x - 0.25, 1.8, z - 0.25), (x + 0.25, 1.95, z + 0.25), p.metal);
    }

    // Campana colgando del dintel lateral, entre las columnas izquierdas.
    add(o, (-2.63, 4.1, -2.78), (-2.57, 4.55, -2.72), p.wood); // cuerda
    add(o, (-2.85, 3.6, -3.0), (-2.35, 4.1, -2.5), p.metal); // cuerpo
    add(o, (-2.92, 3.5, -3.07), (-2.28, 3.6, -2.43), p.metal); // boca
}

// Estatua voxel: pedestal, cuerpo, hombros, cabeza y una lanza de metal.
// `side` (-1 o 1) indica hacia que lado queda la lanza.
fn guardian_statue(o: &mut Vec<Cube>, p: &Palette, x: f32, z: f32, side: f32) {
    add(o, (x - 0.4, 0.0, z - 0.4), (x + 0.4, 0.4, z + 0.4), p.stone); // pedestal
    add(o, (x - 0.25, 0.4, z - 0.2), (x + 0.25, 1.2, z + 0.2), p.stone); // cuerpo
    add(o, (x - 0.38, 0.85, z - 0.12), (x + 0.38, 1.1, z + 0.12), p.stone); // hombros
    add(o, (x - 0.17, 1.2, z - 0.17), (x + 0.17, 1.52, z + 0.17), p.stone); // cabeza

    let spear_x = x + side * 0.42;
    add(o, (spear_x - 0.04, 0.4, z - 0.04), (spear_x + 0.04, 1.9, z + 0.04), p.metal); // asta
    add(o, (spear_x - 0.08, 1.9, z - 0.03), (spear_x + 0.08, 2.1, z + 0.03), p.metal); // punta
}

// Una columna: basa, fuste y capitel de metal (si esta completa).
fn column(o: &mut Vec<Cube>, p: &Palette, x: f32, z: f32, floor_y: f32, height: f32, complete: bool) {
    add(o, (x - 0.4, floor_y, z - 0.4), (x + 0.4, floor_y + 0.2, z + 0.4), p.stone);
    add(o, (x - 0.25, floor_y + 0.2, z - 0.25), (x + 0.25, floor_y + 0.2 + height, z + 0.25), p.stone);
    if complete {
        let top = floor_y + 0.2 + height;
        add(o, (x - 0.4, top, z - 0.4), (x + 0.4, top + 0.05, z + 0.4), p.metal);
    }
}

// Punto focal: altar de piedra con el cristal flotando encima, rodeado por
// un marco de metal.
fn altar_and_crystal(o: &mut Vec<Cube>, p: &Palette) {
    add(o, (-0.6, 1.3, -3.1), (0.6, 2.2, -2.3), p.stone);
    add(o, (-0.7, 2.2, -3.2), (0.7, 2.3, -2.2), p.metal);

    add(o, (-0.3, 2.6, -3.0), (0.3, 3.7, -2.4), p.glass);

    add(o, (-0.6, 3.05, -2.25), (0.6, 3.15, -2.15), p.metal);
    add(o, (-0.6, 3.05, -3.25), (0.6, 3.15, -3.15), p.metal);
    add(o, (-0.6, 3.05, -3.15), (-0.5, 3.15, -2.25), p.metal);
    add(o, (0.5, 3.05, -3.15), (0.6, 3.15, -2.25), p.metal);
}

// Puente de tablones con barandas, hacia una isla satelite con un farol y
// un grupo de cristales.
fn bridge_and_satellite(o: &mut Vec<Cube>, p: &Palette) {
    // Tablones separados (se ve el vacio entre ellos).
    for i in 0..6 {
        let x0 = 7.0 + i as f32 * 0.5;
        add(o, (x0 + 0.05, -0.15, 0.0), (x0 + 0.45, 0.0, 1.2), p.wood);
    }
    // Postes y barandas.
    for x in [7.05, 8.45, 9.85] {
        add(o, (x, -0.15, -0.1), (x + 0.1, 0.7, 0.0), p.wood);
        add(o, (x, -0.15, 1.2), (x + 0.1, 0.7, 1.3), p.wood);
    }
    add(o, (7.0, 0.6, -0.1), (10.0, 0.7, 0.0), p.wood);
    add(o, (7.0, 0.6, 1.2), (10.0, 0.7, 1.3), p.wood);

    // Isla satelite.
    add(o, (10.0, -0.4, -1.5), (13.0, 0.0, 2.0), p.grass);
    add(o, (10.3, -1.4, -1.0), (12.7, -0.4, 1.5), p.stone);
    add(o, (10.8, -2.3, -0.5), (12.2, -1.4, 1.0), p.stone);

    // Farol de metal.
    add(o, (11.4, 0.0, -0.9), (11.6, 1.8, -0.7), p.metal);
    add(o, (11.25, 1.8, -1.05), (11.75, 2.3, -0.55), p.metal);

    // Cristales creciendo del suelo.
    add(o, (12.1, 0.0, 0.9), (12.45, 1.2, 1.25), p.glass);
    add(o, (12.5, 0.0, 1.3), (12.75, 0.7, 1.55), p.glass);
    add(o, (11.8, 0.0, 1.4), (12.0, 0.5, 1.6), p.glass);
}

// Santuario alto: escalones de piedra flotando en espiral desde la terraza
// hasta una isla pequena mas alta, con un obelisco de cristal. Da altura a la
// composicion y un segundo punto de interes al fondo.
fn sky_shrine(o: &mut Vec<Cube>, p: &Palette) {
    // Escalones flotantes en curva: (x, y superior, z), subiendo 0.6 cada uno.
    let steps = [
        (-4.6, 1.4, -5.8),
        (-5.4, 2.0, -6.3),
        (-6.1, 2.6, -6.9),
        (-6.6, 3.2, -7.6),
        (-6.9, 3.8, -8.3),
        (-7.0, 4.4, -9.0),
        (-6.6, 5.0, -9.6),
    ];
    for (x, top, z) in steps {
        add(o, (x - 0.4, top - 0.25, z - 0.4), (x + 0.4, top, z + 0.4), p.stone);
    }

    // Isla alta detras del templo: pasto arriba y roca escalonada debajo.
    add(o, (-6.0, 5.2, -10.8), (-3.0, 5.6, -8.2), p.grass);
    add(o, (-5.7, 4.4, -10.5), (-3.3, 5.2, -8.5), p.stone);
    add(o, (-5.0, 3.6, -10.0), (-3.8, 4.4, -9.0), p.stone);
    add(o, (-4.6, 3.0, -9.7), (-4.2, 3.6, -9.3), p.stone);

    // Obelisco: pedestal de piedra, cristal alto y remate de metal.
    add(o, (-4.9, 5.6, -9.9), (-4.1, 6.0, -9.1), p.stone);
    add(o, (-4.75, 6.0, -9.75), (-4.25, 7.5, -9.25), p.glass);
    add(o, (-4.8, 7.5, -9.8), (-4.2, 7.6, -9.2), p.metal);

    // Arbusto al lado del obelisco.
    add(o, (-3.7, 5.6, -8.9), (-3.2, 6.0, -8.4), p.grass);
}

// Cristales creciendo hacia abajo bajo la isla principal.
fn under_crystals(o: &mut Vec<Cube>, p: &Palette) {
    add(o, (-3.0, -4.6, -2.6), (-2.6, -3.5, -2.2), p.glass);
    add(o, (2.2, -4.2, -2.4), (2.5, -3.5, -2.1), p.glass);
    add(o, (-1.2, -5.2, 2.3), (-0.8, -4.5, 2.7), p.glass);
    add(o, (0.6, -6.4, 0.0), (0.9, -5.6, 0.3), p.glass);
}

// Detalles del patio: camino de losas, cajas de madera y un andamio junto a
// la columna rota.
fn props(o: &mut Vec<Cube>, p: &Palette) {
    // Camino de losas desde la escalinata hasta el puente.
    for (x, z) in [(1.9, 1.0), (3.1, 0.7), (4.3, 0.8), (5.5, 0.55), (6.5, 0.6)] {
        add(o, (x - 0.4, 0.0, z - 0.35), (x + 0.4, 0.05, z + 0.35), p.stone);
    }
    // Camino hacia el estanque.
    for (x, z) in [(-1.0, 1.0), (-0.6, 2.2)] {
        add(o, (x - 0.35, 0.0, z - 0.35), (x + 0.35, 0.05, z + 0.35), p.stone);
    }

    // Cajas apiladas.
    add(o, (3.5, 0.0, 2.0), (4.3, 0.8, 2.8), p.wood);
    add(o, (4.4, 0.0, 2.3), (5.0, 0.6, 2.9), p.wood);
    add(o, (3.65, 0.8, 2.15), (4.15, 1.3, 2.65), p.wood);

    // Andamio de madera junto a la columna rota.
    add(o, (3.6, 1.0, -4.6), (3.7, 3.4, -4.5), p.wood);
    add(o, (3.6, 1.0, -3.3), (3.7, 3.4, -3.2), p.wood);
    add(o, (3.3, 2.6, -4.7), (3.9, 2.7, -3.1), p.wood);
}

// Portal de madera que marca la entrada al puente (enmarca el camino).
fn gate(o: &mut Vec<Cube>, p: &Palette) {
    add(o, (6.5, 0.0, -0.35), (6.7, 2.2, -0.15), p.wood); // poste
    add(o, (6.5, 0.0, 1.35), (6.7, 2.2, 1.55), p.wood); // poste
    add(o, (6.4, 2.2, -0.6), (6.8, 2.4, 1.8), p.wood); // viga superior
    add(o, (6.5, 1.8, -0.15), (6.7, 1.9, 1.35), p.wood); // viga inferior
    add(o, (6.47, 1.9, 0.45), (6.73, 2.2, 0.75), p.metal); // placa
}

// Restos de la muralla que rodeaba el santuario, en el borde trasero.
fn ruined_walls(o: &mut Vec<Cube>, p: &Palette) {
    add(o, (-6.8, 0.0, -5.9), (-5.0, 1.2, -5.5), p.stone);
    add(o, (-6.8, 1.2, -5.9), (-6.0, 1.5, -5.5), p.stone);
    add(o, (-4.6, 0.0, -5.9), (-4.0, 0.6, -5.5), p.stone);
    add(o, (4.5, 0.0, -5.9), (6.8, 1.0, -5.5), p.stone);
    add(o, (5.8, 1.0, -5.9), (6.8, 1.4, -5.5), p.stone);
    add(o, (6.4, 0.0, -5.5), (6.8, 0.8, -4.2), p.stone); // esquina
}

// Arbustos bajos y enredaderas colgando de los bordes de la isla.
fn vegetation(o: &mut Vec<Cube>, p: &Palette) {
    let bushes = [
        ((-6.2, 0.0, -1.0), (-5.6, 0.5, -0.4)),
        ((-6.6, 0.0, 0.2), (-6.1, 0.35, 0.7)),
        ((1.2, 0.0, 4.5), (1.9, 0.45, 5.2)),
        ((-0.5, 0.0, 5.1), (0.0, 0.3, 5.5)),
        ((10.3, 0.0, 1.3), (10.8, 0.4, 1.8)),
    ];
    for (min, max) in bushes {
        add(o, min, max, p.grass);
    }

    // Enredaderas: tiras delgadas bajo el borde del pasto.
    for (x, length) in [(-5.0, 1.4), (-2.5, 0.9), (0.5, 1.7), (3.2, 1.1)] {
        add(o, (x, -0.4 - length, 5.85), (x + 0.12, -0.4, 5.97), p.grass);
    }
    for (z, length) in [(-4.0, 1.2), (4.5, 1.6)] {
        add(o, (-6.97, -0.4 - length, z), (-6.85, -0.4, z + 0.12), p.grass);
    }
}

// Arboles voxel: tronco de madera y copa de bloques escalonados.
fn trees(o: &mut Vec<Cube>, p: &Palette) {
    add(o, (5.5, 0.0, 3.8), (5.9, 2.2, 4.2), p.wood);
    add(o, (4.7, 1.8, 3.0), (6.7, 2.8, 5.0), p.grass);
    add(o, (5.0, 2.8, 3.3), (6.4, 3.4, 4.7), p.grass);
    add(o, (5.3, 3.4, 3.6), (6.1, 3.8, 4.4), p.grass);

    add(o, (-5.6, 0.0, -4.6), (-5.2, 1.8, -4.2), p.wood);
    add(o, (-6.3, 1.5, -5.3), (-4.5, 2.4, -3.5), p.grass);
    add(o, (-6.0, 2.4, -5.0), (-4.8, 2.9, -3.8), p.grass);
}
