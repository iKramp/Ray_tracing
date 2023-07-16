#![no_std]
use spirv_std::spirv;
use spirv_std::glam::{vec4, Vec4};

#[spirv(fragment())]
pub fn main(
    //#[spirv(frag_coord)] in_frag_coord: Vec4,
    output: &mut Vec4,
    ) {

    *output = Vec4::new(1.0, 0.0, 0.0, 0.0)
    //*output = in_frag_coord.clone();
}