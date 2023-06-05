use sdl2;
use std::ops;
extern crate vector3d;

use super::hit::*;
use vector3d::Vector3d;

pub const PI: f64 = 3.14159265358979323846264338327950288419716939937510; //idk how many digits it can store but this many can't hurt

pub struct SceneInfo {
    pub sun_orientation: Vector3d<f64>,
    pub hittable_objects: Vec<Box<dyn HitObject>>
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

    pub fn get_color(&mut self, scene_info: &SceneInfo) -> Vector3d<f64> {
        self.normalize();
        let mut record = HitRecord::new();
        /*for sphere in &scene_info.spheres {
            let _result = sphere.hit(self, (0.0, f64::MAX), &mut record);
        }*/
        for object in &scene_info.hittable_objects {
            let _res = object.hit(self, (0.0, f64::MAX), &mut record);
        }
        if record.t != f64::INFINITY {
            return Vector3d::new(
                (record.normal.x + 1.0) * 255.0 / 2.0,
                (record.normal.y + 1.0) * 255.0 / 2.0,
                (record.normal.z + 1.0) * 255.0 / 2.0,
            );
        }

        self.get_background_color()
    }

    fn get_background_color(&self) -> Vector3d<f64> {
        let mut temp = self.clone();
        temp.normalize();
        let factor = (temp.orientation.y + 0.5).clamp(0.0, 1.0);
        Vector3d::new(255.0, 255.0, 255.0) * (1.0 - factor) + Vector3d::new(105.0, 212.0, 236.0) * (factor)
    }
}
