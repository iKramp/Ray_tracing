use super::hit::*;
use super::material::*;
use shared::CamData;
//use crate::Resources;
use core::f64::consts::PI;
use spirv_std::num_traits::Float;
use vector3d::Vector3d;

pub fn claculate_vec_dir_from_cam(data: &CamData, (pix_x, pix_y): (f32, f32)) -> Ray {
    //only capable up to 180 deg FOV TODO: this has to be rewritten probably. it works, but barely
    let fov_rad: f32 = data.fov / 180.0 * PI as f32;
    let virt_canvas_height: f32 = (fov_rad / 2.0).tan();

    let pix_offset_y = (-pix_y / data.canvas_height as f32 + 0.5) * virt_canvas_height;
    let pix_offset_x = (pix_x / data.canvas_height as f32
        - 0.5 * (data.canvas_width as f32 / data.canvas_height as f32))
        * virt_canvas_height;

    let offset_yaw = pix_offset_x.atan();
    let offset_pitch = pix_offset_y.atan();

    let mut cam_vec = data.orientation;
    let cam_vec = cam_vec.normalize();

    let mut yaw: f32 = (cam_vec.x).asin();
    if cam_vec.z < 0.0 {
        yaw = PI as f32 - yaw;
    }
    let mut pitch: f32 = (cam_vec.y).asin();
    if cam_vec.z < 0.0 {
        pitch = PI as f32 - pitch;
    }

    yaw += offset_yaw;
    pitch += offset_pitch;

    Ray::new(
        Vector3d::new(data.pos.x as f64, data.pos.y as f64, data.pos.z as f64),
        Vector3d::new(
            (yaw.sin() * pitch.cos()) as f64,
            pitch.sin() as f64,
            (yaw.cos() * pitch.cos()) as f64,
        ),
    )
}

pub fn vector_angle(lhs: Vector3d, rhs: Vector3d) -> f64 {
    let dot_product = lhs.dot(rhs);
    let len_product = lhs.norm2().sqrt() * rhs.norm2().sqrt();
    (dot_product / len_product).acos()
}

#[derive(Clone, Copy)]
pub struct Ray {
    pub pos: Vector3d,
    pub orientation: Vector3d,
}

impl Ray {
    pub fn new(pos: Vector3d, orientation: Vector3d) -> Self {
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
        //scene_info: &super::data::SceneInfo,
        ray_depth: u32,
        //rng: &mut ThreadRng,
        /*resources: Rc<Resources>,*/
    ) -> Vector3d {
        self.normalize();
        let mut record = HitRecord::new(/*resources.clone()*/);
        record.ray = *self;
        record.normal = self.orientation;
        normalize_vec(&mut record.normal);
        if ray_depth == 0 {
            //child modules count limit
            return Vector3d::default();
            //return record.material.get_stop_color(&record); //return background color
        }

        /*for object in &scene_info.hittable_objects {
            object.hit(self, (0.001, f64::MAX), &mut record);
        }*/

        record.ray.normalize();
        normalize_vec(&mut record.normal);
        /*let return_type = record.material.get_next_ray_dir(&record/*, rng*/);
        match return_type {
            RayReturnState::Absorb => Vector3d::new(0.0, 0.0, 0.0),
            RayReturnState::Stop => record.material.get_stop_color(&record),
            RayReturnState::Ray(ray) => {
                let mut next_ray = Ray::new(record.pos, ray);
                let next_color = next_ray.trace_ray(scene_info, ray_depth - 1/*, rng,*/ /*resources*/);
                record.material.get_color(&record, next_color)
            }
        }*/
        Vector3d::default()
    }

    pub fn get_color(
        (pix_x, pix_y): (usize, usize),
        /*rng: &mut ThreadRng, */
        data: &CamData,
        /*scene_info: &super::data::SceneInfo,*/ /* resources: &Rc<Resources>*/
    ) -> Vector3d {
        //let mut color = Vector3d::new(0.0, 0.0, 0.0);
        //for _i in 0..data.samples {
        let vec = claculate_vec_dir_from_cam(
            data,
            (
                pix_x as f32, // + rng.gen_range(0.0..1.0),
                pix_y as f32, // + rng.gen_range(0.0..1.0),
            ),
        );

        let mut temp = vec;
        temp.normalize();
        let factor = (temp.orientation.y + 0.5).clamp(0.0, 1.0);

        return Vector3d::new(255.0, 255.0, 255.0) * (1.0 - factor)
            + Vector3d::new(0.5, 0.7, 1.0) * 255.0 * factor;

        /*return vec.orientation;
        vec.normalize();
        color = color + vec.trace_ray(&scene_info, 5 /*, rng*/, /*resources.clone()*/);
        }
        color = color / 1.0/*data.samples as f64*/ / 256.0;
        color.x = color.x.sqrt().clamp(0.0, 0.999999999);
        color.y = color.y.sqrt().clamp(0.0, 0.999999999);
        color.z = color.z.sqrt().clamp(0.0, 0.999999999);
        color = color * 256.0;
        color*/
    }
}
