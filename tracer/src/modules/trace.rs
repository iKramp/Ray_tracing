#![allow(unexpected_cfgs)]

use crate::modules::is_inf;
use crate::modules::is_nan;
use crate::modules::is_vec_3_nan;
use crate::modules::xor_shift;

use super::hit::*;
use super::material::*;
use super::rand_float;
use super::ObjectInfo;
use shared::glam::Affine3A;
use shared::glam::Vec2;
use shared::glam::Vec3;
use shared::glam::Vec4;
use shared::BoundingBox;
use shared::CamData;
use shared::Vec2Aligned;
use shared::Vec3Aligned;
use shared::Vertex;
//use crate::Resources;
use core::f32::consts::PI;
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

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

fn solid_angle(point: Vec3, tri_a: Vec3, tri_b: Vec3, tri_c: Vec3) -> f32 {
    let a = (tri_a - point).normalize();
    let b = (tri_b - point).normalize();
    let c = (tri_c - point).normalize();

    let numerator = a.dot(b.cross(c)).abs();
    let denominator = 1.0 + a.dot(b) + a.dot(c) + b.dot(c);
    let half_angle = numerator.atan2(denominator);
    2.0 * half_angle
}

fn sphere_fraction(origin: Vec3, a: Vec3, b: Vec3, c: Vec3) -> f32 {
    solid_angle(origin, a, b, c) / (4.0 * PI)
}

//returns
//(color contribution, new ray, whether to stop or continue, direct color contribution for mis)
pub fn get_color(
    mut in_ray: Ray,
    mut rng_seed: u32,
    mut color: Vec3,
    data: &CamData,
    scene_info: &shared::SceneInfo,
    objects: &ObjectInfo,
    debug_points_array: &mut [Vec3Aligned],
) -> (Vec3, Ray, RayReturnState, Vec3) {
    let luminance = color.x.max(color.y).max(color.z);
    let probability = (0.5 + luminance / 2.0).clamp(0.0, 1.0);
    let rand_val = rand_float(&mut rng_seed, (0.0, 1.0));
    if rand_val > probability {
        return (Vec3::ZERO, in_ray, RayReturnState::Stop, Vec3::NAN);
    } else {
        color /= probability;
    }

    in_ray.pos += in_ray.orientation * f32::EPSILON * 100.0;
    let (ray_return, direct_cotrib) =
        in_ray.trace_ray(scene_info, &mut rng_seed, data, &mut color, objects);

    if data.debug_information == shared::DebugInformation::RecordPoints {
        let mut i = 1;
        loop {
            if is_vec_3_nan(&debug_points_array[i as usize + 1]) {
                debug_points_array[i as usize] = Vec3Aligned::new(in_ray.pos);
                debug_points_array[i as usize + 1] =
                    Vec3Aligned::new(in_ray.pos + in_ray.orientation * 100.0);
                break;
            }
            i += 1;
        }
    }

    match ray_return {
        RayReturnState::Killed => (Vec3::ZERO, in_ray, ray_return, Vec3::NAN),
        _ => (color, in_ray, ray_return, direct_cotrib),
    }
}

fn get_normal_uv_from_vertex_data(
    hit: Vec3,
    triangle: (Vertex, Vertex, Vertex),
    normals: (Vec3, Vec3, Vec3),
    uv: (Vec2Aligned, Vec2Aligned, Vec2Aligned),
) -> (Vec3, Vec2) {
    let v0 = triangle.1.pos - triangle.0.pos;
    let v1 = triangle.2.pos - triangle.0.pos;
    let v2 = hit - triangle.0.pos;

    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);

    let denom = d00 * d11 - d01 * d01;
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;

    let normal = (normals.0 * u + normals.1 * v + normals.2 * w).normalize();
    let uv = *uv.0 * u + *uv.1 * v + *uv.2 * w;
    (normal, uv)
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

    //returns the new ray, and direct color contribution (for direct lighting)
    pub fn trace_ray(
        &mut self,
        scene_info: &shared::SceneInfo,
        seed: &mut u32,
        cam_data: &CamData,
        color: &mut Vec3,
        objects: &ObjectInfo,
    ) -> (RayReturnState, Vec3) {
        self.normalize();

        let record = self.find_closest(scene_info, objects, (0.0, u32::MAX, 0));

        #[cfg(feature = "debug")]
        let res = Self::record_debug(cam_data, &record, color);
        if let Some(res) = res {
            return (res, Vec3::NAN);
        }

        if record.t == f32::INFINITY {
            self.hit_sky(color);
            return (RayReturnState::Stop, Vec3::NAN);
        }

        let instance = &objects.instance_buffer[record.instance_id as usize];
        let transform = &instance.transform;

        let (shading_normal, geometry_normal, uv) =
            self.calculate_normal_uv(&record, objects, transform);

        let object = &objects.object_buffer[instance.object_id as usize];
        let material_id = object.material_id as usize;
        let ray = *self;

        let num_materials = scene_info.num_materials;
        let material_buffer = objects.material_buffer;

        let mat_return = if material_id < num_materials as usize {
            material_buffer[material_id].bxdf(*color, ray, shading_normal, uv, record.t, seed)
        } else {
            *color = Vec3::new(0.0, 1000.0, 0.0);
            return (RayReturnState::Stop, Vec3::NAN);
        };

        let mut direct_contrib = Vec3::NAN;

        if matches!(mat_return.ray_return_state, RayReturnState::Ray) {
            let (direct_contrib_color, _direct_contrib_dir) = Self::mis_direct_sample(
                ray,
                record.t,
                scene_info,
                objects,
                shading_normal,
                geometry_normal,
                mat_return.next_color,
                seed,
            );
            direct_contrib = direct_contrib_color;
        }

        *self = mat_return.new_ray;
        *color = mat_return.next_color;

        (mat_return.ray_return_state, direct_contrib)
    }

    //returns the color contribution and outgoing vector
    fn mis_direct_sample(
        incoming_ray: Ray,
        t: f32,
        scene_info: &shared::SceneInfo,
        objects: &ObjectInfo,
        shading_normal: Vec3,
        geometry_normal: Vec3,
        curr_color: Vec3,
        seed: &mut u32,
    ) -> (Vec3, Ray) {
        const ILLEGAL: Vec3 = Vec3::ZERO;
        let num_emissive = scene_info.num_emissive_instances;
        let instance_to_check = xor_shift(seed) % num_emissive;
        let instance = &objects.instance_buffer[instance_to_check as usize];

        let object = &objects.object_buffer[instance.object_id as usize];
        let num_triangles = object.num_triangles;
        let triangle_to_check = xor_shift(seed) % num_triangles;
        let triangle_id = object.triangle_start + triangle_to_check;
        let tri = &objects.triangle_buffer[triangle_id as usize];
        let (u, v) = {
            let rand_u = rand_float(seed, (0.0, 1.0));
            let rand_v = rand_float(seed, (0.0, 1.0));
            if rand_u + rand_v > 1.0 {
                (1.0 - rand_u, 1.0 - rand_v)
            } else {
                (rand_u, rand_v)
            }
        };

        let a = instance
            .transform
            .transform_point3(objects.vertex_buffer[tri.vert.x as usize].pos);
        let b = instance
            .transform
            .transform_point3(objects.vertex_buffer[tri.vert.y as usize].pos);
        let c = instance
            .transform
            .transform_point3(objects.vertex_buffer[tri.vert.z as usize].pos);
        let ab = a - b;
        let ac = a - c;

        let light_normal = ab.cross(ac).normalize();
        let light_point = a * (1.0 - u - v) + b * u + c * v;

        let start_point = incoming_ray.pos + incoming_ray.orientation * t;

        let start_point = Self::shadow_terminator_offset(
            start_point,
            shading_normal,
            geometry_normal,
            light_point - start_point,
        );

        let mut light_ray = Ray::new(start_point, light_point - start_point);

        //check for invalid, through current triangle
        if shading_normal.dot(light_ray.orientation) < f32::EPSILON {
            return (ILLEGAL, light_ray);
        }

        let distance = light_ray.orientation.length();
        light_ray.normalize();

        let record = light_ray.find_closest(
            scene_info,
            objects,
            (distance, triangle_id, instance_to_check),
        );

        let same_tri = record.triangle_id == triangle_id && record.instance_id == instance_to_check;

        if !same_tri || distance < 0.5 {
            return (ILLEGAL, light_ray);
        }

        let material = &objects.material_buffer[object.material_id as usize];

        let direct_light = material.emissive_color() * curr_color;

        let d2 = distance * distance;
        let g = light_normal.dot(-light_ray.orientation).abs() / d2;

        let direct_light = direct_light * g * shading_normal.dot(light_ray.orientation).abs();

        let area = ab.cross(ac).length() / 2.0;
        let pdf = 1.0 / (num_emissive as f32 * num_triangles as f32 * area);

        (direct_light / pdf * PI / 2.0, light_ray)
    }

    fn shadow_terminator_offset(
        start: Vec3,
        shading_normal: Vec3,
        geo_normal: Vec3,
        light_dir: Vec3,
    ) -> Vec3 {
        // Project start onto the shading plane defined by each vertex normal
        // then interpolate — but a cheaper approximation:
        let w = light_dir.dot(shading_normal);
        let a = light_dir - w * shading_normal; // tangential component
        let new_dir = (light_dir + a * (1.0 - geo_normal.dot(shading_normal).max(0.0))).normalize();
        // bend the ray so it can't graze below the geometric surface
        start + new_dir * f32::EPSILON * 1000.0
    }

    fn find_closest(
        &self,
        scene_info: &shared::SceneInfo,
        objects: &ObjectInfo,
        short_circuit: (f32, u32, u32), //distance, triangle, instance
    ) -> HitRecord {
        let mut record = HitRecord::new();
        let is_short_circuit = short_circuit.1 != u32::MAX;
        if is_short_circuit {
            record.t = short_circuit.0 - f32::EPSILON * 1000.0 * short_circuit.0;
            record.triangle_id = short_circuit.1;
            record.instance_id = short_circuit.2;
        }

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
            let num_materials = scene_info.num_materials;
            let material_buffer = objects.material_buffer;

            mesh.hit(
                &ray,
                clamp,
                &mut record,
                i as u32,
                get_backface_culling(i as u32, num_materials, material_buffer) || is_short_circuit,
                is_short_circuit,
            );
        }

        record
    }

    fn hit_sky(&mut self, color: &mut Vec3) {
        let sky_material = BackgroundMaterial {};

        let stop_col = sky_material.get_stop_color(self.orientation, (0.0, 0.0), self.orientation);
        *color = stop_col;

        *self = Ray::new(self.pos + self.orientation * 1000.0, Vec3::ZERO);
    }

    //returns shading normal, geometry normal, uv
    fn calculate_normal_uv(
        &self,
        record: &HitRecord,
        objects: &ObjectInfo,
        transform: &Affine3A,
    ) -> (Vec3, Vec3, Vec2) {
        let tmp_tri = &objects.triangle_buffer[record.triangle_id as usize];
        let mut vert_0 = objects.vertex_buffer[tmp_tri.vert.x as usize];
        let mut vert_1 = objects.vertex_buffer[tmp_tri.vert.y as usize];
        let mut vert_2 = objects.vertex_buffer[tmp_tri.vert.z as usize];
        vert_0.pos = transform.transform_point3(vert_0.pos);
        vert_1.pos = transform.transform_point3(vert_1.pos);
        vert_2.pos = transform.transform_point3(vert_2.pos);

        let geometry_normal = (vert_1.pos - vert_0.pos)
            .cross(vert_2.pos - vert_0.pos)
            .normalize();

        let tmp_normals = if tmp_tri.normal.x != u32::MAX {
            (
                transform.transform_vector3(*objects.normal_buffer[tmp_tri.normal.x as usize]),
                transform.transform_vector3(*objects.normal_buffer[tmp_tri.normal.y as usize]),
                transform.transform_vector3(*objects.normal_buffer[tmp_tri.normal.z as usize]),
            )
        } else {
            (geometry_normal, geometry_normal, geometry_normal)
        };

        let tmp_uv = if tmp_tri.uv.x != u32::MAX {
            (
                objects.uv_buffer[tmp_tri.uv.x as usize],
                objects.uv_buffer[tmp_tri.uv.y as usize],
                objects.uv_buffer[tmp_tri.uv.z as usize],
            )
        } else {
            (
                Vec2Aligned::new(Vec2::ZERO),
                Vec2Aligned::new(Vec2::ZERO),
                Vec2Aligned::new(Vec2::ZERO),
            )
        };

        let (normal, uv) = get_normal_uv_from_vertex_data(
            self.pos + self.orientation * record.t,
            (vert_0, vert_1, vert_2),
            tmp_normals,
            tmp_uv,
        );

        (normal, geometry_normal, uv)
    }

    #[cfg(feature = "debug")]
    fn record_debug(
        cam_data: &CamData,
        record: &HitRecord,
        color: &mut Vec3,
    ) -> Option<RayReturnState> {
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
            return Some(RayReturnState::Stop);
        }

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
            return Some(RayReturnState::Stop);
        }

        None
    }

    pub(super) fn hits_bounding(&self, bounding_box: &BoundingBox) -> f32 {
        let mut t_min = (bounding_box.min() - self.pos) / self.orientation;
        let mut t_max = (bounding_box.max() - self.pos) / self.orientation;

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

        if t_near < f32::INFINITY
            && (t_near < t_far || f32::abs(t_near - t_far) < f32::EPSILON)
            && t_far > 0.0
        {
            return t_near;
        }
        f32::INFINITY
    }
}

fn get_backface_culling(
    material_id: u32,
    num_materials: u32,
    material_buffer: &[GenericMaterial],
) -> bool {
    if material_id < num_materials {
        return material_buffer[material_id as usize].backface_culling();
    }
    true
}
