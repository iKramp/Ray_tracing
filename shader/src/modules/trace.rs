use super::hit::*;
use super::material::*;
use super::rand_float;
use super::ObjectInfo;
use shared::glam::Vec3;
use shared::glam::Vec4;
use shared::BoundingBox;
use shared::CamData;
//use crate::Resources;
use core::f32::consts::PI;
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

const MATERIAL_0: MetalMaterial = MetalMaterial::new(Vec3::new(0.8, 0.8, 0.8), 0.15);
// const MATERIAL_1: DiffuseMaterial = DiffuseMaterial::new(Vec3::new(1.0, 1.0, 1.0));
const MATERIAL_1: NormalMaterial = NormalMaterial {};
const MATERIAL_2: EmmissiveMaterial = EmmissiveMaterial::new(Vec3::new(10.0, 10.0, 10.0));

pub fn claculate_vec_dir_from_cam(data: &CamData, (pix_x, pix_y): (f32, f32)) -> Ray {
    //fov is counted in degrees in the horizontal direction
    let fov = (data.fov * PI / 180.0) / 2.0;
    let edge_dist = fov.tan();
    let pix_x_dist = ((pix_x / data.canvas_width as f32) * 2.0 - 1.0) * edge_dist;
    let pix_y_dist = ((pix_y / data.canvas_height as f32) * 2.0 - 1.0) * edge_dist;
    let orientation_vec = Vec3::new(pix_x_dist, pix_y_dist, 1.0);
    let orientation_vec = data.transform.transform_vector3(orientation_vec);
    Ray::new(
        data.transform.transform_point3(Vec3::new(0.0, 0.0, 0.0)),
        orientation_vec,
    )
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
        cam_data: &CamData,
        color: &mut Vec3,
        objects: &ObjectInfo,
    ) -> RayReturnState {
        self.normalize();
        let mut record = HitRecord::new();

        for i in 0..scene_info.num_instances as usize {
            let instance = &objects.instance_buffer[i];
            let object = &objects.object_buffer[instance.object_id as usize];

            let mesh = Mesh {
                verts: objects.vertex_buffer,
                tris: objects.triangle_buffer,
                bvh_buffer: objects.bvh_buffer,
                material_id: i as u32,
                bvh_root: object.bvh_root,
            };
            let inverse_matrix = instance.transform.inverse();
            let ray = Ray {
                pos: inverse_matrix.transform_point3(self.pos),
                orientation: inverse_matrix.transform_vector3(self.orientation),
            };

            let clamp = (0.00001, record.t);
            mesh.hit(&ray, clamp, &mut record, i as u32, get_backface_culling(i as u32));
        }

        #[cfg(feature = "debug")]
        if cam_data.debug_information == shared::DebugInformation::TriangleIntersection {
            if record.triangle_tests > cam_data.debug_number {
                *color = Vec3::new(1.0, 0.0, 0.0);
            } else {
                let color_ = Vec3::new(
                    record.triangle_tests as f32 / cam_data.debug_number as f32,
                    record.triangle_tests as f32 / cam_data.debug_number as f32,
                    record.triangle_tests as f32 / cam_data.debug_number as f32,
                );
                *color = color_;
            }
            return RayReturnState::Stop;
        }

        #[cfg(feature = "debug")]
        if cam_data.debug_information == shared::DebugInformation::BvhIntersection {
            if record.box_tests > cam_data.debug_number {
                *color = Vec3::new(1.0, 0.0, 0.0);
            } else {
                let color_ = Vec3::new(
                    record.box_tests as f32 / cam_data.debug_number as f32,
                    record.box_tests as f32 / cam_data.debug_number as f32,
                    record.box_tests as f32 / cam_data.debug_number as f32,
                );
                *color = color_;
            }
            return RayReturnState::Stop;
        }

        if record.t == f32::INFINITY {
            let sky_material = BackgroundMaterial {};

            let stop_col =
                sky_material.get_stop_color(self.orientation, (0.0, 0.0), self.orientation);
            *color = stop_col;

            return RayReturnState::Stop;
        }

        let instance = &objects.instance_buffer[record.instance_id as usize];
        let transform = &instance.transform;

        let triangle = {
            let tmp_tri = objects.triangle_buffer[record.triangle_id as usize];
            let mut vert_1 = objects.vertex_buffer[tmp_tri.0 as usize].clone();
            let mut vert_2 = objects.vertex_buffer[tmp_tri.1 as usize].clone();
            let mut vert_3 = objects.vertex_buffer[tmp_tri.2 as usize].clone();
            vert_1.pos = transform.transform_point3(vert_1.pos);
            vert_2.pos = transform.transform_point3(vert_2.pos);
            vert_3.pos = transform.transform_point3(vert_3.pos);
            (vert_1, vert_2, vert_3)
        };
        let material_id = record.instance_id as usize;
        let mut ray = *self;
        ray.orientation *= record.t;

        let normal = {
            let a = triangle.0.pos - triangle.1.pos;
            let b = triangle.0.pos - triangle.2.pos;
            a.cross(b).normalize()
        };

        let uv = (0.0, 0.0);

        let ray_return = if material_id == 1 {
            MATERIAL_1.get_next_ray_dir(seed, ray, normal)
        } else if material_id == 2 {
            MATERIAL_2.get_next_ray_dir(seed, ray, normal)
        } else {
            MATERIAL_0.get_next_ray_dir(seed, ray, normal)
        };

        match ray_return.state {
            RayReturnState::Absorb => *color = Vec3::default(),
            RayReturnState::Stop => {
                let stop_col = if material_id == 1 {
                    MATERIAL_1.get_stop_color(normal, uv, ray.orientation)
                } else if material_id == 2 {
                    MATERIAL_2.get_stop_color(normal, uv, ray.orientation)
                } else {
                    MATERIAL_0.get_stop_color(normal, uv, ray.orientation)
                };

                color.x *= stop_col.x;
                color.y *= stop_col.y;
                color.z *= stop_col.z;
            }
            RayReturnState::Ray => {
                if material_id == 1 {
                    *color = MATERIAL_1.get_color(*color, normal, uv, ray.orientation);
                } else if material_id == 2 {
                    *color = MATERIAL_2.get_color(*color, normal, uv, ray.orientation);
                } else {
                    *color = MATERIAL_0.get_color(*color, normal, uv, ray.orientation);
                }
            }
        }

        self.pos += self.orientation * record.t;
        self.orientation = ray_return.direction.normalize();

        ray_return.state
    }

    pub fn get_color(
        (pix_x, pix_y): (usize, usize),
        mut rng_seed: u32,
        data: &CamData,
        scene_info: &shared::SceneInfo,
        objects: &ObjectInfo,
    ) -> Vec3 {
        let mut color = Vec3::new(1.0, 1.0, 1.0);

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
            let ray_return = vec.trace_ray(scene_info, &mut rng_seed, data, &mut color, objects);

            match ray_return {
                RayReturnState::Ray => {}
                _ => {
                    return color;
                }
            }
        }
        //never finished bouncing
        Vec3::default()
    }

    pub(super) fn hits_bounding(&self, bounding_box: &BoundingBox) -> bool {
        let mut t_min = (bounding_box.min - self.pos) / self.orientation;
        let mut t_max = (bounding_box.max - self.pos) / self.orientation;

        if t_min.x > t_max.x {
            core::mem::swap(&mut t_min.x, &mut t_max.x);
        }
        if t_min.y > t_max.y {
            core::mem::swap(&mut t_min.y, &mut t_max.y);
        }
        if t_min.z > t_max.z {
            core::mem::swap(&mut t_min.z, &mut t_max.z);
        }

        let t_near = f32::max(t_min.x, f32::max(t_min.y, t_min.z));
        let t_far = f32::min(t_max.x, f32::min(t_max.y, t_max.z));

        if t_near < f32::INFINITY && t_near < t_far {
            return true;
        }
        false
    }
}

fn get_backface_culling(
    instance_id: u32,
) -> bool {
    if instance_id == 0 {
        MATERIAL_0.backface_culling()
    } else if instance_id == 1 {
        MATERIAL_1.backface_culling()
    } else if instance_id == 2 {
        MATERIAL_2.backface_culling()
    } else {
        true // Default to true for other materials
    }
}
