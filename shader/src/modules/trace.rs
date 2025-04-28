use super::hit::*;
use super::material::*;
use super::rand_float;
use super::ObjectInfo;
use shared::glam;
use shared::glam::Vec3;
use shared::glam::Vec4;
use shared::materials::DiffuseMaterial;
use shared::CamData;
//use crate::Resources;
use core::f32::consts::PI;
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

pub fn claculate_vec_dir_from_cam(data: &CamData, (pix_x, pix_y): (f32, f32)) -> Ray {
    //fov is counted in degrees in the horizontal direction
    let fov = (data.fov * PI / 180.0) / 2.0;
    let edge_dist = fov.tan();
    let pix_x_dist = ((pix_x / data.canvas_width as f32) * 2.0 - 1.0) * edge_dist;
    let pix_y_dist = ((pix_y / data.canvas_height as f32) * 2.0 - 1.0) * edge_dist;
    let orientation_vec = Vec3::new(pix_x_dist, pix_y_dist, 1.0);
    let orientation_vec = data.transform.transform_vector3(orientation_vec);
    Ray::new(data.transform.transform_point3(Vec3::new(0.0, 0.0, 0.0)), orientation_vec)
}

pub fn vector_angle(lhs: Vec4, rhs: Vec4) -> f32 {
    let dot_product = lhs.dot(rhs);
    let len_product = lhs.length() * rhs.length();
    (dot_product / len_product).acos()
}

#[derive(Clone, Copy)]
pub struct Ray {
    pub pos: Vec3,
    pub orientation: Vec3,
}

impl Ray {
    pub fn new(pos: Vec3, orientation: Vec3) -> Self {
        Ray { pos, orientation }
    }

    pub fn normalize(&mut self) {
        self.orientation = self.orientation.normalize();
    }

    pub fn shoot_ray() {}

    pub fn trace_ray(
        &mut self,
        scene_info: &shared::SceneInfo,
        seed: &mut u32,
        //resources: Rc<Resources>,
        //cam_data: &CamData,
        color: &mut Vec3,
        objects: &ObjectInfo,
    ) -> (RayReturn, HitRecord) {

        self.normalize();
        let mut record = HitRecord::new(/*resources.clone()*/);
        record.ray = *self;
        record.ray.normalize();

        for i in 0..scene_info.num_objects as usize {
            let object = &objects.object_buffer[i];
            let mesh = Mesh {
                verts: objects.vertex_buffer,
                //tris: &objects.triangle_buffer[object.first_triangle as usize..object.last_triangle as usize],
                tris: objects.triangle_buffer,
                triangle_range: (object.first_triangle, object.last_triangle),
                material_id: i as u32,
            };
            let ray = self.transform_by_obj_matrix(object.transform);
            let clamp = (0.00001, record.t);
            mesh.hit(&ray, clamp, &mut record);
        }

        if record.t == f32::INFINITY {
            let sky_material = BackgroundMaterial {};
            let stop_col = sky_material.get_stop_color(&record);
            color.x *= stop_col.x;
            color.y *= stop_col.y;
            color.z *= stop_col.z;
            return (
                RayReturn {
                    state: RayReturnState::Stop,
                    ray: Vec3::default(),
                },
                record,
            );
        }         

        record.ray.normalize();
        record.normal = record.normal.normalize();

        const MATERIAL_0: DiffuseMaterial = DiffuseMaterial::new(Vec3::new(230.0, 230.0, 0.0));
        const MATERIAL_1: MetalMaterial = MetalMaterial::new(Vec3::new(200.0, 200.0, 200.0), 0.5);

        let ray_return = if record.material_id == 0 {
            MATERIAL_0.get_next_ray_dir(&record, seed) //record.material.get_next_ray_dir(&record/*, rng*/);
        } else {
            MATERIAL_1.get_next_ray_dir(&record, seed)
        };
        match ray_return.state {
            RayReturnState::Absorb => *color = Vec3::default(),
            RayReturnState::Stop => {
                let stop_col = if record.material_id == 0 {
                    MATERIAL_0.get_stop_color(&record)
                } else {
                    MATERIAL_1.get_stop_color(&record)
                };
                color.x *= stop_col.x;
                color.y *= stop_col.y;
                color.z *= stop_col.z;
            } //record.material.get_stop_color(&record),
            RayReturnState::Ray => {
                if record.material_id == 0 {
                    *color = MATERIAL_0.get_color(&record, color)
                } else {
                    *color = MATERIAL_1.get_color(&record, color)
                }
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
    ) -> Vec3 {
        let mut color = Vec3::new(0.0, 0.0, 0.0);

        for _ in 0..data.samples {
            let mut curr_sample_color = Vec3::new(1.0, 1.0, 1.0);
            let mut vec = claculate_vec_dir_from_cam(
                data,
                (
                    pix_x as f32 + rand_float(&mut rng_seed, (0.0, 1.0)),
                    pix_y as f32 + rand_float(&mut rng_seed, (0.0, 1.0)),
                ),
            );
            vec.normalize();

            for _ in 0..data.depth {
                //depth
                let (ray_return, record) =
                    vec.trace_ray(scene_info, &mut rng_seed, &mut curr_sample_color, objects);
                match ray_return.state {
                    RayReturnState::Ray => {
                        vec = Ray::new(record.pos, ray_return.ray);
                    }
                    _ => {
                        break;
                    }
                }
            }

            color += curr_sample_color;
        }
        color.x /= data.samples as f32 * 256.0;
        color.y /= data.samples as f32 * 256.0;
        color.z /= data.samples as f32 * 256.0;
        color.x = color.x.sqrt().clamp(0.0, 0.999999999);
        color.y = color.y.sqrt().clamp(0.0, 0.999999999);
        color.z = color.z.sqrt().clamp(0.0, 0.999999999);
        color
    }

    fn transform_by_obj_matrix(&self, obj_matrix: glam::Affine3A) -> Self {
        let pos = obj_matrix.transform_point3(self.pos);
        let orientation = obj_matrix.transform_vector3(self.orientation);
        let orientation = orientation.normalize();
        Ray {
            pos,
            orientation,
        }
    }
}
