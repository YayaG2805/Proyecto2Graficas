use nalgebra_glm::{cross, normalize, Vec3};

// Camara orbital: en vez de guardar la posicion directamente, guardamos
// en que punto mira (target) y desde donde lo rodea en coordenadas
// esfericas (yaw, pitch, distance). La posicion (eye) se deriva de eso,
// asi rotar el diorama es solo cambiar dos angulos y hacer zoom es solo
// cambiar la distancia.
pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,      // angulo horizontal alrededor del eje Y (radianes)
    pub pitch: f32,    // angulo vertical sobre el horizonte (radianes)
    pub distance: f32, // radio de la orbita
    pub fov: f32,      // campo de vision vertical (radianes)
}

// Limites para no pasar por encima de los polos (la base se voltea)
// ni meternos dentro de la escena o alejarnos al infinito.
const MIN_PITCH: f32 = -1.4;
const MAX_PITCH: f32 = 1.4;
const MIN_DISTANCE: f32 = 3.0;
const MAX_DISTANCE: f32 = 30.0;

// Base ortonormal de la camara, calculada una vez por frame y reutilizada
// para todos los pixeles.
pub struct CameraBasis {
    pub eye: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub perspective_scale: f32,
}

impl OrbitCamera {
    pub fn new(target: Vec3, yaw: f32, pitch: f32, distance: f32, fov: f32) -> Self {
        OrbitCamera {
            target,
            yaw,
            pitch: pitch.clamp(MIN_PITCH, MAX_PITCH),
            distance: distance.clamp(MIN_DISTANCE, MAX_DISTANCE),
            fov,
        }
    }

    // Esfericas -> cartesianas, relativas al target.
    pub fn eye(&self) -> Vec3 {
        let offset = Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        );
        self.target + offset * self.distance
    }

    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch = (self.pitch + delta_pitch).clamp(MIN_PITCH, MAX_PITCH);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance + delta).clamp(MIN_DISTANCE, MAX_DISTANCE);
    }

    pub fn basis(&self) -> CameraBasis {
        let eye = self.eye();
        let forward = normalize(&(self.target - eye));
        let world_up = Vec3::new(0.0, 1.0, 0.0);
        let right = normalize(&cross(&forward, &world_up));
        let up = cross(&right, &forward);

        CameraBasis {
            eye,
            forward,
            right,
            up,
            perspective_scale: (self.fov / 2.0).tan(),
        }
    }
}

impl CameraBasis {
    // Convierte un punto de pantalla ya normalizado a [-1, 1] (con aspect
    // ratio aplicado en x) en la direccion del rayo en coordenadas de mundo.
    // Es el mismo calculo que en clase (screen_x, screen_y, -1), pero en vez
    // de usar los ejes fijos del mundo usamos los ejes de la camara.
    pub fn ray_direction(&self, screen_x: f32, screen_y: f32) -> Vec3 {
        let dir = self.right * (screen_x * self.perspective_scale)
            + self.up * (screen_y * self.perspective_scale)
            + self.forward;
        normalize(&dir)
    }
}
