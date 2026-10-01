use nalgebra_glm::Vec3;

use crate::cube::{slab_intersect, Cube};

// ============================================================
// BVH (Bounding Volume Hierarchy): jerarquia de cajas envolventes
// ============================================================
// Generaliza la idea de los grupos: en vez de 15 grupos armados a mano, se
// arma automaticamente un arbol binario. La raiz envuelve toda la escena;
// cada nodo parte sus cajas en dos mitades segun su posicion (por el eje mas
// largo) y cada mitad tiene su propia envolvente, hasta llegar a hojas con
// pocas cajas. Un rayo que no toca la envolvente de un nodo descarta de un
// golpe todas las cajas que hay debajo: en vez de probar las ~190 cajas,
// prueba unas pocas envolventes y unas pocas hojas (costo ~ log n).

// Maximo de cajas en una hoja. Con menos, el arbol es mas profundo (mas
// envolventes que probar); con mas, las hojas prueban cajas de mas.
const LEAF_SIZE: usize = 4;

// Profundidad maxima de la pila al recorrer el arbol. Cada nivel parte las
// cajas a la mitad, asi que 64 alcanza de sobra.
const STACK_SIZE: usize = 64;

// Nodo guardado en un arreglo plano (sin punteros). Si `count > 0` es una
// hoja con las cajas cubes[start .. start + count]; si no, es un nodo
// interno cuyos hijos estan en nodes[start] y nodes[start + 1].
struct Node {
    min: Vec3,
    max: Vec3,
    start: usize,
    count: usize,
}

pub struct Bvh {
    nodes: Vec<Node>,
    // Las cajas quedan reordenadas para que cada hoja sea un tramo contiguo.
    pub cubes: Vec<Cube>,
}

impl Bvh {
    pub fn new(mut cubes: Vec<Cube>) -> Self {
        // La raiz va en el indice 0: se reserva y luego se llena.
        let mut nodes = vec![Node { min: Vec3::zeros(), max: Vec3::zeros(), start: 0, count: 0 }];
        let count = cubes.len();
        let root = build(&mut nodes, &mut cubes, 0, count);
        nodes[0] = root;
        Bvh { nodes, cubes }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    // Distancia a la que el rayo entra a la envolvente del nodo (0 si el
    // origen ya esta adentro), o None si no la toca.
    fn entry_distance(&self, index: usize, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<f32> {
        let node = &self.nodes[index];
        slab_intersect(&node.min, &node.max, ray_origin, ray_direction).map(|(tmin, _)| tmin.max(0.0))
    }

    // Recorre el arbol con una pila de (nodo, distancia de entrada). Solo se
    // apilan los nodos cuya envolvente toca el rayo; en las hojas se llama a
    // `visit` con cada caja golpeada. `visit` devuelve la nueva distancia
    // maxima que importa (el impacto mas cercano hasta ahora), asi los nodos
    // que quedan mas lejos se descartan sin abrirlos; si devuelve None, el
    // recorrido se detiene.
    //
    // De los dos hijos se abre primero el mas cercano: si ahi hay un impacto,
    // es probable que el hijo lejano quede detras y se descarte entero.
    pub fn traverse<'a>(
        &'a self,
        ray_origin: &Vec3,
        ray_direction: &Vec3,
        mut max_distance: f32,
        mut visit: impl FnMut(&'a Cube, f32) -> Option<f32>,
    ) {
        let mut stack = [(0usize, 0.0f32); STACK_SIZE];
        let mut top = 0;
        if let Some(entry) = self.entry_distance(0, ray_origin, ray_direction) {
            stack[0] = (0, entry);
            top = 1;
        }

        while top > 0 {
            top -= 1;
            let (index, entry) = stack[top];
            // Puede que desde que se apilo ya hayamos encontrado algo mas cerca.
            if entry >= max_distance {
                continue;
            }
            let node = &self.nodes[index];

            if node.count > 0 {
                for cube in &self.cubes[node.start..node.start + node.count] {
                    if let Some(distance) = cube.hit_distance(ray_origin, ray_direction) {
                        if distance < max_distance {
                            match visit(cube, distance) {
                                Some(new_max) => max_distance = new_max,
                                None => return,
                            }
                        }
                    }
                }
                continue;
            }

            let left = (node.start, self.entry_distance(node.start, ray_origin, ray_direction));
            let right = (node.start + 1, self.entry_distance(node.start + 1, ray_origin, ray_direction));
            // La pila saca primero lo ultimo que entra: se apila primero el
            // lejano y despues el cercano.
            let (near, far) = match (left.1, right.1) {
                (Some(l), Some(r)) if r < l => (right, left),
                _ => (left, right),
            };
            for (child, child_entry) in [far, near] {
                if let Some(child_entry) = child_entry {
                    if child_entry < max_distance {
                        stack[top] = (child, child_entry);
                        top += 1;
                    }
                }
            }
        }
    }
}

// Construye el nodo para cubes[start..end]; si tiene mas de LEAF_SIZE cajas,
// construye tambien sus hijos (recursivamente) y los guarda en `nodes`.
fn build(nodes: &mut Vec<Node>, cubes: &mut [Cube], start: usize, end: usize) -> Node {
    // Envolvente de todas las cajas del tramo.
    let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for cube in &cubes[start..end] {
        min = min.inf(&cube.min);
        max = max.sup(&cube.max);
    }

    if end - start <= LEAF_SIZE {
        return Node { min, max, start, count: end - start }; // hoja
    }

    // Eje mas largo de la envolvente: ahi conviene partir.
    let size = max - min;
    let axis = if size.x >= size.y && size.x >= size.z {
        0
    } else if size.y >= size.z {
        1
    } else {
        2
    };

    // Ordena las cajas por su centro en ese eje y parte a la mitad.
    let center = |cube: &Cube| (cube.min[axis] + cube.max[axis]) * 0.5;
    cubes[start..end].sort_by(|a, b| center(a).total_cmp(&center(b)));
    let mid = (start + end) / 2;

    // Los dos hijos van juntos en el arreglo: se reservan sus lugares y
    // luego se llenan (sus propios hijos quedan mas adelante).
    let children = nodes.len();
    nodes.push(Node { min, max, start: 0, count: 0 });
    nodes.push(Node { min, max, start: 0, count: 0 });
    let left = build(nodes, cubes, start, mid);
    nodes[children] = left;
    let right = build(nodes, cubes, mid, end);
    nodes[children + 1] = right;

    Node { min, max, start: children, count: 0 }
}
