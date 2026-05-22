use shared::{
    glam::{Affine3A, Mat3},
    Bvh, Face, ImageInfo, Instance, Object, Vec2Aligned, Vec3Aligned, Vertex,
};

use crate::modules::{material::GenericMaterial, trace::Ray};

pub mod hit;
pub mod material;
pub mod trace;

pub fn get_seed(x: u32, y: u32, external_seed: u32) -> u32 {
    let mut h = external_seed;

    // Mix in coordinates with large odd constants
    h ^= x.wrapping_mul(0x85EB_CA77);
    h ^= y.wrapping_mul(0xC2B2_AE3D);

    // Avalanche (final mixing for good distribution)
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;

    h
}

pub fn xor_shift(seed: &mut u32) -> u32 {
    let mut x = *seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *seed = x;
    x
}

fn u32_to_f32_range(value: u32, range: (f32, f32)) -> f32 {
    let normalized = (value as f32) / (u32::MAX as f32 + 1.0);
    range.0 + normalized * (range.1 - range.0)
}

pub fn rand_float(seed: &mut u32, range: (f32, f32)) -> f32 {
    xor_shift(seed);
    u32_to_f32_range(*seed, range)
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
    pub normal_buffer: &'a [Vec3Aligned],
    pub uv_buffer: &'a [Vec2Aligned],
    pub triangle_buffer: &'a [Face],
    pub object_buffer: &'a [Object],
    pub instance_buffer: &'a [Instance],
    pub bvh_buffer: &'a [Bvh],
    pub image_info_buffer: &'a [ImageInfo],
    pub material_buffer: &'a [GenericMaterial],
}
