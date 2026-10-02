use nalgebra_glm::{normalize, Vec3};

use crate::bvh::Bvh;
use crate::color::Color;
use crate::cube::Cube;
use crate::dragon::{dragon, Heading};
use crate::light::Light;
use crate::procedural;
use crate::material::{Material, MaterialId};
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::skybox::Skybox;
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
    pub fire: MaterialId,
    pub bark: MaterialId,
    pub rock: MaterialId,
    pub leaves: MaterialId,
    pub blossom: MaterialId,
    pub cloth: MaterialId,
    pub scales: MaterialId,
    pub wing: MaterialId,
    pub tiles: MaterialId,
    pub flowers: MaterialId,
    pub magic: MaterialId,
    pub marble: MaterialId,
    pub obsidian: MaterialId,
    pub paper: MaterialId,
    pub cobble: MaterialId,
    pub koi: MaterialId,
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
            fire: register("fire", procedural::fire, Material::fire),
            bark: register("bark", procedural::bark, Material::bark),
            rock: register("rock", procedural::rock, Material::rock),
            leaves: register("leaves", procedural::leaves, Material::leaves),
            blossom: register("blossom", procedural::blossom, Material::blossom),
            cloth: register("cloth", procedural::cloth, Material::cloth),
            scales: register("scales", procedural::scales, Material::scales),
            wing: register("wing", procedural::wing, Material::wing),
            tiles: register("tiles", procedural::tiles, Material::tiles),
            flowers: register("flowers", procedural::flowers, Material::flowers),
            magic: register("magic", procedural::glass, Material::magic),
            marble: register("marble", procedural::marble, Material::marble),
            obsidian: register("obsidian", procedural::obsidian, Material::obsidian),
            paper: register("paper", procedural::paper, Material::paper),
            cobble: register("cobble", procedural::cobble, Material::cobble),
            koi: register("koi", procedural::koi, Material::koi),
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

// La escena completa: los materiales (cada caja los referencia por indice),
// todas las cajas organizadas en un BVH, las luces y el cielo.
pub struct Scene {
    pub materials: Vec<Material>,
    pub bvh: Bvh,
    pub lights: Vec<Light>,
    pub skybox: Skybox,
}

impl Scene {
    pub fn cube_count(&self) -> usize {
        self.bvh.cubes.len()
    }

    // Impacto mas cercano del rayo contra toda la escena. El BVH solo nos
    // entrega las cajas de las hojas que el rayo alcanza; cada impacto mas
    // cercano reduce la distancia maxima, asi los nodos que quedan mas lejos
    // ni se abren.
    pub fn closest_hit(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let mut closest: Option<&Cube> = None;

        self.bvh.traverse(ray_origin, ray_direction, f32::INFINITY, |cube, distance| {
            closest = Some(cube);
            Some(distance) // nuevo zbuffer
        });

        // Intersect completo (punto, normal, UV, material) solo para la ganadora.
        match closest {
            Some(cube) => cube.ray_intersect(ray_origin, ray_direction),
            None => Intersect::empty(),
        }
    }

    // Rayo de sombra: que fraccion de la luz llega desde el origen hasta
    // max_distance (1.0 = toda, 0.0 = nada). Cada caja en el camino deja
    // pasar solo su `transparency`: la piedra (0) bloquea todo, el cristal
    // (0.85) deja pasar casi todo y el fuego no bloquea nada. No importa cual
    // caja es la mas cercana, asi que en cuanto algo opaco bloquea la luz el
    // recorrido se detiene (mas barato que closest_hit).
    pub fn shadow_transmission(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> f32 {
        let mut transmission = 1.0;

        self.bvh.traverse(ray_origin, ray_direction, max_distance, |cube, _| {
            let material = &self.materials[cube.material];
            // El fuego no da sombra: es la fuente de luz misma (la luz del
            // brasero esta junto a su llama).
            if material.emission <= 0.0 {
                transmission *= material.transparency;
                if transmission <= 0.0 {
                    return None; // ya no llega luz: detener
                }
            }
            Some(max_distance) // se sigue buscando hasta la luz
        });

        transmission
    }
}

// Cada funcion de construccion arma una parte del diorama (asi el codigo de
// la escena queda organizado por zonas).
type PartBuilder = fn(&mut Vec<Cube>, &Palette);

pub fn build_scene() -> Scene {
    let mut materials = Vec::new();
    let p = Palette::new(&mut materials);

    let parts: [PartBuilder; 29] = [
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
        cherry_tree,
        banners,
        stone_lanterns,
        flower_beds,
        magic_orbs,
        overgrowth,
        birds,
        sky_lanterns,
        lantern_garland,
        distant_islands,
        obsidian_circle,
        pagoda_island,
        hanging_bridge,
        dragons,
        vegetation,
    ];

    // Todas las partes aportan sus cajas a una sola lista; el BVH las
    // organiza segun su posicion (no segun la parte a la que pertenecen).
    let mut cubes = Vec::new();
    for build_part in parts {
        build_part(&mut cubes, &p);
    }
    let bvh = Bvh::new(cubes);

    let lights = build_lights();
    // El disco del sol del skybox apunta hacia la primera luz (el sol).
    let skybox = Skybox { sun_direction: normalize(&lights[0].position) };

    Scene { materials, bvh, lights, skybox }
}

// Iluminacion: una luz principal calida y una de relleno fria (el contraste
// calido/frio da volumen y atractivo). Estas dos estan lejos para que su
// direccion casi no cambie de un extremo del diorama al otro, como el sol.
// Ademas, luces puntuales cercanas para el fuego, que se atenuan con la
// distancia.
fn build_lights() -> Vec<Light> {
    vec![
        // Sol del atardecer: bajo (unos 25 grados sobre el horizonte), a la
        // izquierda. Luz inclinada = sombras largas; algo mas intensa para
        // compensar que llega de lado a las caras de arriba.
        Light::new(Vec3::new(-45.0, 25.0, 31.0), 1.25, Color::new(255.0, 225.0, 180.0)),
        // Relleno: desde el lado contrario, frio y debil, como el cielo.
        Light::new(Vec3::new(25.0, 18.0, -20.0), 0.35, Color::new(150.0, 175.0, 255.0)),
        // Fuego de los braseros: luz puntual naranja junto a cada llama, con
        // alcance corto (ilumina la terraza, las estatuas y las columnas).
        Light::point(Vec3::new(-3.6, 2.3, -0.8), 1.4, Color::new(255.0, 150.0, 60.0), 4.0),
        Light::point(Vec3::new(3.6, 2.3, -0.8), 1.4, Color::new(255.0, 150.0, 60.0), 4.0),
        // Farol de la isla satelite: mas amarillo y con algo mas de alcance.
        Light::point(Vec3::new(11.5, 2.04, -0.8), 1.3, Color::new(255.0, 190.0, 100.0), 4.5),
        // Energia del cristal del altar: luz cian tenue que tine las columnas.
        Light::point(Vec3::new(0.0, 3.15, -2.0), 0.7, Color::new(120.0, 230.0, 255.0), 3.5),
        // Orbe del circulo de obsidiana: luz cian sobre el altar.
        Light::point(Vec3::new(OBSIDIAN_ISLAND.0, OBSIDIAN_ISLAND.1 + 1.3, OBSIDIAN_ISLAND.2), 0.9, Color::new(120.0, 230.0, 255.0), 3.5),
        // Linternas de piedra junto al portal (posiciones en STONE_LANTERNS).
        Light::point(lantern_light(STONE_LANTERNS[0]), 0.9, Color::new(255.0, 175.0, 90.0), 3.0),
        Light::point(lantern_light(STONE_LANTERNS[1]), 0.9, Color::new(255.0, 175.0, 90.0), 3.0),
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
    add(o, (-6.6, -1.5, -5.6), (6.6, -0.4, 5.6), p.rock);
    add(o, (-5.0, -2.5, -4.5), (5.0, -1.5, 4.5), p.rock);
    add(o, (-3.5, -3.5, -3.0), (3.0, -2.5, 3.0), p.rock);
    add(o, (-2.0, -4.5, -1.5), (1.5, -3.5, 2.0), p.rock);
    add(o, (-0.8, -5.6, -0.5), (0.5, -4.5, 0.8), p.rock);

    // Estalactitas colgando bajo cada capa.
    add(o, (5.4, -2.3, 4.0), (6.0, -1.5, 4.8), p.rock);
    add(o, (-6.0, -2.0, -3.0), (-5.4, -1.5, -2.2), p.rock);
    add(o, (3.6, -3.3, -4.0), (4.2, -2.5, -3.4), p.rock);
    add(o, (-4.6, -3.0, 2.8), (-4.0, -2.5, 3.4), p.rock);
    add(o, (1.8, -4.2, 2.2), (2.4, -3.5, 2.8), p.rock);
}

// Islotes pequenos flotando alrededor (dan escala y profundidad): roca con
// tapa de pasto y algun detalle, y dos islotes de cristales magicos.
fn floating_rocks(o: &mut Vec<Cube>, p: &Palette) {
    // Islote con arbusto (atras a la izquierda).
    add(o, (-9.05, 0.95, -5.25), (-8.55, 1.45, -4.75), p.rock);
    add(o, (-9.1, 1.45, -5.3), (-8.5, 1.55, -4.7), p.grass);
    add(o, (-8.95, 1.55, -5.15), (-8.65, 1.85, -4.85), p.leaves);

    // Islote con flores (a la derecha, junto al puente colgante).
    add(o, (7.9, 2.2, -5.5), (8.5, 2.8, -4.9), p.rock);
    add(o, (7.85, 2.8, -5.55), (8.55, 2.9, -4.85), p.grass);
    add(o, (8.0, 2.9, -5.4), (8.2, 3.0, -5.2), p.flowers);
    add(o, (8.25, 2.9, -5.15), (8.42, 3.05, -4.98), p.flowers);

    // Islotes de cristales: (x, y de la superficie, z).
    for (x, y, z) in [(-9.5, -0.4, 3.5), (4.0, -2.6, 8.6)] {
        add(o, (x - 0.45, y - 0.8, z - 0.45), (x + 0.45, y, z + 0.45), p.rock);
        add(o, (x - 0.25, y - 1.2, z - 0.25), (x + 0.2, y - 0.8, z + 0.2), p.rock);
        crystal_cluster(o, p, x, y, z);
    }
}

// Racimo de cristales sobre (x, y, z): tres cristales de alturas distintas,
// cada uno con un nucleo de energia brillante adentro que se ve a traves del
// vidrio (refraccion) como en el cristal del altar.
fn crystal_cluster(o: &mut Vec<Cube>, p: &Palette, x: f32, y: f32, z: f32) {
    let crystals = [(0.0, 0.0, 0.15, 0.9), (0.22, 0.15, 0.09, 0.55), (-0.2, 0.18, 0.08, 0.45)];
    for (dx, dz, half, height) in crystals {
        let (cx, cz) = (x + dx, z + dz);
        add(o, (cx - half, y + 0.01, cz - half), (cx + half, y + height, cz + half), p.glass);
        let core = half * 0.4;
        add(o, (cx - core, y + height * 0.3, cz - core), (cx + core, y + height * 0.6, cz + core), p.magic);
    }
}

// Estanque elevado adelante a la izquierda: borde de piedra, fondo y agua,
// con un pilar en el centro (la futura fuente).
//
// Nota para la refraccion: el agua y los cristales NO comparten caras exactas
// con sus vecinos (se separan ~0.01-0.02). Si las compartieran, el rayo
// refractado que sale por esa cara arrancaria dentro de la caja vecina.
fn pond(o: &mut Vec<Cube>, p: &Palette) {
    add(o, (-5.2, 0.0, 4.4), (-1.3, 0.6, 4.7), p.stone); // borde frente
    add(o, (-5.2, 0.0, 1.3), (-1.3, 0.6, 1.6), p.stone); // borde fondo
    // Borde izquierdo partido en dos: por el hueco sale el canal de la cascada.
    add(o, (-5.2, 0.0, 1.6), (-4.9, 0.6, 2.7), p.stone);
    add(o, (-5.2, 0.0, 3.3), (-4.9, 0.6, 4.4), p.stone);
    add(o, (-1.6, 0.0, 1.6), (-1.3, 0.6, 4.4), p.stone); // borde der
    add(o, (-4.9, 0.0, 1.6), (-1.6, 0.03, 4.4), p.stone); // fondo
    add(o, (-4.89, 0.05, 1.61), (-1.61, 0.45, 4.39), p.water); // agua
    add(o, (-3.45, 0.05, 2.75), (-2.95, 1.1, 3.25), p.stone); // pilar fuente
    add(o, (-3.6, 1.1, 2.6), (-2.8, 1.25, 3.4), p.metal); // cuenco
    fountain(o, p, -3.2, 3.0, 1.25);
    pond_life(o, p);
}

// Vida en el estanque: peces koi nadando bajo el agua (se ven a traves de
// ella, desviados por la refraccion) y nenufares flotando en la superficie,
// algunos con flor. El agua llega hasta y = 0.45.
fn pond_life(o: &mut Vec<Cube>, p: &Palette) {
    // Koi: (x, y, z, nada hacia +x o -x). Cuerpo, cabeza mas angosta, cola
    // abierta y aletas laterales.
    for (x, y, z, dir) in [(-4.1, 0.34, 2.2, 1.0), (-2.3, 0.36, 3.9, -1.0), (-3.9, 0.3, 3.75, 1.0), (-2.2, 0.33, 2.15, -1.0)] {
        let body = |a: f32, b: f32| (x + dir * a).min(x + dir * b)..(x + dir * a).max(x + dir * b);
        let r = body(-0.2, 0.2);
        add(o, (r.start, y - 0.05, z - 0.08), (r.end, y + 0.05, z + 0.08), p.koi);
        let r = body(0.2, 0.32);
        add(o, (r.start, y - 0.04, z - 0.055), (r.end, y + 0.04, z + 0.055), p.koi);
        let r = body(-0.36, -0.2);
        add(o, (r.start, y - 0.08, z - 0.02), (r.end, y + 0.08, z + 0.02), p.koi);
        let r = body(0.0, 0.1);
        add(o, (r.start, y - 0.01, z - 0.16), (r.end, y + 0.01, z + 0.16), p.koi);
    }

    // Nenufares: hojas finas sobre el agua (apenas encima, sin compartir la
    // cara con ella) y una flor en algunos.
    for (x, z, size, flower) in [(-4.45, 3.0, 0.22, true), (-1.95, 3.2, 0.18, false), (-4.5, 1.95, 0.15, false), (-2.9, 4.1, 0.2, true), (-3.4, 1.95, 0.12, false)] {
        add(o, (x - size, 0.46, z - size), (x + size, 0.475, z + size), p.leaves);
        if flower {
            add(o, (x - 0.07, 0.475, z - 0.07), (x + 0.07, 0.56, z + 0.07), p.blossom);
            add(o, (x - 0.025, 0.56, z - 0.025), (x + 0.025, 0.6, z + 0.025), p.flowers);
        }
    }
}

// Fuente sobre el cuenco en (cx, cz) a la altura `top`: agua en el cuenco,
// un chorro vertical y gotas que caen en arco (parabola) hacia los cuatro
// lados hasta el estanque.
fn fountain(o: &mut Vec<Cube>, p: &Palette, cx: f32, cz: f32, top: f32) {
    add(o, (cx - 0.33, top, cz - 0.33), (cx + 0.33, top + 0.05, cz + 0.33), p.water);
    add(o, (cx - 0.05, top + 0.05, cz - 0.05), (cx + 0.05, top + 0.85, cz + 0.05), p.water);
    add(o, (cx - 0.09, top + 0.85, cz - 0.09), (cx + 0.09, top + 0.95, cz + 0.09), p.water);

    // Gotas: t avanza por el arco; x sale hacia afuera en linea recta y la
    // altura sigue una parabola (sube un poco y luego cae).
    for (dx, dz) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        for i in 1..=5 {
            let t = i as f32 / 5.0;
            let reach = 0.15 + 0.75 * t;
            let y = top + 0.9 + 0.25 * t - 1.6 * t * t;
            let (x, z) = (cx + dx * reach, cz + dz * reach);
            let r = 0.035;
            add(o, (x - r, y - r, z - r), (x + r, y + r, z + r), p.water);
        }
    }
}

// Cascada: el estanque desborda por un canal de piedra hasta el borde de la
// isla y el agua cae al vacio, deshaciendose en gotas.
fn waterfall(o: &mut Vec<Cube>, p: &Palette) {
    // Canal con paredes de piedra.
    add(o, (-7.0, 0.0, 2.5), (-5.2, 0.5, 2.7), p.stone);
    add(o, (-7.0, 0.0, 3.3), (-5.2, 0.5, 3.5), p.stone);
    add(o, (-7.0, 0.02, 2.71), (-4.9, 0.4, 3.29), p.water);

    // Caida de agua pegada al costado de la isla.
    add(o, (-7.25, -3.2, 2.7), (-7.02, 0.4, 3.3), p.water);

    // Gotas que se separan al final de la caida.
    add(o, (-7.22, -3.8, 2.8), (-7.05, -3.5, 2.97), p.water);
    add(o, (-7.2, -4.4, 3.02), (-7.08, -4.2, 3.14), p.water);
    add(o, (-7.18, -4.9, 2.85), (-7.1, -4.78, 2.93), p.water);
}

// Templo: terraza, escalinata, estilobato, columnas (una rota), dinteles y
// un frontón escalonado.
fn temple(o: &mut Vec<Cube>, p: &Palette) {
    // Terraza y estilobato (plataforma donde se apoyan las columnas). El
    // estilobato es de marmol pulido: refleja las columnas y el cristal.
    add(o, (-4.0, 0.0, -5.0), (4.0, 1.0, -0.5), p.stone);
    add(o, (-3.5, 1.0, -4.5), (3.5, 1.3, -1.0), p.marble);

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
        // Llama: base ancha y una lengua mas delgada encima.
        add(o, (x - 0.15, 1.95, z - 0.15), (x + 0.15, 2.3, z + 0.15), p.fire);
        add(o, (x - 0.07, 2.3, z - 0.05), (x + 0.05, 2.5, z + 0.07), p.fire);
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
    // Nucleo de energia dentro del cristal: se ve a traves de el, deformado
    // por la refraccion.
    add(o, (-0.1, 3.0, -2.8), (0.1, 3.3, -2.6), p.magic);

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
    add(o, (10.3, -1.4, -1.0), (12.7, -0.4, 1.5), p.rock);
    add(o, (10.8, -2.3, -0.5), (12.2, -1.4, 1.0), p.rock);

    // Farol de metal: poste, base, nucleo encendido, 4 barrotes y techo.
    add(o, (11.4, 0.0, -0.9), (11.6, 1.8, -0.7), p.metal);
    add(o, (11.25, 1.8, -1.05), (11.75, 1.88, -0.55), p.metal);
    add(o, (11.38, 1.88, -0.92), (11.62, 2.2, -0.68), p.fire);
    for (bx, bz) in [(11.25, -1.05), (11.69, -1.05), (11.25, -0.61), (11.69, -0.61)] {
        add(o, (bx, 1.88, bz), (bx + 0.06, 2.22, bz + 0.06), p.metal);
    }
    add(o, (11.2, 2.22, -1.1), (11.8, 2.3, -0.5), p.metal);
    add(o, (11.4, 2.3, -0.9), (11.6, 2.38, -0.7), p.metal);

    // Cristales creciendo del suelo.
    add(o, (12.1, 0.01, 0.9), (12.45, 1.2, 1.25), p.glass);
    add(o, (12.5, 0.01, 1.3), (12.75, 0.7, 1.55), p.glass);
    add(o, (11.8, 0.01, 1.4), (12.0, 0.5, 1.6), p.glass);
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
    add(o, (-5.7, 4.4, -10.5), (-3.3, 5.2, -8.5), p.rock);
    add(o, (-5.0, 3.6, -10.0), (-3.8, 4.4, -9.0), p.rock);
    add(o, (-4.6, 3.0, -9.7), (-4.2, 3.6, -9.3), p.rock);

    // Obelisco: pedestal de piedra, cristal alto y remate de metal.
    add(o, (-4.9, 5.6, -9.9), (-4.1, 6.0, -9.1), p.stone);
    add(o, (-4.75, 6.02, -9.75), (-4.25, 7.48, -9.25), p.glass);
    add(o, (-4.8, 7.5, -9.8), (-4.2, 7.6, -9.2), p.metal);

    // Arbusto al lado del obelisco.
    add(o, (-3.7, 5.6, -8.9), (-3.2, 6.0, -8.4), p.leaves);
}

// Cristales creciendo hacia abajo bajo la isla principal, cada uno con su
// nucleo de energia brillante (x0, x1, y0, y1, z0, z1).
fn under_crystals(o: &mut Vec<Cube>, p: &Palette) {
    let crystals = [
        (-3.0, -2.6, -4.6, -3.51, -2.6, -2.2),
        (2.2, 2.5, -4.2, -3.51, -2.4, -2.1),
        (-1.2, -0.8, -5.2, -4.5, 2.3, 2.7),
        (0.6, 0.9, -6.4, -5.6, 0.0, 0.3),
    ];
    for (x0, x1, y0, y1, z0, z1) in crystals {
        add(o, (x0, y0, z0), (x1, y1, z1), p.glass);
        let (cx, cy, cz) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5, (z0 + z1) * 0.5);
        let r = (x1 - x0) * 0.2;
        add(o, (cx - r, cy - 2.0 * r, cz - r), (cx + r, cy + 2.0 * r, cz + r), p.magic);
    }
}

// Detalles del patio: camino de losas, cajas de madera y un andamio junto a
// la columna rota.
fn props(o: &mut Vec<Cube>, p: &Palette) {
    // Plazoleta empedrada al pie de la escalinata.
    add(o, (-1.6, 0.0, 0.7), (1.6, 0.03, 1.7), p.cobble);
    // Camino empedrado desde la escalinata hasta el puente.
    for (x, z) in [(1.9, 1.0), (3.1, 0.7), (4.3, 0.8), (5.5, 0.55), (6.5, 0.6)] {
        add(o, (x - 0.4, 0.0, z - 0.35), (x + 0.4, 0.05, z + 0.35), p.cobble);
    }
    // Camino hacia el estanque.
    for (x, z) in [(-1.0, 1.0), (-0.6, 2.2)] {
        add(o, (x - 0.35, 0.0, z - 0.35), (x + 0.35, 0.05, z + 0.35), p.cobble);
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
        add(o, min, max, p.leaves);
    }

    // Enredaderas: tiras delgadas bajo el borde del pasto.
    for (x, length) in [(-5.0, 1.4), (-2.5, 0.9), (0.5, 1.7), (3.2, 1.1)] {
        add(o, (x, -0.4 - length, 5.85), (x + 0.12, -0.4, 5.97), p.grass);
    }
    for (z, length) in [(-4.0, 1.2), (4.5, 1.6)] {
        add(o, (-6.97, -0.4 - length, z), (-6.85, -0.4, z + 0.12), p.grass);
    }
}

// Arboles voxel: tronco con raices y una rama, y copa irregular hecha con
// varios bloques de hojas que sobresalen a distintas alturas.
fn trees(o: &mut Vec<Cube>, p: &Palette) {
    // Arbol grande junto a las cajas.
    add(o, (5.5, 0.0, 3.8), (5.9, 2.2, 4.2), p.bark);
    add(o, (5.35, 0.0, 3.95), (6.05, 0.2, 4.05), p.bark); // raices
    add(o, (5.65, 0.0, 3.65), (5.75, 0.15, 4.35), p.bark);
    add(o, (5.9, 1.5, 3.9), (6.5, 1.65, 4.05), p.bark); // rama
    let canopy = [
        ((4.7, 1.9, 3.0), (6.7, 2.8, 5.0)),
        ((5.0, 2.8, 3.3), (6.4, 3.4, 4.7)),
        ((5.3, 3.4, 3.6), (6.0, 3.8, 4.3)),
        ((6.3, 1.6, 3.5), (7.0, 2.4, 4.4)),
        ((4.4, 2.2, 3.6), (4.8, 2.7, 4.5)),
        ((5.2, 2.4, 4.9), (6.1, 3.0, 5.3)),
    ];
    for (min, max) in canopy {
        add(o, min, max, p.leaves);
    }

    // Arbol detras del templo, a la izquierda.
    add(o, (-5.6, 0.0, -4.6), (-5.2, 1.8, -4.2), p.bark);
    add(o, (-5.75, 0.0, -4.45), (-5.05, 0.18, -4.35), p.bark);
    let canopy = [
        ((-6.3, 1.5, -5.3), (-4.5, 2.4, -3.5)),
        ((-6.0, 2.4, -5.0), (-4.8, 2.9, -3.8)),
        ((-5.7, 2.9, -4.7), (-5.1, 3.2, -4.1)),
        ((-6.6, 1.7, -4.6), (-6.2, 2.2, -3.9)),
        ((-5.2, 1.3, -3.7), (-4.6, 1.8, -3.2)),
    ];
    for (min, max) in canopy {
        add(o, min, max, p.leaves);
    }
}

// Cerezo en flor al frente de la isla: tronco inclinado con ramas, copa
// irregular hecha de varios bloques rosados, racimos colgando y petalos en
// el suelo y en el aire, llevados por el viento hacia +x.
fn cherry_tree(o: &mut Vec<Cube>, p: &Palette) {
    // Tronco que se inclina un poco hacia +x al subir, y ramas.
    add(o, (0.75, 0.0, 3.55), (1.1, 1.4, 3.9), p.bark);
    add(o, (0.9, 1.3, 3.5), (1.25, 2.0, 3.85), p.bark);
    add(o, (0.25, 1.8, 3.55), (0.95, 1.95, 3.7), p.bark); // rama izq
    add(o, (0.15, 1.8, 3.5), (0.35, 2.3, 3.7), p.bark);
    add(o, (1.2, 1.9, 3.6), (1.9, 2.05, 3.75), p.bark); // rama der
    add(o, (1.75, 1.9, 3.55), (1.95, 2.4, 3.75), p.bark);
    add(o, (1.0, 1.9, 2.95), (1.15, 2.05, 3.5), p.bark); // rama atras

    // Copa: un bloque central y otros que sobresalen a distintas alturas
    // para que la silueta no sea una caja.
    let canopy = [
        ((0.2, 2.0, 2.8), (1.9, 2.8, 4.5)),
        ((0.5, 2.8, 3.1), (1.6, 3.3, 4.2)),
        ((0.8, 3.3, 3.4), (1.3, 3.55, 3.9)),
        ((-0.5, 1.9, 3.1), (0.5, 2.6, 4.1)),
        ((1.6, 2.0, 3.3), (2.6, 2.7, 4.3)),
        ((0.6, 1.8, 4.3), (1.5, 2.4, 4.9)),
        ((0.4, 2.1, 2.4), (1.3, 2.6, 2.9)),
    ];
    for (min, max) in canopy {
        add(o, min, max, p.blossom);
    }

    // Racimos colgando bajo el borde de la copa.
    for (x, z, length) in [(-0.3, 3.4, 0.35), (0.1, 4.0, 0.25), (2.3, 3.6, 0.3), (1.0, 4.7, 0.3), (2.0, 4.1, 0.2)] {
        add(o, (x, 1.9 - length, z), (x + 0.15, 1.9, z + 0.15), p.blossom);
    }

    // Petalos caidos alrededor del tronco (posiciones "al azar" con el hash,
    // siempre las mismas). Muy delgados, apoyados sobre el pasto.
    for i in 0..36 {
        let angle = procedural::hash(i, 0, 121) * std::f32::consts::TAU;
        let radius = 0.4 + 2.0 * procedural::hash(i, 1, 122);
        let x = 1.0 + angle.cos() * radius + 0.5; // el viento los corre a +x
        let z = 3.7 + angle.sin() * radius;
        add(o, (x, 0.0, z), (x + 0.1, 0.012, z + 0.1), p.blossom);
    }

    // Petalos en el aire, desprendiendose de la copa hacia +x.
    for i in 0..14 {
        let x = 1.0 + 4.0 * procedural::hash(i, 2, 123);
        let y = 0.4 + 2.2 * procedural::hash(i, 3, 124) * (1.0 - (x - 1.0) / 6.0);
        let z = 2.8 + 2.0 * procedural::hash(i, 4, 125);
        add(o, (x, y, z), (x + 0.06, y + 0.06, z + 0.06), p.blossom);
    }
}

// Estandartes rojos colgando del dintel frontal del templo, con una barra de
// metal arriba y la punta cortada en "V" (dos tiras al final), y una bandera
// en un mastil sobre la isla alta.
fn banners(o: &mut Vec<Cube>, p: &Palette) {
    for x in [-1.75, 1.15] {
        let z0 = -1.29; // apenas delante de la cara del dintel (z = -1.3)
        add(o, (x - 0.05, 4.45, z0), (x + 0.65, 4.55, z0 + 0.06), p.metal); // barra
        add(o, (x, 3.35, z0), (x + 0.6, 4.45, z0 + 0.03), p.cloth);
        add(o, (x, 3.15, z0), (x + 0.22, 3.35, z0 + 0.03), p.cloth); // punta izq
        add(o, (x + 0.38, 3.15, z0), (x + 0.6, 3.35, z0 + 0.03), p.cloth); // punta der
    }

    // Mastil con bandera en la isla alta, ondeando hacia +x.
    add(o, (-3.55, 5.6, -10.5), (-3.45, 7.9, -10.4), p.wood);
    add(o, (-3.58, 7.9, -10.53), (-3.42, 8.0, -10.37), p.metal);
    add(o, (-3.45, 7.3, -10.47), (-2.9, 7.8, -10.43), p.cloth);
    add(o, (-2.9, 7.25, -10.47), (-2.4, 7.72, -10.43), p.cloth);
    add(o, (-2.4, 7.3, -10.47), (-2.05, 7.65, -10.43), p.cloth);
}

// Tres dragones rodeando la isla a distintas alturas, tamanos y poses: uno
// grande aleteando sobre el templo, otro planeando por la derecha y uno mas
// pequeno subiendo por la izquierda.
fn dragons(o: &mut Vec<Cube>, p: &Palette) {
    dragon(o, p, Vec3::new(-5.0, 8.5, -2.0), Heading::PlusX, 1.1, true);
    dragon(o, p, Vec3::new(11.5, 7.0, -5.0), Heading::MinusZ, 0.9, false);
    dragon(o, p, Vec3::new(-10.5, 2.5, 3.0), Heading::PlusZ, 0.7, true);
}

// Un piso de la pagoda centrado en (cx, cz) desde la altura y0: paredes de
// madera, pilares en las esquinas, una ventana encendida en cada cara y un
// techo de tejas en dos capas con las puntas levantadas. Devuelve la altura
// del techo, donde empieza el piso siguiente.
fn pagoda_tier(o: &mut Vec<Cube>, p: &Palette, cx: f32, cz: f32, y0: f32, half: f32, height: f32, eave: f32) -> f32 {
    let top = y0 + height;
    add(o, (cx - half, y0, cz - half), (cx + half, top, cz + half), p.wood);

    // Pilares oscuros un poco afuera de las esquinas.
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let (px, pz) = (cx + sx * half, cz + sz * half);
        add(o, (px - 0.08, y0, pz - 0.08), (px + 0.08, top, pz + 0.08), p.bark);
    }

    // Ventanas: luz calida desde adentro (fuego), una por cara.
    let (wy0, wy1) = (y0 + height * 0.35, y0 + height * 0.75);
    let w = (half * 0.35).min(0.25);
    add(o, (cx - w, wy0, cz + half), (cx + w, wy1, cz + half + 0.02), p.fire);
    add(o, (cx - w, wy0, cz - half - 0.02), (cx + w, wy1, cz - half), p.fire);
    add(o, (cx + half, wy0, cz - w), (cx + half + 0.02, wy1, cz + w), p.fire);

    // Alero de madera bajo el techo y techo de tejas en dos capas (la de
    // arriba mas chica, para dar pendiente).
    add(o, (cx - eave + 0.1, top - 0.06, cz - eave + 0.1), (cx + eave - 0.1, top, cz + eave - 0.1), p.wood);
    add(o, (cx - eave, top, cz - eave), (cx + eave, top + 0.16, cz + eave), p.tiles);
    let inner = eave - (eave - half) * 0.6;
    add(o, (cx - inner, top + 0.16, cz - inner), (cx + inner, top + 0.3, cz + inner), p.tiles);
    // Puntas levantadas en las esquinas, tipicas de las pagodas.
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let (ex, ez) = (cx + sx * (eave - 0.1), cz + sz * (eave - 0.1));
        add(o, (ex - 0.12, top + 0.16, ez - 0.12), (ex + 0.12, top + 0.32, ez + 0.12), p.tiles);
        add(o, (ex - 0.06, top + 0.32, ez - 0.06), (ex + 0.06, top + 0.42, ez + 0.06), p.metal);
    }
    top + 0.3
}

// Isla de la pagoda, detras del templo a la derecha: roca escalonada,
// plataforma de piedra, pagoda de tres pisos con aguja dorada, puerta hacia
// el puente, farolillos colgando y un pino pequeno.
fn pagoda_island(o: &mut Vec<Cube>, p: &Palette) {
    let (cx, cz) = (11.5, -6.0);

    // Isla: pasto y roca escalonada hacia abajo.
    add(o, (9.5, 0.4, -8.5), (13.5, 0.7, -3.0), p.grass);
    add(o, (9.8, -0.5, -8.2), (13.2, 0.4, -3.3), p.rock);
    add(o, (10.3, -1.6, -7.7), (12.7, -0.5, -3.9), p.rock);
    add(o, (10.9, -2.6, -7.0), (12.1, -1.6, -4.6), p.rock);
    add(o, (11.3, -3.3, -6.3), (11.8, -2.6, -5.6), p.rock);
    add(o, (12.4, -1.2, -4.2), (12.8, -0.5, -3.8), p.rock);

    // Plataforma de piedra con escalon hacia el puente (-x).
    add(o, (cx - 1.4, 0.7, cz - 1.4), (cx + 1.4, 1.0, cz + 1.4), p.stone);
    add(o, (cx - 1.8, 0.7, cz - 0.5), (cx - 1.4, 0.85, cz + 0.5), p.stone);

    // Tres pisos, cada uno mas pequeno.
    let mut y = 1.0;
    y = pagoda_tier(o, p, cx, cz, y, 0.95, 1.1, 1.55);
    y = pagoda_tier(o, p, cx, cz, y, 0.7, 0.85, 1.2);
    y = pagoda_tier(o, p, cx, cz, y, 0.45, 0.7, 0.85);

    // Aguja dorada con anillos.
    add(o, (cx - 0.07, y, cz - 0.07), (cx + 0.07, y + 1.0, cz + 0.07), p.metal);
    for (i, r) in [0.18, 0.14, 0.1].iter().enumerate() {
        let ry = y + 0.25 + i as f32 * 0.22;
        add(o, (cx - r, ry, cz - r), (cx + r, ry + 0.06, cz + r), p.metal);
    }

    // Puerta de madera oscura hacia el puente.
    add(o, (cx - 0.97, 1.0, cz - 0.3), (cx - 0.95, 1.8, cz + 0.3), p.bark);

    // Farolillos colgando de las esquinas del primer techo.
    for (sx, sz) in [(-1.0, -1.0), (-1.0, 1.0)] {
        let (lx, lz) = (cx + sx * 1.35, cz + sz * 1.35);
        add(o, (lx - 0.015, 1.75, lz - 0.015), (lx + 0.015, 2.04, lz + 0.015), p.wood);
        add(o, (lx - 0.1, 1.5, lz - 0.1), (lx + 0.1, 1.75, lz + 0.1), p.paper);
        add(o, (lx - 0.06, 1.46, lz - 0.06), (lx + 0.06, 1.5, lz + 0.06), p.wood);
    }

    // Pino pequeno en la esquina: tronco y tres pisos de hojas.
    let (tx, tz) = (13.0, -3.6);
    add(o, (tx - 0.1, 0.7, tz - 0.1), (tx + 0.1, 1.6, tz + 0.1), p.bark);
    for (i, r) in [0.45, 0.33, 0.2].iter().enumerate() {
        let ty = 1.1 + i as f32 * 0.4;
        add(o, (tx - r, ty, tz - r), (tx + r, ty + 0.4, tz + r), p.leaves);
    }
}

// Puente colgante de tablones entre la isla principal y la de la pagoda.
// Cada tablon baja segun una curva (como una cuerda que cuelga): la altura
// va de un extremo al otro en linea recta y se le resta un seno que es 0 en
// las puntas y maximo en el medio. Las cuerdas laterales siguen la misma
// curva.
fn hanging_bridge(o: &mut Vec<Cube>, p: &Palette) {
    let (x_start, x_end) = (7.0, 9.5);
    let (y_start, y_end) = (0.0, 0.7);
    let (z0, z1) = (-3.85, -3.15);
    let planks = 8;
    let step = (x_end - x_start) / planks as f32;

    for i in 0..planks {
        let t = (i as f32 + 0.5) / planks as f32;
        let y = y_start + (y_end - y_start) * t - 0.35 * (std::f32::consts::PI * t).sin();
        let x = x_start + i as f32 * step;
        add(o, (x + 0.03, y - 0.08, z0), (x + step - 0.03, y, z1), p.wood);
        // Cuerdas: un tramo por tablon, a cada lado.
        for z in [z0 - 0.04, z1] {
            add(o, (x, y + 0.5, z), (x + step, y + 0.54, z + 0.04), p.wood);
            add(o, (x + step * 0.45, y, z), (x + step * 0.55, y + 0.5, z + 0.04), p.wood);
        }
    }
    // Postes en los dos extremos.
    for (x, y) in [(x_start - 0.1, y_start), (x_end, y_end)] {
        for z in [z0 - 0.1, z1] {
            add(o, (x, y, z), (x + 0.1, y + 0.75, z + 0.1), p.bark);
        }
    }
}

// Posiciones (x, z) de las linternas de piedra, a los lados del camino que
// llega al portal. Las usan tanto la geometria como sus luces.
const STONE_LANTERNS: [(f32, f32); 2] = [(5.7, -0.3), (5.7, 2.0)];

// Altura del fuego dentro de la linterna (ver stone_lanterns).
fn lantern_light((x, z): (f32, f32)) -> Vec3 {
    Vec3::new(x, 0.95, z)
}

// Linternas de piedra estilo toro: base ancha, poste, caja de fuego abierta
// con cuatro pilarcitos, techo escalonado y una punta.
fn stone_lanterns(o: &mut Vec<Cube>, p: &Palette) {
    for (x, z) in STONE_LANTERNS {
        add(o, (x - 0.25, 0.0, z - 0.25), (x + 0.25, 0.12, z + 0.25), p.stone); // base
        add(o, (x - 0.1, 0.12, z - 0.1), (x + 0.1, 0.7, z + 0.1), p.stone); // poste
        add(o, (x - 0.2, 0.7, z - 0.2), (x + 0.2, 0.8, z + 0.2), p.stone); // plato
        // Caja de fuego: el fuego adentro y cuatro pilarcitos en las esquinas.
        add(o, (x - 0.1, 0.8, z - 0.1), (x + 0.1, 1.08, z + 0.1), p.fire);
        for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            let (px, pz) = (x + sx * 0.15, z + sz * 0.15);
            add(o, (px - 0.04, 0.8, pz - 0.04), (px + 0.04, 1.1, pz + 0.04), p.stone);
        }
        add(o, (x - 0.3, 1.1, z - 0.3), (x + 0.3, 1.2, z + 0.3), p.stone); // techo
        add(o, (x - 0.18, 1.2, z - 0.18), (x + 0.18, 1.3, z + 0.18), p.stone);
        add(o, (x - 0.06, 1.3, z - 0.06), (x + 0.06, 1.42, z + 0.06), p.stone); // punta
    }
}

// Macizos de flores: cada uno es un rectangulo (x0, z0, x1, z1) sobre el
// pasto a la altura `ground`. Se recorre en una cuadricula de 0.3 y en la
// mayoria de las celdas se pone una mata con tamano y altura "al azar"
// (hash), asi el macizo se ve natural y no como una caja.
fn flower_beds(o: &mut Vec<Cube>, p: &Palette) {
    let beds = [
        ((-4.6, 5.0, -1.6, 5.75), 0.0),   // frente del estanque
        ((-1.1, 3.0, -0.3, 4.6), 0.0),    // junto al cerezo
        ((9.7, -8.3, 10.6, -7.0), 0.7),   // isla de la pagoda
        ((12.6, -8.3, 13.35, -5.5), 0.7), // isla de la pagoda
        ((10.2, 0.2, 11.0, 1.2), 0.0),    // isla satelite
        ((-5.9, -10.7, -5.1, -10.2), 5.6), // isla alta
    ];
    for (bed, ((x0, z0, x1, z1), ground)) in beds.iter().enumerate() {
        let cols = ((x1 - x0) / 0.3) as i32;
        let rows = ((z1 - z0) / 0.3) as i32;
        for i in 0..cols {
            for j in 0..rows {
                let seed = bed as u32 * 7;
                if procedural::hash(i, j, 181 + seed) < 0.3 {
                    continue; // celda vacia
                }
                let size = 0.1 + 0.05 * procedural::hash(i, j, 182 + seed);
                let height = 0.08 + 0.16 * procedural::hash(i, j, 183 + seed);
                let x = x0 + 0.15 + i as f32 * 0.3;
                let z = z0 + 0.15 + j as f32 * 0.3;
                add(o, (x - size, *ground, z - size), (x + size, ground + height, z + size), p.flowers);
            }
        }
    }
}

// Orbes de energia flotando en espiral alrededor del cristal del altar, a
// distintas alturas y tamanos.
fn magic_orbs(o: &mut Vec<Cube>, p: &Palette) {
    let (cx, cy, cz) = (0.0, 3.15, -2.7);
    for i in 0..8 {
        let angle = i as f32 / 8.0 * std::f32::consts::TAU;
        let radius = 0.95 + 0.15 * (i % 2) as f32;
        let x = cx + angle.cos() * radius;
        let z = cz + angle.sin() * radius;
        let y = cy - 0.45 + 0.9 * (i as f32 / 8.0); // sube en espiral
        let r = if i % 3 == 0 { 0.07 } else { 0.045 };
        add(o, (x - r, y - r, z - r), (x + r, y + r, z + r), p.magic);
    }
}

// La naturaleza recuperando las ruinas: capas finas de musgo sobre las
// piedras altas y enredaderas de hojas bajando por columnas y dinteles.
fn overgrowth(o: &mut Vec<Cube>, p: &Palette) {
    // Musgo: (x0, x1, y de la superficie, z0, z1).
    let moss = [
        (-3.1, -2.4, 5.0, -1.9, -1.3),  // extremo del dintel frontal
        (2.5, 3.1, 5.0, -1.9, -1.3),
        (-3.1, -1.6, 5.0, -4.2, -3.6),  // dintel trasero roto
        (-2.9, -2.3, 5.0, -3.4, -2.2),  // dintel lateral
        (2.35, 2.85, 3.1, -4.15, -3.65), // columna rota
        (-6.8, -6.1, 1.5, -5.9, -5.5),  // muralla izquierda
        (4.5, 5.6, 1.0, -5.9, -5.5),    // muralla derecha
        (4.4, 5.1, 0.5, -3.2, -2.6),    // trozo caido
    ];
    for (x0, x1, y, z0, z1) in moss {
        add(o, (x0, y, z0), (x1, y + 0.06, z1), p.grass);
        // Un mechon que cuelga por el borde delantero.
        add(o, (x0 + 0.1, y - 0.25, z1), (x0 + 0.22, y + 0.06, z1 + 0.04), p.grass);
    }

    // Enredaderas en la cara delantera de columnas (x, z de la columna,
    // altura donde empieza, largo).
    for (x, z, top, length) in [(-2.6, -1.6, 4.5, 2.2), (2.6, -1.6, 4.5, 1.4), (-2.6, -3.9, 4.5, 2.6)] {
        let face = z + 0.25;
        add(o, (x - 0.12, top - length, face), (x - 0.04, top, face + 0.04), p.leaves);
        // Hojas sueltas a los lados del tallo.
        for k in 0..((length / 0.35) as i32) {
            let y = top - 0.2 - k as f32 * 0.35;
            let dx = if k % 2 == 0 { 0.06 } else { -0.2 };
            add(o, (x + dx - 0.06, y - 0.12, face), (x + dx + 0.08, y, face + 0.06), p.leaves);
        }
    }

    // Enredaderas colgando del dintel frontal, entre los estandartes.
    for (x, length) in [(-2.95, 0.9), (-0.35, 0.5), (0.25, 0.75), (2.85, 0.6)] {
        add(o, (x, 4.55 - length, -1.3), (x + 0.1, 4.55, -1.26), p.leaves);
        add(o, (x - 0.06, 4.55 - length - 0.12, -1.3), (x + 0.16, 4.55 - length, -1.25), p.leaves);
    }
}

// Bandada de pajaros entre la isla y el sol: a contraluz se ven como
// siluetas oscuras. Cada pajaro es un cuerpo y dos alas en forma de "M"
// (tramo interior hacia arriba, exterior hacia abajo); vuelan hacia +z.
fn birds(o: &mut Vec<Cube>, p: &Palette) {
    for i in 0..7 {
        // Formacion en "V" con un poco de desorden (hash).
        let row = (i + 1) / 2;
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        let x = -9.0 + side * row as f32 * 0.8 + 0.3 * procedural::hash(i, 0, 191);
        let y = 7.5 + 0.5 * procedural::hash(i, 1, 192) - row as f32 * 0.15;
        let z = 6.0 - row as f32 * 0.7;
        let flap = if procedural::hash(i, 2, 193) > 0.5 { 0.12 } else { 0.04 };

        add(o, (x - 0.04, y - 0.03, z - 0.14), (x + 0.04, y + 0.03, z + 0.14), p.bark); // cuerpo
        for s in [-1.0, 1.0] {
            let (a0, a1) = (x + s * 0.04, x + s * 0.24);
            let (b0, b1) = (x + s * 0.24, x + s * 0.42);
            add(o, (a0.min(a1), y, z - 0.06), (a0.max(a1), y + 0.03 + flap, z + 0.06), p.bark);
            add(o, (b0.min(b1), y + flap - 0.04, z - 0.04), (b0.max(b1), y + flap, z + 0.04), p.bark);
        }
    }
}

// Un farolillo de papel centrado en (x, y, z): cuerpo de papel encendido con
// tapas de madera arriba y abajo. `size` es la mitad del ancho.
fn paper_lantern(o: &mut Vec<Cube>, p: &Palette, x: f32, y: f32, z: f32, size: f32) {
    let h = size * 1.3;
    add(o, (x - size, y - h, z - size), (x + size, y + h, z + size), p.paper);
    let cap = size * 0.7;
    add(o, (x - cap, y + h, z - cap), (x + cap, y + h + size * 0.25, z + cap), p.wood);
    add(o, (x - cap, y - h - size * 0.25, z - cap), (x + cap, y - h, z + cap), p.wood);
}

// Farolillos voladores subiendo alrededor del santuario, como en un
// festival. Se reparten en un anillo con el hash (siempre en el mismo
// lugar): mas lejos del centro, mas altos y mas chicos, asi la "nube" de
// luces se pierde en el cielo. Brillan con luz propia, asi que de lejos se
// ven como puntos calidos contra el atardecer.
fn sky_lanterns(o: &mut Vec<Cube>, p: &Palette) {
    for i in 0..40 {
        let angle = procedural::hash(i, 0, 251) * std::f32::consts::TAU;
        let t = procedural::hash(i, 1, 252);
        let radius = 8.5 + 7.5 * t;
        let x = 1.5 + angle.cos() * radius;
        let z = -1.5 + angle.sin() * radius;
        let y = 1.5 + 8.0 * t + 3.0 * procedural::hash(i, 2, 253);
        let size = 0.14 - 0.04 * t;
        paper_lantern(o, p, x, y, z, size);
    }
}

// Guirnalda de farolillos colgando de una cuerda entre el dintel del templo
// y el portal del puente. La cuerda cuelga en curva (igual que el puente
// colgante): linea recta entre los extremos menos un seno.
fn lantern_garland(o: &mut Vec<Cube>, p: &Palette) {
    let start = Vec3::new(3.1, 4.6, -1.5);
    let end = Vec3::new(6.6, 2.35, 0.6);
    let segments = 14;
    let point = |t: f32| start + (end - start) * t - Vec3::new(0.0, 0.6 * (std::f32::consts::PI * t).sin(), 0.0);
    for i in 0..segments {
        let a = point(i as f32 / segments as f32);
        let b = point((i + 1) as f32 / segments as f32);
        let r = 0.012;
        add(o, (a.x.min(b.x) - r, a.y.min(b.y) - r, a.z.min(b.z) - r), (a.x.max(b.x) + r, a.y.max(b.y) + r, a.z.max(b.z) + r), p.wood);
    }
    // Un farolillo cada tanto, colgando un poco bajo la cuerda.
    for i in 1..5 {
        let c = point(i as f32 / 5.0);
        add(o, (c.x - 0.01, c.y - 0.12, c.z - 0.01), (c.x + 0.01, c.y, c.z + 0.01), p.wood);
        paper_lantern(o, p, c.x, c.y - 0.25, c.z, 0.09);
    }
}

// Que lleva encima cada isla lejana.
#[derive(Clone, Copy)]
enum FarDetail {
    Tree,
    Ruins,
    Waterfall,
    Crystal,
}

// Islas flotantes lejanas que rodean el santuario a 35-45 unidades (mas
// alla del zoom maximo de la camara). Llenan
// el fondo y, gracias a la bruma (apply_fog), se ven claras y "perdidas" en
// el aire, lo que da escala: el santuario es parte de un archipielago.
// (centro x, altura del pasto, centro z, mitad del ancho, detalle)
const DISTANT_ISLANDS: [(f32, f32, f32, f32, FarDetail); 9] = [
    (-30.0, 9.0, -30.0, 3.5, FarDetail::Tree),
    (-44.0, 4.0, -4.0, 3.0, FarDetail::Waterfall),
    (2.0, 12.0, -44.0, 4.0, FarDetail::Ruins),
    (-16.0, 2.0, -42.0, 2.5, FarDetail::Crystal),
    (32.0, 7.0, -26.0, 3.0, FarDetail::Tree),
    (-32.0, 13.0, 18.0, 2.5, FarDetail::Crystal),
    (16.0, 3.0, 36.0, 3.5, FarDetail::Ruins),
    (40.0, 10.0, 6.0, 2.5, FarDetail::Tree),
    (-8.0, 15.0, 38.0, 2.0, FarDetail::Waterfall),
];

fn distant_islands(o: &mut Vec<Cube>, p: &Palette) {
    for (i, &(cx, top, cz, half, detail)) in DISTANT_ISLANDS.iter().enumerate() {
        let seed = i as i32;
        let h = |k: i32| procedural::hash(seed, k, 261);

        // Pasto y roca escalonada hacia abajo (piramide invertida), cada capa
        // un poco corrida al azar para que no queden todas centradas.
        add(o, (cx - half, top - 0.4, cz - half * 0.8), (cx + half, top, cz + half * 0.8), p.grass);
        let mut width = half * 0.95;
        let mut y = top - 0.4;
        for layer in 0..4 {
            let dx = (h(layer) - 0.5) * half * 0.3;
            let dz = (h(layer + 10) - 0.5) * half * 0.3;
            let height = 0.7 + 0.6 * h(layer + 20);
            add(o, (cx + dx - width, y - height, cz + dz - width * 0.8), (cx + dx + width, y, cz + dz + width * 0.8), p.rock);
            y -= height;
            width *= 0.62;
        }

        match detail {
            FarDetail::Tree => {
                let (tx, tz) = (cx + half * 0.3, cz - half * 0.2);
                add(o, (tx - 0.2, top, tz - 0.2), (tx + 0.2, top + 1.6, tz + 0.2), p.bark);
                add(o, (tx - 1.1, top + 1.3, tz - 1.0), (tx + 1.1, top + 2.3, tz + 1.0), p.leaves);
                add(o, (tx - 0.7, top + 2.3, tz - 0.6), (tx + 0.6, top + 2.9, tz + 0.7), p.leaves);
                add(o, (cx - half * 0.6, top, cz + half * 0.2), (cx - half * 0.3, top + 0.4, cz + half * 0.5), p.leaves);
            }
            FarDetail::Ruins => {
                // Columnas de alturas distintas y un dintel sobre las dos primeras.
                let columns = [(-0.6, 2.4), (0.6, 2.4), (-0.6, 1.2), (0.6, 0.7)];
                for (k, (dx, height)) in columns.iter().enumerate() {
                    let z = if k < 2 { cz - half * 0.3 } else { cz + half * 0.3 };
                    let x = cx + dx * half;
                    add(o, (x - 0.22, top, z - 0.22), (x + 0.22, top + height, z + 0.22), p.stone);
                }
                add(o, (cx - half * 0.6 - 0.35, top + 2.4, cz - half * 0.3 - 0.3), (cx + half * 0.6 + 0.35, top + 2.75, cz - half * 0.3 + 0.3), p.stone);
            }
            FarDetail::Waterfall => {
                // Estanque en la cima y una caida por el borde.
                add(o, (cx - half * 0.5, top - 0.3, cz - half * 0.4), (cx + half * 0.3, top + 0.02, cz + half * 0.4), p.water);
                add(o, (cx + half * 0.3, top - 5.0, cz - 0.3), (cx + half * 0.3 + 0.25, top, cz + 0.3), p.water);
                add(o, (cx - half * 0.8, top, cz + half * 0.5), (cx - half * 0.5, top + 0.8, cz + half * 0.7), p.leaves);
            }
            FarDetail::Crystal => {
                add(o, (cx - 0.35, top, cz - 0.35), (cx + 0.35, top + 2.2, cz + 0.35), p.glass);
                add(o, (cx - 0.12, top + 0.7, cz - 0.12), (cx + 0.12, top + 1.4, cz + 0.12), p.magic);
                add(o, (cx + 0.6, top, cz + 0.2), (cx + 0.9, top + 1.1, cz + 0.5), p.glass);
            }
        }
    }
}

// Centro (x, altura del pasto, z) de la isla de los monolitos.
const OBSIDIAN_ISLAND: (f32, f32, f32) = (-17.0, 3.5, -7.0);

// Isla de los monolitos, a la izquierda del santuario: un circulo de piedras
// altas de obsidiana pulida (casi espejos oscuros que reflejan el cielo, las
// otras piedras y el orbe) alrededor de un altar de marmol con un orbe de
// energia flotando. Es la vitrina de la reflexion: cada monolito muestra lo
// que tiene enfrente, incluidos los reflejos de los otros.
fn obsidian_circle(o: &mut Vec<Cube>, p: &Palette) {
    let (cx, top, cz) = OBSIDIAN_ISLAND;

    // Isla: pasto, piso de marmol circular (cruz de dos losas) y roca.
    add(o, (cx - 2.6, top - 0.4, cz - 2.6), (cx + 2.6, top, cz + 2.6), p.grass);
    add(o, (cx - 1.4, top, cz - 0.6), (cx + 1.4, top + 0.05, cz + 0.6), p.marble);
    add(o, (cx - 0.6, top, cz - 1.4), (cx + 0.6, top + 0.05, cz + 1.4), p.marble);
    add(o, (cx - 2.3, top - 1.4, cz - 2.3), (cx + 2.3, top - 0.4, cz + 2.3), p.rock);
    add(o, (cx - 1.6, top - 2.4, cz - 1.4), (cx + 1.4, top - 1.4, cz + 1.6), p.rock);
    add(o, (cx - 0.8, top - 3.3, cz - 0.7), (cx + 0.6, top - 2.4, cz + 0.8), p.rock);
    add(o, (cx - 0.3, top - 3.9, cz - 0.2), (cx + 0.2, top - 3.3, cz + 0.3), p.rock);

    // Monolitos en circulo, de alturas distintas. Como las cajas no se
    // pueden rotar, se usan solo los lados del circulo que dan cajas
    // alineadas (cada monolito es mas ancho en la direccion tangente).
    let stones = [(0.0, 2.0), (60.0, 1.6), (120.0, 2.2), (180.0, 1.4), (240.0, 2.4), (300.0, 1.8)];
    for (degrees, height) in stones {
        let angle = (degrees as f32).to_radians();
        let (x, z) = (cx + angle.cos() * 1.9, cz + angle.sin() * 1.9);
        // Ancho a lo largo de la tangente del circulo: si el monolito esta
        // mas a los lados (|cos| grande), la tangente va en z.
        let (hx, hz) = if angle.cos().abs() > 0.6 { (0.12, 0.3) } else { (0.3, 0.12) };
        add(o, (x - hx - 0.06, top, z - hz - 0.06), (x + hx + 0.06, top + 0.2, z + hz + 0.06), p.stone);
        add(o, (x - hx, top + 0.2, z - hz), (x + hx, top + 0.2 + height, z + hz), p.obsidian);
    }

    // Altar de marmol con un orbe de energia flotando encima.
    add(o, (cx - 0.35, top + 0.05, cz - 0.35), (cx + 0.35, top + 0.7, cz + 0.35), p.marble);
    add(o, (cx - 0.45, top + 0.7, cz - 0.45), (cx + 0.45, top + 0.8, cz + 0.45), p.metal);
    add(o, (cx - 0.15, top + 1.15, cz - 0.15), (cx + 0.15, top + 1.45, cz + 0.15), p.magic);
}
