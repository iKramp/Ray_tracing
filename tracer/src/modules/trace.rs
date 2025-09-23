#![allow(unexpected_cfgs)]

use crate::modules::is_vec_3_nan;

use super::hit::*;
use super::material::*;
use super::rand_float;
use super::ObjectInfo;
use shared::glam::Vec3;
use shared::glam::Vec4;
use shared::BoundingBox;
use shared::CamData;
use shared::Vertex;
//use crate::Resources;
use core::f32::consts::PI;
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

// const MATERIAL_0: RefractiveMaterial = RefractiveMaterial::new(Vec3::new(0.9, 0.9, 0.9), 1.33);
const MATERIAL_0: GenericMaterial = GenericMaterial {
    color: Vec3::new(1.0, 1.0, 1.0),
    specular: 0.0,
    specular_roughness: 0.0,
    roughness: 0.0,
    ior: 1.5,
};
const MATERIAL_1: NormalMaterial = NormalMaterial {};
const MATERIAL_2: EmmissiveMaterial = EmmissiveMaterial::new(Vec3::new(15.0, 15.0, 15.0));

pub fn claculate_vec_dir_from_cam(data: &CamData, (pix_x, pix_y): (f32, f32)) -> Ray {
    //fov is counted in degrees in the horizontal direction
    let fov = (data.fov * PI / 180.0) / 2.0;
    let edge_dist = fov.tan();
    let pix_x_frac = (pix_x / data.canvas_width as f32) * 2.0 - 1.0;
    let pix_y_frac = (pix_y / data.canvas_height as f32) * 2.0 - 1.0;
    let pix_y_frac_adjusted = pix_y_frac * (data.canvas_height as f32 / data.canvas_width as f32);
    let pix_x_dist = pix_x_frac * edge_dist;
    let pix_y_dist = pix_y_frac_adjusted * edge_dist;
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

pub fn get_color(
    mut in_ray: Ray,
    mut rng_seed: u32,
    mut color: Vec3,
    data: &CamData,
    scene_info: &shared::SceneInfo,
    objects: &ObjectInfo,
    debug_points_array: &mut [Vertex],
) -> (Vec3, Ray, RayReturnState) {
    let mut iterator = 0;
    loop {
        let luminance = color.x.max(color.y).max(color.z);
        let probability = (0.5 + luminance / 2.0).clamp(0.0, 1.0);
        let rand_val = rand_float(&mut rng_seed, (0.0, 1.0));
        if rand_val > probability {
            return (Vec3::ZERO, in_ray, RayReturnState::Stop);
        } else {
            color /= probability;
        }


        in_ray.pos += in_ray.orientation * f32::EPSILON * 100.0;
        let ray_return = in_ray.trace_ray(scene_info, &mut rng_seed, data, &mut color, objects);

        if data.debug_information == shared::DebugInformation::RecordPoints {
            let mut i = 1;
            loop {
                if is_vec_3_nan(&debug_points_array[i as usize + 1].pos) {
                    debug_points_array[i as usize] = Vertex::new(in_ray.pos);
                    debug_points_array[i as usize + 1] = Vertex::new(in_ray.pos + in_ray.orientation * 100.0);
                    break;
                }
                i += 1;
            }
        }
        iterator += 1;

        match ray_return {
            RayReturnState::Killed => {
                return (Vec3::ZERO, in_ray, ray_return);
            },
            RayReturnState::Stop => {
                return (color, in_ray, ray_return);
            }
            RayReturnState::Ray => {
                if iterator > data.depth {
                    return (color, in_ray, RayReturnState::Ray);
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Ray {
    pub pos: Vec3,
    #[cfg(not(target_arch = "spirv"))]
    _padding: [u8; 4],
    pub orientation: Vec3,
    #[cfg(not(target_arch = "spirv"))]
    _padding_2: [u8; 4],
}

impl Ray {
    pub const NAN: Ray = Ray {
        pos: Vec3::NAN,
        #[cfg(not(target_arch = "spirv"))]
        _padding: [0; 4],
        
        orientation: Vec3::NAN,

        #[cfg(not(target_arch = "spirv"))]
        _padding_2: [0; 4],
    };

    pub fn new(pos: Vec3, orientation: Vec3) -> Self {
        Ray { 
            pos, 
            #[cfg(not(target_arch = "spirv"))]
            _padding: [0; 4],
            orientation,
            #[cfg(not(target_arch = "spirv"))]
            _padding_2: [0; 4],
        }
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
                #[cfg(not(target_arch = "spirv"))]
                _padding: [0; 4],

                orientation: inverse_matrix.transform_vector3(self.orientation),
                #[cfg(not(target_arch = "spirv"))]
                _padding_2: [0; 4],
            };

            let clamp = (f32::EPSILON, record.t);
            mesh.hit(
                &ray,
                clamp,
                &mut record,
                i as u32,
                get_backface_culling(i as u32),
            );
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

            *self = Ray::new(self.pos + self.orientation * 1000.0, Vec3::ZERO);

            return RayReturnState::Stop;
        }

        let instance = &objects.instance_buffer[record.instance_id as usize];
        let transform = &instance.transform;

        let triangle = {
            let tmp_tri = objects.triangle_buffer[record.triangle_id as usize];
            let mut vert_1 = objects.vertex_buffer[tmp_tri.0 as usize];
            let mut vert_2 = objects.vertex_buffer[tmp_tri.1 as usize];
            let mut vert_3 = objects.vertex_buffer[tmp_tri.2 as usize];
            vert_1.pos = transform.transform_point3(vert_1.pos);
            vert_2.pos = transform.transform_point3(vert_2.pos);
            vert_3.pos = transform.transform_point3(vert_3.pos);
            (vert_1, vert_2, vert_3)
        };
        let material_id = record.instance_id as usize;
        let ray = *self;

        let normal = {
            let a = triangle.0.pos - triangle.1.pos;
            let b = triangle.0.pos - triangle.2.pos;
            a.cross(b).normalize()
        };


        let uv = (0.0, 0.0);

        let mat_return = if material_id == 0 {
            MATERIAL_0.bxdf(*color, ray, normal, uv, record.t, seed)
        } else if material_id == 1 {
            MATERIAL_1.bxdf(*color, ray, normal, uv, record.t, seed)
        } else {
            MATERIAL_2.bxdf(*color, ray, normal, uv, record.t, seed)
        };

        *self = mat_return.new_ray;
        *color = mat_return.next_color;

        #[cfg(not(target_arch = "spirv"))]
        {
            //debug normal, hit point, triangle, color
            println!(
                "Hit triangle: {:?}, normal: {:?}, at point: {:?}, t: {}",
                record.triangle_id, normal, ray.pos + ray.orientation * record.t, record.t
            );
            println!("Color after hit: {color:?}");
            
        }

        mat_return.ray_return_state
    }

    pub(super) fn hits_bounding(&self, bounding_box: &BoundingBox) -> f32 {
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

        if t_near < f32::INFINITY && t_near < t_far && t_far > 0.0 {
            return t_near;
        }
        f32::INFINITY
    }
}

fn get_backface_culling(instance_id: u32) -> bool {
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
