use nalgebra_glm::{cross, normalize, Vec3};

// Camara orbital: en vez de guardar la posicion directamente, guardamos
// en que punto mira (target) y desde donde lo rodea en coordenadas
// esfericas (yaw, pitch, distance). La posicion (eye) se deriva de eso,
// asi rotar el diorama es solo cambiar dos angulos y hacer zoom es solo
// cambiar la distancia.
//
// Movimiento suave: el input no mueve la camara directamente sino los
// valores "deseados" (desired_*); en cada frame update() acerca los valores
// reales a los deseados. Asi la camara acelera y frena sin saltos.
pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,      // angulo horizontal alrededor del eje Y (radianes)
    pub pitch: f32,    // angulo vertical sobre el horizonte (radianes)
    pub distance: f32, // radio de la orbita
    pub fov: f32,      // campo de vision vertical (radianes)
    desired_yaw: f32,
    desired_pitch: f32,
    desired_distance: f32,
}

// Limites para no pasar por encima de los polos (la base se voltea)
// ni meternos dentro de la escena o alejarnos al infinito.
const MIN_PITCH: f32 = -1.4;
const MAX_PITCH: f32 = 1.4;
const MIN_DISTANCE: f32 = 3.0;
const MAX_DISTANCE: f32 = 30.0;

// Que tan rapido la camara alcanza los valores deseados (por segundo).
// Mas alto = mas directo; mas bajo = mas "flotante".
const SMOOTHING: f32 = 10.0;

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
        let pitch = pitch.clamp(MIN_PITCH, MAX_PITCH);
        let distance = distance.clamp(MIN_DISTANCE, MAX_DISTANCE);
        OrbitCamera {
            target,
            yaw,
            pitch,
            distance,
            fov,
            desired_yaw: yaw,
            desired_pitch: pitch,
            desired_distance: distance,
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

    // Rotacion: cambia los angulos deseados (los limites se aplican aqui).
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.desired_yaw += delta_yaw;
        self.desired_pitch = (self.desired_pitch + delta_pitch).clamp(MIN_PITCH, MAX_PITCH);
    }

    // Zoom: acerca (delta < 0) o aleja (delta > 0) la distancia deseada.
    pub fn zoom(&mut self, delta: f32) {
        self.desired_distance = (self.desired_distance + delta).clamp(MIN_DISTANCE, MAX_DISTANCE);
    }

    // Acerca los valores reales a los deseados. En cada frame se recorre la
    // fraccion (1 - e^(-SMOOTHING * dt)) de lo que falta: el resultado no
    // depende de los FPS. Devuelve true si la camara todavia se esta moviendo.
    pub fn update(&mut self, dt: f32) -> bool {
        let t = 1.0 - (-SMOOTHING * dt).exp();
        self.yaw += (self.desired_yaw - self.yaw) * t;
        self.pitch += (self.desired_pitch - self.pitch) * t;
        self.distance += (self.desired_distance - self.distance) * t;

        // Cuando ya casi llego, se ajusta exacto y se deja de re-renderizar.
        let remaining = (self.desired_yaw - self.yaw).abs()
            + (self.desired_pitch - self.pitch).abs()
            + (self.desired_distance - self.distance).abs() * 0.1;
        if remaining < 1e-3 {
            self.yaw = self.desired_yaw;
            self.pitch = self.desired_pitch;
            self.distance = self.desired_distance;
            return false;
        }
        true
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
