# Guion del video (≈ 3 minutos)

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
| 1 | 0:00 – 0:15 | Vista `1` (general), quieto | "Santuario flotante: raytracer en Rust, todo en el CPU, con un diorama de 176 cajas." |
| 2 | 0:15 – 0:35 | `Espacio` (giro automático) una vuelta completa | **Rotación.** Se ve la escena completa: templo, estanque, puente, isla satélite, santuario alto. |
| 3 | 0:35 – 0:50 | `Espacio` para detener; `Q`/`E` o la rueda para acercar y alejar; arrastrar con el mouse | **Zoom y rotación manual.** |
| 4 | 0:50 – 1:15 | Vista `2` (altar y cristal); rotar un poco con `A`/`D` | **Refracción:** el fondo se deforma a través del cristal. **Reflexión:** el marco y los capiteles de metal reflejan dorado. |
| 5 | 1:15 – 1:40 | Vista `3` (estanque); inclinar con `W`/`S` | **Agua:** refracta el fondo del estanque. Al bajar la cámara, refleja más el cielo (Fresnel). |
| 6 | 1:40 – 1:55 | Vista `4` (contraluz) | **Skybox:** sol, degradado de atardecer, nubes, montañas y estrellas. |
| 7 | 1:55 – 2:15 | Vista `1`; acercarse a los braseros y luego ir a la isla del farol | **Luces puntuales:** el fuego ilumina las columnas y el poste con luz naranja. **Sombras** largas del sol bajo. |
| 8 | 2:15 – 2:35 | Recorrer piedra, madera, metal, cristal y agua de cerca | **5 materiales:** cada uno con su textura y sus parámetros (mostrar la tabla del README o la de la consola). |
| 9 | 2:35 – 2:50 | Vista `5` (obelisco) y vista `6` (desde abajo) | Isla alta con el obelisco de cristal; roca natural y cristales bajo la isla. |
| 10 | 2:50 – 3:00 | `R` para volver a la vista inicial | Cierre. |

## Después de grabar

1. Subir el video a YouTube (no listado) o a Google Drive con acceso por
   enlace.
2. Pegar el enlace en la sección **Video** del `README.md`.
