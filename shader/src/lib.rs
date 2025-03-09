#![no_std]
#![allow(clippy::type_complexity)]

use modules::ObjectInfo;
use shared::*;
#[allow(unused_imports)]
use spirv_std::glam::{vec2, vec4, Vec2, Vec4};
use spirv_std::spirv;
pub mod modules;
#[allow(unused_imports)]
use modules::material::*;

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] in_frag_coord: Vec4, //counts pixels, from 0 to canvas_width/canvas_height
    #[spirv(uniform, descriptor_set = 0, binding = 0)] data: &CamData,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] scene_info: &SceneInfo,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] vertex_buffer: &[Vertex],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] triangle_buffer: &[(u32, u32, u32)],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] object_buffer: &[Object],
    output: &mut Vec4,
) {
    let objects = ObjectInfo {
        vertex_buffer,
        triangle_buffer,
        object_buffer,
    };

    //-3 1.8 1

    //let first_obj = &object_buffer[0];

    //if first_obj.first_triangle == 0 {
    //    *output = Vec4::new(0.0, 0.0, 0.0, 1.0);
    //    return;
    //}
    //

    let seed: f32 = in_frag_coord.x
        + in_frag_coord.y * 255.0
        + in_frag_coord.z * 255.0 * 255.0
        + in_frag_coord.w * 255.0 * 255.0 * 255.0;

    let color = modules::trace::Ray::get_color(
        (in_frag_coord.x as usize, in_frag_coord.y as usize),
        seed as u32,
        data,
        scene_info,
        &objects,
    ) / 255.0; //tracer gives colors from 0 to 255

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
