use sdl2;
use std::ops;

pub const PI: f64 = 3.14159265358979323846264338327950288419716939937510; //idk how many digits it can store but this many can't hurt

#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f64, //left-/right+
    pub y: f64, //down-/up+
    pub z: f64, //backward-/forward+
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3::new(
            self.x * rhs,
            self.y * rhs,
            self.z * rhs,
        )
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

impl ops::Add<Color> for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Self::Output {
        Color::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl ops::Mul<f64> for Color {
    type Output = Color;

    fn mul(self, rhs: f64) -> Self::Output {
        Color::new(
            (self.r as f64 * rhs) as u8,
            (self.g as f64 * rhs) as u8,
            (self.b as f64 * rhs) as u8,
        )
    }
}

impl Into<sdl2::pixels::Color> for Color {
    fn into(self) -> sdl2::pixels::Color {
        sdl2::pixels::Color::RGB(self.r, self.g, self.b)
    }
}

pub struct SceneInfo {
    pub sun_orientation: Vec3,
    pub verts: Vec<Vec3>,
    pub tris: Vec<(usize, usize, usize)>, //optimize by packing into smaller areas
}

#[derive(Clone, Copy)]
pub struct Ray {
    pub pos: Vec3,
    pub orientation: Vec3,
}

impl Ray {
    fn len(&self) -> f64 {
        (self.pos.x * self.pos.x + self.pos.y * self.pos.y + self.pos.z * self.pos.z).sqrt()
    }

    pub fn normalize(&mut self) {
        let len = self.len();
        self.pos.x = self.pos.x / len;
        self.pos.y = self.pos.y / len;
        self.pos.z = self.pos.z / len;
    }

    pub fn get_color(&mut self) -> Color {
        self.get_background_color()
    }

    fn get_background_color(&mut self) -> Color {
        self.normalize();
        let factor = (self.orientation.y + 0.5).clamp(0.0, 1.0);
        //print!("{:.1},{:.1},{:.1}   ", self.orientation.x, self.orientation.y, self.orientation.z);
        Color::new(255, 255, 255) * (1.0 - factor)
            + Color::new(105, 212, 236) * (factor)
    }
}
