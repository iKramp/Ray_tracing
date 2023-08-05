extern crate vector3d;

use super::data::CamData;
use super::hit::*;
use super::material::*;
use crate::Resources;
use core::f64::consts::PI;
use rand::prelude::*;
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
    let len_product = lhs.norm2().sqrt() * rhs.norm2().sqrt();
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
        self.orientation.norm2().sqrt()
    }

    pub fn normalize(&mut self) {
        self.orientation = self.orientation / self.len();
    }

    pub fn trace_ray(
        &mut self,
        scene_info: &super::data::SceneInfo,
        ray_depth: u32,
        rng: &mut ThreadRng,
        resources: std::rc::Rc<Resources>,
    ) -> Vector3d<f64> {
        self.normalize();
        let mut record = HitRecord::new(resources.clone());
        record.ray = *self;
        record.normal = self.orientation;
        normalize_vec(&mut record.normal);
        if ray_depth == 0 {
            //child modules count limit
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

    pub fn render(
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        event_pump: &mut sdl2::EventPump,
        data: &CamData,
        scene_info: &super::data::SceneInfo,
        resources: std::rc::Rc<Resources>,
    ) {
        let mut rng = thread_rng();

        let start = std::time::Instant::now();

        for pix_y in 0..data.canvas_height {
            for pix_x in 0..data.canvas_width {
                let color = Self::get_color((pix_x, pix_y), &mut rng, data, scene_info, &resources);
                canvas.set_draw_color(sdl2::pixels::Color::RGB(
                    color.x as u8,
                    color.y as u8,
                    color.z as u8,
                ));
                let _res = canvas.draw_point((pix_x as i32, pix_y as i32));
            }
            println!(
                "estimated time left: {}s",
                (std::time::Instant::now() - start).as_secs_f64() / (pix_y as f64 + 1.0)
                    * (data.canvas_height - pix_y) as f64
            );
            canvas.present();

            for event in event_pump.poll_iter() {
                if let sdl2::event::Event::Quit { .. } = event {
                    return;
                }
            }
        }
    }

    pub fn get_color(
        (pix_x, pix_y): (usize, usize),
        rng: &mut ThreadRng,
        data: &super::data::CamData,
        scene_info: &super::data::SceneInfo,
        resources: &std::rc::Rc<Resources>,
    ) -> Vector3d<f64> {
        let mut color = Vector3d::new(0.0, 0.0, 0.0);
        for _i in 0..data.samples {
            let mut vec = claculate_vec_dir_from_cam(
                data,
                (
                    pix_x as f64 + rng.gen_range(0.0..1.0),
                    pix_y as f64 + rng.gen_range(0.0..1.0),
                ),
            );
            vec.normalize();
            color = color + vec.trace_ray(scene_info, 5, rng, resources.clone());
        }
        color = color / data.samples as f64 / 256.0;
        color.x = color.x.sqrt().clamp(0.0, 0.999999999);
        color.y = color.y.sqrt().clamp(0.0, 0.999999999);
        color.z = color.z.sqrt().clamp(0.0, 0.999999999);
        color = color * 256.0;
        color
    }
}
