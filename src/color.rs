use raylib::color::Color as RColor;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b }
    }

    pub fn to_raylib(&self) -> RColor {
        RColor::new(
            self.r.clamp(0.0, 255.0) as u8,
            self.g.clamp(0.0, 255.0) as u8,
            self.b.clamp(0.0, 255.0) as u8,
            255,
        )
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Color;
    fn mul(self, k: f32) -> Color {
        Color::new(self.r * k, self.g * k, self.b * k)
    }
}

impl std::ops::Add for Color {
    type Output = Color;
    fn add(self, other: Color) -> Color {
        Color::new(self.r + other.r, self.g + other.g, self.b + other.b)
    }
}

impl std::ops::Mul<Color> for Color {
    type Output = Color;
    fn mul(self, other: Color) -> Color {
        Color::new(
            self.r * other.r / 255.0,
            self.g * other.g / 255.0,
            self.b * other.b / 255.0,
        )
    }
}
