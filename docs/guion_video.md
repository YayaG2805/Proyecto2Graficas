# Guion del video (≈ 4 minutos)

## Antes de grabar

1. Compilar y abrir: `cargo run`.
2. Presionar `P` para desactivar la vista previa: así todo se ve a
   resolución completa mientras la cámara se mueve. El panel de ayuda debe
   decir `P: vista previa rapida [OFF]`.
3. Dejar el panel de ayuda visible (muestra los FPS y los controles) o
   ocultarlo con `H` en las tomas "bonitas".
4. Opcional: `F11` para pantalla completa (la imagen se ve más grande;
   el render sigue siendo de 800×600, así que la velocidad no cambia).
5. Grabar la ventana con OBS, la Xbox Game Bar (`Win + Alt + R`) o similar.

## Tomas

| # | Tiempo | Acción | Qué decir o mostrar |
|---|---|---|---|
| 1 | 0:00 – 0:15 | Vista `1` (general), quieto (se ve con antialiasing) | "Santuario flotante: raytracer en Rust, todo en el CPU: 1447 cajas, 35 materiales y 10 luces." |
| 2 | 0:15 – 0:35 | `Espacio` (giro automático) una vuelta completa | **Rotación.** Se ve todo: templo, cerezo, fuente, pagoda, puentes, isla alta, dirigible, farolillos, dragones y el archipiélago de islas lejanas en la bruma. |
| 3 | 0:35 – 0:50 | `Espacio` para detener; `Q`/`E` o la rueda para acercar y alejar; arrastrar con el mouse | **Zoom y rotación manual.** |
| 4 | 0:50 – 1:15 | Vista `2` (altar y cristal); bajar la cámara con `S` | **Refracción:** el fondo se deforma a través del cristal. **Reflexión:** el piso de mármol refleja las columnas y el cristal, más fuerte al bajar la cámara (Fresnel); el metal refleja dorado. |
| 5 | 1:15 – 1:35 | Vista `3` (estanque); inclinar con `W`/`S` | **Agua:** se ven los koi a través de ella (refracción). Al bajar la cámara refleja más el cielo (Fresnel). |
| 6 | 1:35 – 1:50 | Vista `7` (monolitos); rotar con `A`/`D` | **Reflexión:** monolitos de obsidiana que reflejan el cielo, el orbe y a los otros monolitos (rebotes recursivos). |
| 7 | 1:50 – 2:05 | Vista `4` (contraluz) y vista `6` (desde abajo) | **Skybox:** sol, nubes altas iluminadas del lado del sol, mar de nubes con relieve, montañas, estrellas y la luna. |
| 8 | 2:05 – 2:25 | Vista `1`; acercarse a los braseros y luego ir a la isla del farol | **Luces puntuales:** el fuego ilumina las columnas y el poste con luz naranja. **Sombras** largas del sol bajo. |
| 9 | 2:25 – 2:45 | Recorrer piedra, madera, metal, cristal y agua de cerca | **5 materiales:** cada uno con su textura y sus parámetros (mostrar la tabla del README o la de la consola). **Bump mapping:** las juntas de la piedra y el empedrado tienen relieve. |
| 10 | 2:45 – 3:00 | Vista `5` (obelisco) | Isla alta con el obelisco de cristal y la bandera; detrás, el dirigible. |
| 11 | 3:00 – 3:15 | Acercarse a un dragón y a la pagoda | **Dragones:** escamas de marfil, alas translúcidas que brillan a contraluz, ojos de fuego. **Pagoda:** tejas vidriadas que reflejan, farolillos de papel, puente colgante. |
| 12 | 3:15 – 3:25 | Acercarse al cerezo y al islote de cristales | Pétalos y luciérnagas; núcleos mágicos que se ven a través del vidrio (refracción). |
| 13 | 3:25 – 3:40 | Vista `8` (All Might) | **Personaje gigante:** traje con sus franjas y paneles, ojos que brillan en la sombra, mechones en V. Hecho con coordenadas locales como los dragones. |
| 14 | 3:40 – 3:55 | Vista `9` (Asta); rotar un poco con `A`/`D` | **Espada mata demonios:** las grietas rojas brillan con `glow` sobre el hierro negro; aura de antimagia y luz roja que tiñe el patio frente al templo. |
| 15 | 3:55 – 4:05 | `R` para volver a la vista inicial y esperar un segundo quieto | Cierre: la imagen se vuelve a calcular con antialiasing. |

## Después de grabar

1. Subir el video a YouTube (no listado) o a Google Drive con acceso por
   enlace.
2. Pegar el enlace en la sección **Video** del `README.md`.
