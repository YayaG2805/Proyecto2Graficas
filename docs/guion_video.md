# Guion del video (≈ 3 minutos y medio)

## Antes de grabar

1. Compilar y abrir: `cargo run`.
2. Presionar `P` para desactivar la vista previa: así todo se ve a
   resolución completa mientras la cámara se mueve. El panel de ayuda debe
   decir `P: vista previa rapida [OFF]`.
3. Dejar el panel de ayuda visible (muestra los FPS y los controles) o
   ocultarlo con `H` en las tomas "bonitas".
4. Grabar la ventana con OBS, la Xbox Game Bar (`Win + Alt + R`) o similar.

## Tomas

| # | Tiempo | Acción | Qué decir o mostrar |
|---|---|---|---|
| 1 | 0:00 – 0:15 | Vista `1` (general), quieto (se ve con antialiasing) | "Santuario flotante: raytracer en Rust, todo en el CPU: 836 cajas, 17 materiales y 8 luces." |
| 2 | 0:15 – 0:35 | `Espacio` (giro automático) una vuelta completa | **Rotación.** Se ve todo: templo, cerezo, fuente, pagoda, puentes, isla alta, islotes de cristal y los tres dragones. |
| 3 | 0:35 – 0:50 | `Espacio` para detener; `Q`/`E` o la rueda para acercar y alejar; arrastrar con el mouse | **Zoom y rotación manual.** |
| 4 | 0:50 – 1:15 | Vista `2` (altar y cristal); rotar un poco con `A`/`D` | **Refracción:** el fondo se deforma a través del cristal. **Reflexión:** el marco y los capiteles de metal reflejan dorado. |
| 5 | 1:15 – 1:40 | Vista `3` (estanque); inclinar con `W`/`S` | **Agua:** refracta el fondo del estanque. Al bajar la cámara, refleja más el cielo (Fresnel). |
| 6 | 1:40 – 1:55 | Vista `4` (contraluz) | **Skybox:** sol, degradado de atardecer, nubes, montañas y estrellas. |
| 7 | 1:55 – 2:15 | Vista `1`; acercarse a los braseros y luego ir a la isla del farol | **Luces puntuales:** el fuego ilumina las columnas y el poste con luz naranja. **Sombras** largas del sol bajo. |
| 8 | 2:15 – 2:35 | Recorrer piedra, madera, metal, cristal y agua de cerca | **5 materiales:** cada uno con su textura y sus parámetros (mostrar la tabla del README o la de la consola). |
| 9 | 2:35 – 2:50 | Vista `5` (obelisco) y vista `6` (desde abajo) | Isla alta con el obelisco de cristal y la bandera; desde abajo, la roca iluminada por el rebote rosado de las nubes y los cristales mágicos. |
| 10 | 2:50 – 3:10 | Acercarse a un dragón y a la pagoda | **Dragones:** escamas de marfil, alas translúcidas que brillan a contraluz, ojos de fuego. **Pagoda:** tejas vidriadas que reflejan, ventanas encendidas, puente colgante. |
| 11 | 3:10 – 3:20 | Acercarse al cerezo y al islote de cristales | Pétalos en el suelo y en el aire; núcleos mágicos que se ven a través del vidrio (refracción). |
| 12 | 3:20 – 3:30 | `R` para volver a la vista inicial y esperar un segundo quieto | Cierre: la imagen se vuelve a calcular con antialiasing. |

## Después de grabar

1. Subir el video a YouTube (no listado) o a Google Drive con acceso por
   enlace.
2. Pegar el enlace en la sección **Video** del `README.md`.
