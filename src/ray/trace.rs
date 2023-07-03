extern crate vector3d;

use super::data::CamData;
use super::hit::*;
use super::material::*;
use crate::Resources;
use core::f64::consts::PI;
use vector3d::Vector3d;

pub fn claculate_vec_dir_from_cam(data: &CamData, (pix_x, pix_y): (f64, f64)) -> Ray {
    //only capable up to 180 deg FOV TODO: this has to be rewritten probably. it works, but barely
    let fov_rad = data.fov / 180.0 * PI;
    let virt_canvas_height = (fov_rad / 2.0).tan();

    let pix_offset_y = (-pix_y / data.canvas_height as f64 + 0.5) * virt_canvas_height;
    let pix_offset_x = (pix_x / data.canvas_height as f64
        - 0.5 * (data.canvas_width as f64 / data.canvas_height as f64))
        * virt_canvas_height;

    //println!("{},{}   ", pix_offset_x, pix_offset_y);

    let offset_yaw = pix_offset_x.atan();
    let offset_pitch = pix_offset_y.atan();

    let mut cam_vec = data.transform;
    cam_vec.normalize();

    let mut yaw = cam_vec.orientation.x.asin();
    if cam_vec.orientation.z < 0.0 {
        yaw = PI - yaw;
    }
    let mut pitch = cam_vec.orientation.y.asin();
    if cam_vec.orientation.z < 0.0 {
        pitch = PI - pitch;
    }

    yaw += offset_yaw;
    pitch += offset_pitch;

    Ray::new(
        data.transform.pos,
        Vector3d::new(
            yaw.sin() * pitch.cos(),
            pitch.sin(),
            yaw.cos() * pitch.cos(),
        ),
    )
}

pub fn vector_angle(lhs: Vector3d<f64>, rhs: Vector3d<f64>) -> f64 {
    let dot_product = lhs.dot(rhs);
    let len_product = lhs.dot(lhs).sqrt() * rhs.dot(rhs).sqrt();
    (dot_product / len_product).acos()
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
        self.orientation.dot(self.orientation).sqrt()
    }

    pub fn normalize(&mut self) {
        let len = self.len();
        self.orientation = self.orientation / len;
    }

    pub fn trace_ray(
        &mut self,
        scene_info: &super::data::SceneInfo,
        ray_depth: u32,
        rng: &mut rand::prelude::ThreadRng,
        resources: std::rc::Rc<Resources>,
    ) -> Vector3d<f64> {
        self.normalize();
        let mut record = HitRecord::new(resources.clone());
        record.ray = *self;
        record.normal = self.orientation;
        normalize_vec(&mut record.normal);
        if ray_depth >= 1 {
            //child ray count limit
            return Vector3d::default();
            //return record.material.get_stop_color(&record); //return background color
        }

        for object in &scene_info.hittable_objects {
            let _res = object.hit(self, (0.001, f64::MAX), &mut record);
        }

        record.ray.normalize();
        normalize_vec(&mut record.normal);
        let return_type = record.material.get_next_ray_dir(&record, rng);
        match return_type {
            RayReturnState::Absorb => Vector3d::new(0.0, 0.0, 0.0),
            RayReturnState::Stop => record.material.get_stop_color(&record),
            RayReturnState::Ray(ray) => {
                let mut next_ray = Ray::new(record.pos, ray);
                let next_color = next_ray.trace_ray(scene_info, ray_depth + 1, rng, resources);
                record.material.get_color(&record, next_color)
            }
        }
    }
}
