use sdl2;
use std::ops;
extern crate vector3d;

use super::hit::*;
use vector3d::Vector3d;

pub const PI: f64 = 3.14159265358979323846264338327950288419716939937510; //idk how many digits it can store but this many can't hurt

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
    pub sun_orientation: Vector3d<f64>,
    pub verts: Vec<Vector3d<f64>>,
    pub tris: Vec<(usize, usize, usize)>, //optimize by packing into smaller areas
    pub spheres: Vec<Sphere>,
}

#[derive(Clone, Copy)]
pub struct Ray {
    pub pos: Vector3d<f64>,
    pub orientation: Vector3d<f64>,
}

impl Ray {
    pub fn new(pos: Vector3d<f64>, orientation: Vector3d<f64>) -> Self {
        Ray { pos, orientation }
    }

    fn len(&self) -> f64 {
        (self.orientation.x * self.orientation.x
            + self.orientation.y * self.orientation.y
            + self.orientation.z * self.orientation.z)
            .sqrt()
    }

    pub fn normalize(&mut self) {
        let len = self.len();
        self.orientation.x = self.orientation.x / len;
        self.orientation.y = self.orientation.y / len;
        self.orientation.z = self.orientation.z / len;
    }

    pub fn get_color(&mut self, scene_info: &SceneInfo) -> Color {
        self.normalize();
        let mut record = HitRecord::new();
        for sphere in &scene_info.spheres {
            let _result = sphere.hit(self, (0.0, f64::MAX), &mut record);
        }
        if record.t != f64::INFINITY {
            return Color::new(
                ((record.normal.x + 1.0) * 255.0 / 2.0) as u8,
                ((record.normal.y + 1.0) * 255.0 / 2.0) as u8,
                ((record.normal.z + 1.0) * 255.0 / 2.0) as u8,
            );
        }

        self.get_background_color()
    }

    fn get_background_color(&self) -> Color {
        let mut temp = self.clone();
        temp.normalize();
        let factor = (temp.orientation.y + 0.5).clamp(0.0, 1.0);
        Color::new(255, 255, 255) * (1.0 - factor) + Color::new(105, 212, 236) * (factor)
    }
}
