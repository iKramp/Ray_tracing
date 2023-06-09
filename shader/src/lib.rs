#![no_std]
use spirv_std::spirv;
use spirv_std::glam::{vec4, Vec4};

#[spirv(fragment())]
pub fn main_fs(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    output: &mut Vec4
    ) {
    output.x = in_frag_coord.x;
    output.y = in_frag_coord.y;
    output.z = in_frag_coord.z;
    output.w = in_frag_coord.w;
}