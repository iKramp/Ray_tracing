#![no_std]

use shared::*;
#[allow(unused_imports)]
use spirv_std::glam::{vec2, vec4, Vec2, Vec4};
use spirv_std::spirv;
pub mod modules;
#[allow(unused_imports)]
use modules::{data::*, material::*};

#[spirv(fragment())]
pub fn main_fs(
    #[spirv(frag_coord)] in_frag_coord: Vec4,//counts pixels, from 0 to canvas_width/canvas_height
    #[spirv(push_constant)] shader_consts: &ShaderConstants,
    output: &mut Vec4,
) {


    let data = CamData {
        transform: PositionedVector3d {
            pos: Vector3d::new(shader_consts.pos_x as f64, shader_consts.pos_y as f64, shader_consts.pos_z as f64),
            orientation: Vector3d::new(shader_consts.orientation_x as f64, shader_consts.orientation_y as f64, shader_consts.orientation_z as f64),
        },
        fov: shader_consts.fov,
        canvas_width: shader_consts.canvas_width,
        canvas_height: shader_consts.canvas_height,
        samples: shader_consts.samples,
    };

    let color =
        modules::trace::Ray::get_color((in_frag_coord.x as usize, in_frag_coord.y as usize), &data) / 255.0; //tracer gives colors from 0 to 255

    *output = Vec4::new(color.x as f32, color.y as f32, color.z as f32, 1.0)
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
