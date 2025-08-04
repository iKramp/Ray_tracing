use shared::{Bvh, Instance, Object, Vertex};

pub mod hit;
pub mod material;
pub mod trace;

pub fn get_seed(
    frag_coord: (usize, usize),
    frame: u32,
) -> u32 {
    (frame * 13) ^ (frag_coord.0 as u32 * 29) ^ (frag_coord.1 as u32 * 67)
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

pub struct ObjectInfo<'a> {
    pub vertex_buffer: &'a [Vertex],
    pub triangle_buffer: &'a [(u32, u32, u32)],
    pub object_buffer: &'a [Object],
    pub instance_buffer: &'a [Instance],
    pub bvh_buffer: &'a [Bvh],
}
