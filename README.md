# Proyecto 2 — Raytracing: Santuario flotante

Raytracer en Rust que renderiza en tiempo real un diorama voxel: un santuario
en ruinas sobre una isla flotante, al atardecer, sobre un mar de nubes, con
una pagoda en una isla vecina, un cerezo en flor, tres dragones volando
alrededor, un dirigible, farolillos de papel subiendo al cielo, un
archipiélago de islas lejanas y dos invitados: All Might gigante detrás del
templo y Asta con la espada mata demonios. Todo se calcula en el CPU, sin
GPU ni shaders: 1447 cajas, 35 materiales y 10 luces.

![Vista general del santuario](docs/vista_general.png)

## Video

**Video de demostración:** _(pendiente: agregar el enlace)_

## Cómo ejecutarlo

Requiere Rust (edición 2024) y las herramientas de compilación de raylib
(CMake y un compilador de C).

```bash
cargo run
```

El perfil `dev` ya compila con `opt-level = 3`, así que `cargo run` corre a
velocidad completa. Al iniciar se imprime la tabla de materiales y el número
de cajas de la escena.

Modos sin ventana:

```bash
# Captura de un cuadro: archivo, yaw, pitch, distancia y (opcional) el punto que mira la cámara
cargo run -- --screenshot captura.png 0.6 0.35 22 1.5 1.0 -1.5

# Benchmark: renderiza cada vista predefinida 10 veces e imprime el promedio
cargo run -- --bench
```

### Controles

| Tecla | Acción |
|---|---|
| `A` / `D`, flechas o arrastrar con el mouse | Rotar alrededor del diorama |
| `W` / `S` | Inclinar la cámara |
| `Q` / `E` o rueda del mouse | Zoom |
| `1` – `9` | Vistas predefinidas (transición suave) |
| `R` | Reiniciar la cámara |
| `Espacio` | Giro automático |
| `P` | Activa o desactiva la vista previa a media resolución mientras la cámara se mueve (desactivada, todo se renderiza a resolución completa: útil para grabar) |
| `F11` | Pantalla completa: la imagen se escala para llenar el monitor sin deformarse (sigue renderizándose a 800×600) |
| `F12` | Guardar captura en `screenshots/` |
| `H` | Mostrar u ocultar la ayuda |

## Requisitos de la rúbrica

| Requisito | Dónde se ve | Implementación |
|---|---|---|
| Complejidad de la escena | 1447 cajas: templo, pagoda, cerezo, fuente con koi, puentes, dragones, dirigible, monolitos, islas lejanas, All Might y Asta | `scene.rs` (una función por zona), `dragon.rs`, `characters.rs` |
| Atractivo visual | Atardecer, luz cálida y fría, fuego, magia, farolillos, bruma, relieve en las texturas, antialiasing | `scene.rs: build_lights`, `skybox.rs`, `postprocess.rs`, `renderer.rs: bump_normal` |
| Rotación y zoom de la cámara | Teclado, mouse y vistas 1–9 | `camera.rs: OrbitCamera` |
| 5 materiales con textura, albedo, specular, transparencia y reflectividad | Piedra, madera, metal, cristal y agua | `material.rs`, `procedural.rs` |
| Refracción | Cristal del altar, obelisco y agua del estanque | `renderer.rs: refract` (ley de Snell) |
| Reflexión | Metal dorado, piso de mármol pulido, monolitos de obsidiana, tejas, agua y cristal; más reflejo al mirar de lado (Fresnel) | `renderer.rs: reflect` y rayos recursivos |
| Skybox | Cielo de atardecer con sol, luna, nubes altas sombreadas, mar de nubes con relieve, montañas y estrellas | `skybox.rs` |

## Materiales

Cada material tiene su propia textura procedural de 32×32 px, generada por
código en `procedural.rs` y guardada en `assets/textures/`, más los
parámetros que definen cómo responde a la luz:

| Material | Albedo (RGB) | Specular | Shininess | Reflectividad | Transparencia | IOR | Fresnel | Relieve |
|---|---|---|---|---|---|---|---|---|
| **Piedra** | 255, 248, 235 | 0.10 | 8 | 0.02 | 0.00 | 1.00 | 0.00 | 0.060 |
| **Madera** | 255, 230, 205 | 0.25 | 16 | 0.05 | 0.00 | 1.00 | 0.15 | 0.024 |
| **Metal** (oro) | 255, 235, 190 | 0.90 | 128 | 0.65 | 0.00 | 1.00 | 1.00 | 0.016 |
| **Cristal** | 225, 245, 255 | 1.00 | 256 | 0.10 | 0.85 | 1.50 | 0.00 | 0.000 |
| **Agua** | 195, 230, 255 | 0.70 | 64 | 0.25 | 0.60 | 1.33 | 0.00 | 0.000 |

- **Fresnel:** cuánto aumenta el reflejo de una superficie opaca al mirarla
  de lado (el cristal y el agua ya lo calculan con Schlick sobre su parte
  transparente).
- **Relieve:** profundidad del *bump mapping* en unidades de mundo.

Además hay 30 materiales decorativos (no cuentan para la rúbrica). Los de
la escena tienen su propia textura procedural:

| Material | Dónde | Detalle |
|---|---|---|
| Pasto, Hojas, Flores | Suelo, copas, macizos | Mates |
| Corteza, Roca | Troncos, base de las islas | Parámetros de madera y piedra, otra textura |
| Cerezo | Copa del cerezo y pétalos | Rosado con pétalos claros |
| Tela | Estandartes y bandera | Rojo con franjas y rombos dorados |
| Tejas | Techos de la pagoda | Cerámica vidriada: specular 0.6, reflectividad 0.12 |
| Escamas | Cuerpo de los dragones | Marfil nacarado: specular 0.5, reflectividad 0.08 |
| Membrana | Alas de los dragones | Translúcida (transparencia 0.35, n = 1.0): brilla a contraluz |
| Fuego | Braseros, linternas, ventanas, ojos de dragón | Emisivo: brilla con luz propia y no da sombra |
| Magia | Núcleos de cristal y orbes | Emisivo cian |
| Mármol | Piso del templo, altar de los monolitos | Pulido: reflectividad 0.18 y Fresnel 0.8, vetas con ruido fractal periódico |
| Obsidiana | Monolitos | Casi un espejo oscuro: reflectividad 0.5, Fresnel 1.0 |
| Papel | Farolillos | Emisivo cálido con varillas de bambú |
| Empedrado | Caminos y plazoleta | Diagrama de Voronoi, con relieve marcado entre las piedras |
| Koi | Peces del estanque | Escamas naranjas con manchas blancas |
| Luciérnaga | Nubes de luciérnagas | Emisivo verde-amarillo |

Y los de los personajes:

| Material | Dónde | Detalle |
|---|---|---|
| Piel | All Might y Asta | Mate con brillo ancho y un relieve muy leve |
| Cabello dorado, Cabello blanco | Mechones de All Might, picos de Asta | Mechones verticales con brillo en bandas y relieve entre mechones |
| Traje azul, rojo, blanco y amarillo | Traje de All Might, dientes | Spandex satinado. Comparten una textura casi blanca y cada uno le da su color con el albedo |
| Tela negra | Ropa, capa y bandana de Asta, cuencas de los ojos de All Might | Misma textura del traje, albedo oscuro |
| Cuero | Botas y cinturón de Asta | Arrugas, poros y costura punteada, con relieve |
| Hierro demoníaco | Espada mata demonios | Metal negro con grietas rojas que **brillan** (`glow` 0.9), relieve 0.05 |
| Vendas | Mango de la espada | Tiras enrolladas en diagonal |
| Antimagia | Aura, brazo y ojo de Asta | Emisivo rojo sobre la textura de la obsidiana |

La luz que llega a una superficie se reparte así: `reflectividad` se refleja
como espejo, `transparencia` atraviesa el material y el resto se ve con el
color propio (textura × albedo, iluminada con Lambert y Phong). Las texturas
se repiten sin costuras: el ruido de las nuevas (mármol, obsidiana, koi) es
periódico, así que el borde derecho de la textura empata con el izquierdo.

## Galería

| | |
|---|---|
| ![Altar y cristal](docs/altar_cristal.png) **Altar y cristal:** refracción en el cristal y reflejo en el marco de metal | ![Estanque](docs/estanque.png) **Estanque:** el agua refracta el fondo y refleja el cielo según el ángulo (Fresnel) |
| ![Contraluz](docs/contraluz.png) **Contraluz:** el sol del skybox detrás del templo | ![Obelisco](docs/obelisco.png) **Obelisco:** cristal alto sobre la isla del santuario |
| ![Braseros](docs/braseros.png) **Braseros:** llamas emisivas con luz puntual cálida | ![Farol](docs/farol.png) **Farol:** núcleo de fuego que ilumina su isla |
| ![Desde abajo](docs/desde_abajo.png) **Desde abajo:** roca natural iluminada por el rebote de las nubes | ![Dragón](docs/dragon.png) **Dragón:** escamas de marfil, alas de membrana translúcida y ojos de fuego |
| ![Pagoda](docs/pagoda.png) **Pagoda:** tejas vidriadas, ventanas encendidas y puente colgante | ![Cerezo](docs/cerezo.png) **Cerezo:** copa irregular y pétalos en el suelo y en el aire |
| ![Cristales](docs/cristales.png) **Islote de cristales:** núcleos mágicos vistos a través del vidrio | ![Linternas](docs/linternas.png) **Linternas de piedra:** luz puntual cálida junto al portal |
| ![Mármol](docs/marmol.png) **Piso de mármol:** refleja las columnas, el cristal y el cielo, más fuerte al mirarlo de lado (Fresnel) | ![Monolitos](docs/monolitos.png) **Monolitos de obsidiana:** espejos oscuros que se reflejan entre sí |
| ![Dirigible](docs/dirigible.png) **Dirigible:** casco de madera, globo de tela y hélice de metal, sobre la isla de los monolitos | ![All Might](docs/all_might.png) **All Might:** gigante de 17 unidades con el puño en alto |
| ![Asta](docs/asta.png) **Asta:** espada mata demonios con grietas que brillan, aura de antimagia y luz roja | |

## Personajes invitados

`characters.rs` arma a los dos personajes con cajas, en coordenadas locales
(adelante, arriba, su derecha) igual que los dragones (`placement.rs`), así
que se pueden mover, girar en 4 direcciones y escalar. Están basados en
imágenes de referencia del anime (son *fan art* sin fines comerciales; los
personajes pertenecen a sus autores).

- **All Might** (*My Hero Academia*): a escala 1.75 mide unas 17 unidades
  y se alza detrás del templo, sobre su propia isla con el pasto agrietado.
  Pose de victoria: el puño izquierdo al cielo y el brazo derecho relajado.
  Traje azul marino con los paneles rojos del pecho y el abdomen, blanco
  junto al cuello y en los hombros, cinturón amarillo con hebilla de metal,
  franjas blancas y rojas a los lados de las piernas y botas rojas. La cara
  queda en sombra con los ojos brillando (material de magia), la sonrisa de
  dientes blancos y los dos mechones en V hechos de escalones que se abren.
- **Asta** (*Black Clover*): en el patio frente a la escalinata del
  templo, entre el cerezo y las cajas, a escala 1.5 y en su forma de
  antimagia. El lugar se eligió para que quede al centro de la vista
  general sin tapar el cuadro de la vista del altar, y mira hacia el puente
  para que la espada apunte al pasto abierto y no se meta en el cerezo. Media cara negra con el ojo rojo encendido y el otro verde,
  bandana con emblema, cabello blanco en picos, cuerno negro de obsidiana y
  la capa de los Toros Negros con borde y emblema dorados. La espada mata
  demonios cruza en diagonal frente a él: como las cajas no se pueden
  inclinar, la hoja es una escalera de cajas a lo largo de su dirección,
  con mellas en el filo y el mango vendado. Lo rodea un aura de chispas y
  lenguas de antimagia (recogida en la espalda), con una luz puntual roja
  que tiñe el patio.

## Cómo funciona

Por cada píxel se lanza un rayo desde la cámara (`renderer.rs: render`) y se
busca la caja más cercana que toca (`scene.rs: closest_hit`). En el punto
de impacto, `cast_ray` calcula:

1. **Color propio:** textura × albedo, con luz ambiental **hemisférica**:
   las caras que miran arriba reciben el azul del cielo y las que miran
   abajo, el rebote rosado del mar de nubes.
2. **Luces:** para cada luz, difuso de Lambert (`N·L`) y brillo especular de
   Phong (`(R·V)^shininess`). Hay ocho luces: el sol cálido, un relleno frío
   y seis puntuales (braseros, farol, linternas de piedra y la energía cian
   del altar). Las puntuales se atenúan con `(1 - (d/r)²)²` y no alumbran
   más allá de su alcance `r`.
3. **Sombras:** un rayo hacia cada luz. Si choca con algo antes de llegar,
   pasa solo la fracción que deja pasar ese objeto: nada a través de la
   piedra, casi todo a través del cristal. El origen se desplaza un poco
   (*bias*) para evitar el *shadow acne*.
4. **Reflexión:** un rayo secundario en la dirección `R = I - 2(I·N)N`. En
   los metales se filtra por el color de la superficie (el oro refleja
   dorado); en los demás materiales conserva el color de lo reflejado. En
   las superficies opacas pulidas, el reflejo crece al mirarlas de lado
   (Fresnel con la aproximación de Schlick, usando la reflectividad como
   reflejo de frente): por eso el mármol y la obsidiana parecen espejos
   cuando se ven a ras.
5. **Refracción:** un rayo que atraviesa el material doblándose según la
   ley de Snell. Si el ángulo no permite salir, hay reflexión interna total.
   La aproximación de Schlick (Fresnel) decide cuánto se refleja y cuánto
   se refracta según el ángulo de vista.
6. **Skybox:** si el rayo no toca nada, el color sale de un cielo
   procedural que solo depende de la dirección del rayo: degradado de
   atardecer, sol, luna con mares, nubes altas y mar de nubes con ruido
   fractal propio, montañas y estrellas. Las nubes se sombrean leyendo el
   ruido un poco más hacia el sol: si ahí hay más nube, el punto queda en
   sombra; si hay menos, es un borde iluminado.

Los rebotes son recursivos, con un máximo de 4 (`MAX_DEPTH`).

Detalles extra de calidad:

- **Bump mapping:** el brillo de la textura se usa como mapa de alturas.
  Con diferencias finitas se calcula su pendiente en `u` y en `v`, y la
  normal se inclina sobre la tangente y la bitangente de la cara. Así las
  juntas de la piedra, los surcos de la corteza y el empedrado reciben la
  luz como si tuvieran relieve, aunque las cajas sean planas.
- **Brillo propio parcial (`glow`):** a diferencia de `emission` (que
  reemplaza la iluminación), se suma a ella multiplicado por el color de la
  textura. Las partes claras brillan mucho y las oscuras casi nada: así
  solo se encienden las grietas rojas de la espada de Asta.
- **Ondas en el agua:** la normal se inclina con una suma de senos según la
  posición (como un *normal map* calculado), así el reflejo y la refracción
  ondulan.
- **Bruma de distancia:** lo lejano se funde con el color del cielo
  (`1 - e^(-densidad·d)`), lo que da profundidad: las islas lejanas del
  archipiélago se ven claras y perdidas en el aire.
- **Post-proceso** (`postprocess.rs`): *tone mapping* con rodilla suave, que
  comprime las luces muy brillantes en vez de cortarlas en blanco, y una
  viñeta sutil.
- **Antialiasing:** con la cámara quieta, cada píxel promedia 4 rayos en una
  grilla rotada, y los bordes de las cajas quedan suaves.
- **Dragones en coordenadas locales** (`dragon.rs`): el modelo se describe
  con adelante, arriba y lado, y se coloca en cualquiera de las 4
  direcciones y escalas. Así hay tres dragones con distintas poses y
  rumbos, aunque las cajas no se puedan rotar.

## Rendimiento

En un i7-12700H (20 hilos), a 800×600, medido con `--bench`:

| Vista | Tiempo por cuadro |
|---|---|
| General | ~117 ms |
| Altar y cristal | ~250 ms |
| Estanque | ~225 ms |
| Contraluz, obelisco, desde abajo, monolitos, All Might, Asta | ~70–100 ms |

Los tiempos varían bastante según el modo de energía del equipo: medida
justo antes, en las mismas condiciones, la versión anterior de la escena
(836 cajas, sin mármol ni bump mapping) promediaba ~90 ms. La diferencia
sale sobre todo de los reflejos nuevos (el piso de mármol lanza un rayo
reflejado en cada píxel) y de las cajas extra.

Esos tiempos son sin antialiasing, que es como se renderiza mientras la
cámara se mueve. El cuadro quieto con antialiasing tarda unas 4 veces más,
pero se calcula una sola vez.

Optimizaciones, en el orden en que se hicieron (cada una está en su propio
commit, con su medición):

- **Intersect completo solo para la caja ganadora:** primero se busca solo
  la distancia, y la normal y la UV se calculan una vez.
- **Render en paralelo** con `std::thread::scope`. Las filas se reparten
  dinámicamente: cada hilo toma la siguiente libre, así los núcleos rápidos
  del CPU híbrido no esperan a los lentos.
- **Vista previa a media resolución** mientras la cámara se mueve.
- **Corte por importancia:** no se lanza un rebote que aportaría menos del
  6 % al píxel. Por ejemplo, la piedra (2 % de reflejo) ya no paga un rayo
  extra por píxel. Bajó el promedio de 59 a 37 ms.
- **BVH** (`bvh.rs`): árbol de cajas envolventes construido partiendo por
  el eje más largo, que se recorre abriendo primero el hijo más cercano.

## Estructura del código

| Archivo | Contenido |
|---|---|
| `main.rs` | Ventana, controles, vistas predefinidas, modos `--screenshot` y `--bench` |
| `camera.rs` | Cámara orbital con movimiento suavizado |
| `renderer.rs` | `cast_ray` (iluminación, sombras, reflexión con Fresnel, refracción, bump mapping, ondas, bruma) y render en paralelo con antialiasing |
| `postprocess.rs` | Tone mapping y viñeta |
| `placement.rs` | Coordenadas locales (adelante, arriba, lado) para modelos voxel |
| `dragon.rs` | Modelo de dragón |
| `characters.rs` | All Might y Asta |
| `scene.rs` | Construcción del diorama, luces, búsqueda de impactos y sombras |
| `bvh.rs` | Jerarquía de cajas envolventes |
| `cube.rs` | Caja alineada a los ejes: intersección por el método *slab*, normal y UV |
| `material.rs` | Parámetros de cada material |
| `procedural.rs` | Texturas pixel-art y ruido (hash, value noise, ruido fractal, versiones periódicas y Voronoi) |
| `skybox.rs` | Cielo procedural |
| `texture.rs` | Carga, muestreo, alturas bilineales y guardado de texturas |
| `light.rs` | Luces lejanas y puntuales con atenuación |
| `color.rs`, `framebuffer.rs`, `ray_intersect.rs` | Tipos básicos |

## Dependencias

Solo las permitidas en el curso:

- [`nalgebra-glm`](https://crates.io/crates/nalgebra-glm) 0.21.0, para vectores.
- [`raylib`](https://crates.io/crates/raylib) 6.0.0 con
  `SUPPORT_IMAGE_GENERATION`, para la ventana, la entrada y las imágenes.

Las texturas, el ruido y el paralelismo están implementados a mano, con la
librería estándar de Rust.
