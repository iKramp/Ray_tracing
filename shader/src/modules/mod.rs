pub mod data;
pub mod hit;
pub mod material;
pub mod trace;

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
    (*seed & 65535) as f32 * (range.1 - range.0) / 65535.0 + range.0
}
