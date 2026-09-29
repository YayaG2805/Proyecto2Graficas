use nalgebra_glm::Vec3;

use crate::color::Color;
use crate::cube::Cube;
use crate::ray_intersect::Material;

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
// Por ahora cada material es solo un color solido; en las fases 4-5 se
// convierten en materiales completos con textura, specular, etc.

pub struct Palette {
    pub stone: Material,
    pub wood: Material,
    pub metal: Material,
    pub glass: Material,
    pub water: Material,
    pub grass: Material,
}

impl Palette {
    fn new() -> Self {
        Palette {
            stone: Material::new(Color::new(150.0, 142.0, 130.0)),
            wood: Material::new(Color::new(130.0, 85.0, 45.0)),
            metal: Material::new(Color::new(230.0, 180.0, 70.0)),
            glass: Material::new(Color::new(150.0, 230.0, 255.0)),
            water: Material::new(Color::new(40.0, 120.0, 170.0)),
            grass: Material::new(Color::new(90.0, 150.0, 70.0)),
        }
    }
}

// Atajo para no escribir Vec3::new dos veces por caja:
// agrega una caja con esquina minima (x0,y0,z0) y maxima (x1,y1,z1).
fn add(objects: &mut Vec<Cube>, min: (f32, f32, f32), max: (f32, f32, f32), material: &Material) {
    objects.push(Cube::from_min_max(
        Vec3::new(min.0, min.1, min.2),
        Vec3::new(max.0, max.1, max.2),
        material.clone(),
    ));
}

pub fn build_scene() -> Vec<Cube> {
    let p = Palette::new();
    let mut objects = Vec::new();

    floating_island(&mut objects, &p);
    pond(&mut objects, &p);
    waterfall(&mut objects, &p);
    temple(&mut objects, &p);
    temple_details(&mut objects, &p);
    altar_and_crystal(&mut objects, &p);
    bridge_and_satellite(&mut objects, &p);
    props(&mut objects, &p);
    trees(&mut objects, &p);

    objects
}

// Isla principal: capas de roca cada vez mas pequenas hacia abajo (piramide
// invertida), con salientes y estalactitas para romper la simetria.
fn floating_island(o: &mut Vec<Cube>, p: &Palette) {
    // Capa superior de pasto + salientes irregulares del borde.
    add(o, (-7.0, -0.4, -6.0), (7.0, 0.0, 6.0), &p.grass);
    add(o, (-7.8, -0.4, -2.5), (-7.0, 0.0, 1.5), &p.grass);
    add(o, (2.0, -0.4, 6.0), (6.0, 0.0, 6.8), &p.grass);

    // Cuerpo de roca, escalonado hacia abajo.
    add(o, (-6.6, -1.5, -5.6), (6.6, -0.4, 5.6), &p.stone);
    add(o, (-5.0, -2.5, -4.5), (5.0, -1.5, 4.5), &p.stone);
    add(o, (-3.5, -3.5, -3.0), (3.0, -2.5, 3.0), &p.stone);
    add(o, (-2.0, -4.5, -1.5), (1.5, -3.5, 2.0), &p.stone);
    add(o, (-0.8, -5.6, -0.5), (0.5, -4.5, 0.8), &p.stone);

    // Estalactitas colgando bajo cada capa.
    add(o, (5.4, -2.3, 4.0), (6.0, -1.5, 4.8), &p.stone);
    add(o, (-6.0, -2.0, -3.0), (-5.4, -1.5, -2.2), &p.stone);
    add(o, (3.6, -3.3, -4.0), (4.2, -2.5, -3.4), &p.stone);
    add(o, (-4.6, -3.0, 2.8), (-4.0, -2.5, 3.4), &p.stone);
    add(o, (1.8, -4.2, 2.2), (2.4, -3.5, 2.8), &p.stone);

    // Rocas pequenas flotando alrededor (dan escala y profundidad).
    o.push(Cube::new(Vec3::new(-9.5, -0.8, 3.5), 0.8, p.stone.clone()));
    o.push(Cube::new(Vec3::new(-8.8, 1.2, -5.0), 0.5, p.stone.clone()));
    o.push(Cube::new(Vec3::new(8.2, 2.5, -5.2), 0.6, p.stone.clone()));
}

// Estanque elevado adelante a la izquierda: borde de piedra, fondo y agua,
// con un pilar en el centro (la futura fuente).
fn pond(o: &mut Vec<Cube>, p: &Palette) {
    add(o, (-5.2, 0.0, 4.4), (-1.3, 0.6, 4.7), &p.stone); // borde frente
    add(o, (-5.2, 0.0, 1.3), (-1.3, 0.6, 1.6), &p.stone); // borde fondo
    // Borde izquierdo partido en dos: por el hueco sale el canal de la cascada.
    add(o, (-5.2, 0.0, 1.6), (-4.9, 0.6, 2.7), &p.stone);
    add(o, (-5.2, 0.0, 3.3), (-4.9, 0.6, 4.4), &p.stone);
    add(o, (-1.6, 0.0, 1.6), (-1.3, 0.6, 4.4), &p.stone); // borde der
    add(o, (-4.9, 0.0, 1.6), (-1.6, 0.05, 4.4), &p.stone); // fondo
    add(o, (-4.9, 0.05, 1.6), (-1.6, 0.45, 4.4), &p.water); // agua
    add(o, (-3.45, 0.05, 2.75), (-2.95, 1.1, 3.25), &p.stone); // pilar fuente
    add(o, (-3.6, 1.1, 2.6), (-2.8, 1.25, 3.4), &p.metal); // cuenco
}

// Cascada: el estanque desborda por un canal de piedra hasta el borde de la
// isla y el agua cae al vacio, deshaciendose en gotas.
fn waterfall(o: &mut Vec<Cube>, p: &Palette) {
    // Canal con paredes de piedra.
    add(o, (-7.0, 0.0, 2.5), (-5.2, 0.5, 2.7), &p.stone);
    add(o, (-7.0, 0.0, 3.3), (-5.2, 0.5, 3.5), &p.stone);
    add(o, (-7.0, 0.0, 2.7), (-4.9, 0.4, 3.3), &p.water);

    // Caida de agua pegada al costado de la isla.
    add(o, (-7.25, -3.2, 2.7), (-7.0, 0.4, 3.3), &p.water);

    // Gotas que se separan al final de la caida.
    add(o, (-7.22, -3.8, 2.8), (-7.05, -3.5, 2.97), &p.water);
    add(o, (-7.2, -4.4, 3.02), (-7.08, -4.2, 3.14), &p.water);
    add(o, (-7.18, -4.9, 2.85), (-7.1, -4.78, 2.93), &p.water);
}

// Templo: terraza, escalinata, estilobato, columnas (una rota), dinteles y
// un frontón escalonado.
fn temple(o: &mut Vec<Cube>, p: &Palette) {
    // Terraza y estilobato (plataforma donde se apoyan las columnas).
    add(o, (-4.0, 0.0, -5.0), (4.0, 1.0, -0.5), &p.stone);
    add(o, (-3.5, 1.0, -4.5), (3.5, 1.3, -1.0), &p.stone);

    // Escalinata de 3 escalones bajando hacia el frente (+z).
    add(o, (-1.5, 0.0, -0.5), (1.5, 0.33, 0.7), &p.stone);
    add(o, (-1.5, 0.33, -0.5), (1.5, 0.66, 0.3), &p.stone);
    add(o, (-1.5, 0.66, -0.5), (1.5, 1.0, -0.1), &p.stone);

    // Columnas: (x, z, altura del fuste). La de atras a la derecha esta rota.
    let columns = [(-2.6, -1.6, 3.0), (2.6, -1.6, 3.0), (-2.6, -3.9, 3.0), (2.6, -3.9, 1.6)];
    for (x, z, height) in columns {
        column(o, p, x, z, 1.3, height, height >= 3.0);
    }

    // Dinteles (vigas horizontales sobre las columnas). El de atras solo
    // cubre la mitad porque la columna derecha se derrumbo.
    add(o, (-3.1, 4.55, -1.9), (3.1, 5.0, -1.3), &p.stone); // frente
    add(o, (-3.1, 4.55, -4.2), (0.3, 5.0, -3.6), &p.stone); // fondo (roto)
    add(o, (-2.9, 4.55, -3.6), (-2.3, 5.0, -1.9), &p.stone); // lado izq

    // Fronton escalonado sobre el dintel frontal.
    add(o, (-2.4, 5.0, -1.85), (2.4, 5.3, -1.35), &p.stone);
    add(o, (-1.4, 5.3, -1.85), (1.4, 5.6, -1.35), &p.stone);
    add(o, (-0.5, 5.6, -1.85), (0.5, 5.85, -1.35), &p.metal); // ornamento

    // Trozos caidos de la columna rota, en el pasto junto al templo.
    add(o, (4.4, 0.0, -3.2), (5.6, 0.5, -2.6), &p.stone);
    add(o, (4.9, 0.0, -4.4), (5.4, 0.5, -3.9), &p.stone);
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
        add(o, (x - 0.08, 1.0, z - 0.08), (x + 0.08, 1.8, z + 0.08), &p.metal);
        add(o, (x - 0.25, 1.8, z - 0.25), (x + 0.25, 1.95, z + 0.25), &p.metal);
    }

    // Campana colgando del dintel lateral, entre las columnas izquierdas.
    add(o, (-2.63, 4.1, -2.78), (-2.57, 4.55, -2.72), &p.wood); // cuerda
    add(o, (-2.85, 3.6, -3.0), (-2.35, 4.1, -2.5), &p.metal); // cuerpo
    add(o, (-2.92, 3.5, -3.07), (-2.28, 3.6, -2.43), &p.metal); // boca
}

// Estatua voxel: pedestal, cuerpo, hombros, cabeza y una lanza de metal.
// `side` (-1 o 1) indica hacia que lado queda la lanza.
fn guardian_statue(o: &mut Vec<Cube>, p: &Palette, x: f32, z: f32, side: f32) {
    add(o, (x - 0.4, 0.0, z - 0.4), (x + 0.4, 0.4, z + 0.4), &p.stone); // pedestal
    add(o, (x - 0.25, 0.4, z - 0.2), (x + 0.25, 1.2, z + 0.2), &p.stone); // cuerpo
    add(o, (x - 0.38, 0.85, z - 0.12), (x + 0.38, 1.1, z + 0.12), &p.stone); // hombros
    add(o, (x - 0.17, 1.2, z - 0.17), (x + 0.17, 1.52, z + 0.17), &p.stone); // cabeza

    let spear_x = x + side * 0.42;
    add(o, (spear_x - 0.04, 0.4, z - 0.04), (spear_x + 0.04, 1.9, z + 0.04), &p.metal); // asta
    add(o, (spear_x - 0.08, 1.9, z - 0.03), (spear_x + 0.08, 2.1, z + 0.03), &p.metal); // punta
}

// Una columna: basa, fuste y capitel de metal (si esta completa).
fn column(o: &mut Vec<Cube>, p: &Palette, x: f32, z: f32, floor_y: f32, height: f32, complete: bool) {
    add(o, (x - 0.4, floor_y, z - 0.4), (x + 0.4, floor_y + 0.2, z + 0.4), &p.stone);
    add(o, (x - 0.25, floor_y + 0.2, z - 0.25), (x + 0.25, floor_y + 0.2 + height, z + 0.25), &p.stone);
    if complete {
        let top = floor_y + 0.2 + height;
        add(o, (x - 0.4, top, z - 0.4), (x + 0.4, top + 0.05, z + 0.4), &p.metal);
    }
}

// Punto focal: altar de piedra con el cristal flotando encima, rodeado por
// un marco de metal.
fn altar_and_crystal(o: &mut Vec<Cube>, p: &Palette) {
    add(o, (-0.6, 1.3, -3.1), (0.6, 2.2, -2.3), &p.stone);
    add(o, (-0.7, 2.2, -3.2), (0.7, 2.3, -2.2), &p.metal);

    add(o, (-0.3, 2.6, -3.0), (0.3, 3.7, -2.4), &p.glass);

    add(o, (-0.6, 3.05, -2.25), (0.6, 3.15, -2.15), &p.metal);
    add(o, (-0.6, 3.05, -3.25), (0.6, 3.15, -3.15), &p.metal);
    add(o, (-0.6, 3.05, -3.15), (-0.5, 3.15, -2.25), &p.metal);
    add(o, (0.5, 3.05, -3.15), (0.6, 3.15, -2.25), &p.metal);
}

// Puente de tablones con barandas, hacia una isla satelite con un farol y
// un grupo de cristales.
fn bridge_and_satellite(o: &mut Vec<Cube>, p: &Palette) {
    // Tablones separados (se ve el vacio entre ellos).
    for i in 0..6 {
        let x0 = 7.0 + i as f32 * 0.5;
        add(o, (x0 + 0.05, -0.15, 0.0), (x0 + 0.45, 0.0, 1.2), &p.wood);
    }
    // Postes y barandas.
    for x in [7.05, 8.45, 9.85] {
        add(o, (x, -0.15, -0.1), (x + 0.1, 0.7, 0.0), &p.wood);
        add(o, (x, -0.15, 1.2), (x + 0.1, 0.7, 1.3), &p.wood);
    }
    add(o, (7.0, 0.6, -0.1), (10.0, 0.7, 0.0), &p.wood);
    add(o, (7.0, 0.6, 1.2), (10.0, 0.7, 1.3), &p.wood);

    // Isla satelite.
    add(o, (10.0, -0.4, -1.5), (13.0, 0.0, 2.0), &p.grass);
    add(o, (10.3, -1.4, -1.0), (12.7, -0.4, 1.5), &p.stone);
    add(o, (10.8, -2.3, -0.5), (12.2, -1.4, 1.0), &p.stone);

    // Farol de metal.
    add(o, (11.4, 0.0, -0.9), (11.6, 1.8, -0.7), &p.metal);
    add(o, (11.25, 1.8, -1.05), (11.75, 2.3, -0.55), &p.metal);

    // Cristales creciendo del suelo.
    add(o, (12.1, 0.0, 0.9), (12.45, 1.2, 1.25), &p.glass);
    add(o, (12.5, 0.0, 1.3), (12.75, 0.7, 1.55), &p.glass);
    add(o, (11.8, 0.0, 1.4), (12.0, 0.5, 1.6), &p.glass);
}

// Detalles del patio: camino de losas, cajas de madera y un andamio junto a
// la columna rota.
fn props(o: &mut Vec<Cube>, p: &Palette) {
    // Camino de losas desde la escalinata hasta el puente.
    for (x, z) in [(1.9, 1.0), (3.1, 0.7), (4.3, 0.8), (5.5, 0.55), (6.5, 0.6)] {
        add(o, (x - 0.4, 0.0, z - 0.35), (x + 0.4, 0.05, z + 0.35), &p.stone);
    }
    // Camino hacia el estanque.
    for (x, z) in [(-1.0, 1.0), (-0.6, 2.2)] {
        add(o, (x - 0.35, 0.0, z - 0.35), (x + 0.35, 0.05, z + 0.35), &p.stone);
    }

    // Cajas apiladas.
    add(o, (3.5, 0.0, 2.0), (4.3, 0.8, 2.8), &p.wood);
    add(o, (4.4, 0.0, 2.3), (5.0, 0.6, 2.9), &p.wood);
    add(o, (3.65, 0.8, 2.15), (4.15, 1.3, 2.65), &p.wood);

    // Andamio de madera junto a la columna rota.
    add(o, (3.6, 1.0, -4.6), (3.7, 3.4, -4.5), &p.wood);
    add(o, (3.6, 1.0, -3.3), (3.7, 3.4, -3.2), &p.wood);
    add(o, (3.3, 2.6, -4.7), (3.9, 2.7, -3.1), &p.wood);
}

// Arboles voxel: tronco de madera y copa de bloques escalonados.
fn trees(o: &mut Vec<Cube>, p: &Palette) {
    add(o, (5.5, 0.0, 3.8), (5.9, 2.2, 4.2), &p.wood);
    add(o, (4.7, 1.8, 3.0), (6.7, 2.8, 5.0), &p.grass);
    add(o, (5.0, 2.8, 3.3), (6.4, 3.4, 4.7), &p.grass);
    add(o, (5.3, 3.4, 3.6), (6.1, 3.8, 4.4), &p.grass);

    add(o, (-5.6, 0.0, -4.6), (-5.2, 1.8, -4.2), &p.wood);
    add(o, (-6.3, 1.5, -5.3), (-4.5, 2.4, -3.5), &p.grass);
    add(o, (-6.0, 2.4, -5.0), (-4.8, 2.9, -3.8), &p.grass);
}
