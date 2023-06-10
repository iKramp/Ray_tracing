extern crate vector3d;

use super::hit::*;
use super::material::*;
use crate::Resources;
use vector3d::Vector3d;

pub fn vector_angle(lhs: Vector3d<f64>, rhs: Vector3d<f64>) -> f64 {
    let dot_product = lhs.dot(rhs);
    let len_product = lhs.norm2().sqrt() * rhs.norm2().sqrt();
    (dot_product / len_product).acos()
}

pub struct SceneInfo {
    pub sun_orientation: Vector3d<f64>,
    pub hittable_objects: Vec<Box<dyn HitObject>>,
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
        self.orientation.norm2().sqrt()
    }

    pub fn normalize(&mut self) {
        self.orientation = self.orientation / self.len();
    }

    pub fn trace_ray(
        &mut self,
        scene_info: &SceneInfo,
        ray_depth: u32,
        rng: &mut rand::prelude::ThreadRng,
        resources: std::rc::Rc<Resources>,
    ) -> Vector3d<f64> {
        self.normalize();
        let mut record = HitRecord::new(resources.clone());
        record.ray = *self;
        record.normal = self.orientation;
        normalize_vec(&mut record.normal);
        if ray_depth == 0 {
            //child ray count limit
            return Vector3d::default();
            //return record.material.get_stop_color(&record); //return background color
        }

        for object in &scene_info.hittable_objects {
            object.hit(self, (0.001, f64::MAX), &mut record);
        }

        record.ray.normalize();
        normalize_vec(&mut record.normal);
        let return_type = record.material.get_next_ray_dir(&record, rng);
        match return_type {
            RayReturnState::Absorb => Vector3d::new(0.0, 0.0, 0.0),
            RayReturnState::Stop => record.material.get_stop_color(&record),
            RayReturnState::Ray(ray) => {
                let mut next_ray = Ray::new(record.pos, ray);
                let next_color = next_ray.trace_ray(scene_info, ray_depth - 1, rng, resources);
                record.material.get_color(&record, next_color)
            }
        }
    }
}
