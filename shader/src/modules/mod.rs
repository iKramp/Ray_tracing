use shared::{glam::{Affine3A, Mat3}, Bvh, Instance, Object, Vertex};

use crate::modules::trace::Ray;

pub mod hit;
pub mod material;
pub mod trace;

pub fn get_seed(
    frame: u32,
    x: u32,
    y: u32,
    prev_r: f32,
    prev_g: f32,
    prev_b: f32,
) -> u32 {
    let mut h = 0;

    // mix in all components with relatively large odd constants
    h ^= frame.wrapping_mul(0x9E3779B9);
    h ^= x.wrapping_mul(0x85EBCA77);
    h ^= y.wrapping_mul(0xC2B2AE3D);
    h ^= prev_r.to_bits().wrapping_mul(0x27D4EB2F);
    h ^= prev_g.to_bits().wrapping_mul(0x165667B1);
    h ^= prev_b.to_bits().wrapping_mul(0x7F4A7C15);

    // avalanche
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846CA68B);
    h ^= h >> 16;

    h
}

pub fn xor_shift(seed: u32) -> u32 {
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

pub fn rand_float(seed: &mut u32, range: (f32, f32)) -> f32 {
    let num = xor_shift(*seed);
    *seed = num;
    (*seed & 65535) as f32 / 65535.0 * (range.1 - range.0) + range.0
}

pub fn is_nan(value: f32) -> bool {
    //bitmask because actual checks are optimized out by the compiler
    let bitmask = value.to_bits();
    let nan = (bitmask & 0x7F800000) == 0x7F800000 && (bitmask & 0x007FFFFF) != 0;
    let inf = (bitmask & 0x7F800000) == 0x7F800000 && (bitmask & 0x007FFFFF) == 0;
    nan || inf
}

pub fn is_inf(value: f32) -> bool {
    let value = value.to_bits();
    value == 0x7F800000 || value == 0xFF800000 // +inf or -inf
}

pub fn is_vec_3_nan(vec: &spirv_std::glam::Vec3) -> bool {
    is_nan(vec.x) || is_nan(vec.y) || is_nan(vec.z)
}

pub fn is_ray_nan(ray: &Ray) -> bool {
    is_vec_3_nan(&ray.pos) || is_vec_3_nan(&ray.orientation)
}

pub fn is_mat3_nan(mat: Mat3) -> bool {
    is_vec_3_nan(&mat.x_axis) || is_vec_3_nan(&mat.y_axis) || is_vec_3_nan(&mat.z_axis)
}

pub fn is_aff3a_nan(mat: &Affine3A) -> bool {
    is_vec_3_nan(&mat.translation.into()) || is_mat3_nan(mat.matrix3.into())   
}

pub struct ObjectInfo<'a> {
    pub vertex_buffer: &'a [Vertex],
    pub triangle_buffer: &'a [(u32, u32, u32)],
    pub object_buffer: &'a [Object],
    pub instance_buffer: &'a [Instance],
    pub bvh_buffer: &'a [Bvh],
}
