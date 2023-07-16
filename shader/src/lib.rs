#![no_std]

use spirv_std::spirv;
use spirv_std::glam::{vec4, Vec4, Vec2, vec2};
use shared::*;
pub mod modules;
use modules::{data::*, material::*};

#[spirv(fragment())]
pub fn main_fs(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(push_constant)] shader_consts: &ShaderConstants,
    output: &mut Vec4,
) {
    let width = 1280.0;//shader_consts.width as f32;//commented until i get shader constants to work
    let height = 720.0;//shader_consts.height as f32;
    *output = Vec4::new(in_frag_coord.x / width, in_frag_coord.y / height, in_frag_coord.z / 255.0, in_frag_coord.w);
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_idx: i32,
    #[spirv(position)] builtin_pos: &mut Vec4,
    out_uv: &mut Vec2,
) {
    // Create a "full screen triangle" by mapping the vertex index.
    // ported from https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
    let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;

    *builtin_pos = pos.extend(0.0).extend(1.0);
    *out_uv = uv;
}