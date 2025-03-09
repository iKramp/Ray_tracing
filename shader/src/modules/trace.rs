use super::hit::*;
use super::material::*;
use super::rand_float;
use super::ObjectInfo;
use shared::glam;
use shared::CamData;
//use crate::Resources;
use core::f64::consts::PI;
#[allow(unused_imports)] //actually used for .sqrt because we don't allow std
use spirv_std::num_traits::Float;
use spirv_std::ByteAddressableBuffer;
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

    let cam_vec = data.orientation;
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

    pub fn shoot_ray() {}

    pub fn trace_ray(
        &mut self,
        scene_info: &shared::SceneInfo,
        seed: &mut u32,
        //resources: Rc<Resources>,
        //cam_data: &CamData,
        color: &mut Vector3d,
        objects: &ObjectInfo,
    ) -> (RayReturn, HitRecord) {
        self.normalize();
        let mut record = HitRecord::new(/*resources.clone()*/);
        record.ray = *self;
        record.normal = self.orientation;
        normalize_vec(&mut record.normal);

        record.ray.normalize(); //normal rendering
        record.normal.normalize();

        for i in 0..scene_info.num_objects as usize {
            let object = &objects.object_buffer[i];
            let mesh = Mesh {
                verts: objects.vertex_buffer,
                //tris: &objects.triangle_buffer[object.first_triangle as usize..object.last_triangle as usize],
                tris: objects.triangle_buffer,
                triangle_range: (object.first_triangle, object.last_triangle),
            };
            let ray = self.transform_by_obj_matrix(object.transform);
            let clamp = (0.00001, record.t);
            mesh.hit(&ray, clamp, &mut record);
        }

        record.ray.normalize();
        record.normal.normalize();

        if record.t == f64::INFINITY {
            let sky_material = BackgroundMaterial {};
            let stop_col = sky_material.get_stop_color(&record);
            color.x *= stop_col.x;
            color.y *= stop_col.y;
            color.z *= stop_col.z;
            return (
                RayReturn {
                    state: RayReturnState::Stop,
                    ray: Vector3d::default(),
                },
                record,
            );
        }

        let material = MetalMaterial::new(Vector3d::new(230.0, 230.0, 230.0), 0.0);

        let ray_return = material.get_next_ray_dir(&record, seed); //record.material.get_next_ray_dir(&record/*, rng*/);
        match ray_return.state {
            RayReturnState::Absorb => *color = Vector3d::default(),
            RayReturnState::Stop => {
                let stop_col = material.get_stop_color(&record);
                color.x *= stop_col.x;
                color.y *= stop_col.y;
                color.z *= stop_col.z;
            } //record.material.get_stop_color(&record),
            RayReturnState::Ray => {
                *color = material.get_color(&record, color) //record.material.get_color(&record, next_color)
            }
        }
        (ray_return, record)
    }

    pub fn get_color(
        (pix_x, pix_y): (usize, usize),
        mut rng_seed: u32,
        data: &CamData,
        scene_info: &shared::SceneInfo, /* resources: &Rc<Resources>*/
        objects: &ObjectInfo,
    ) -> Vector3d {
        let mut color = Vector3d::new(0.0, 0.0, 0.0);

        for _ in 0..data.samples {
            let mut curr_sample_color = Vector3d::new(1.0, 1.0, 1.0);
            let mut vec = claculate_vec_dir_from_cam(
                data,
                (
                    pix_x as f32 + rand_float(&mut rng_seed, (0.0, 1.0)),
                    pix_y as f32 + rand_float(&mut rng_seed, (0.0, 1.0)),
                ),
            );
            vec.normalize();

            for _ in 0..20 {
                //depth
                let (ray_return, record) = vec.trace_ray(
                    scene_info,
                    &mut rng_seed,
                    &mut curr_sample_color,
                    objects
                );
                match ray_return.state {
                    RayReturnState::Ray => {
                        vec = Ray::new(record.pos, ray_return.ray);
                    }
                    _ => {
                        break;
                    }
                }
            }

            color = color + curr_sample_color;
        }
        color = color / data.samples as f64 / 256.0;
        color.x = color.x.sqrt().clamp(0.0, 0.999999999);
        color.y = color.y.sqrt().clamp(0.0, 0.999999999);
        color.z = color.z.sqrt().clamp(0.0, 0.999999999);
        color = color * 256.0;
        color
    }

    fn transform_by_obj_matrix(&self, obj_matrix: glam::Mat4) -> Self {
        let pos = glam::Vec4::new(self.pos.x as f32, self.pos.y as f32, self.pos.z as f32, 1.0);
        let orientation = glam::Vec4::new(
            self.orientation.x as f32,
            self.orientation.y as f32,
            self.orientation.z as f32,
            0.0,
        );

        let pos = obj_matrix * pos;
        let orientation = obj_matrix * orientation;

        let pos = Vector3d::new(pos.x as f64, pos.y as f64, pos.z as f64);
        let orientation = Vector3d::new(orientation.x as f64, orientation.y as f64, orientation.z as f64);
        Self::new(pos, orientation)
    }
}
