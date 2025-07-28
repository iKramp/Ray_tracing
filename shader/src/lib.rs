#![no_std]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(unexpected_cfgs)]
#![feature(stmt_expr_attributes)]

use glam::{UVec3, Vec3Swizzles};
use modules::ObjectInfo;
use shared::*;
#[allow(unused_imports)]
use spirv_std::glam::{vec2, vec4, Vec2, Vec4};
use spirv_std::{image::{ImageCoordinate, StorageImage2d}, spirv};
pub mod modules;
#[allow(unused_imports)]
use modules::material::*;

#[spirv(compute(threads(16, 16)))]
pub fn main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_image, descriptor_set = 0, binding = 7)] output: &mut StorageImage2d,

    #[spirv(uniform, descriptor_set = 0, binding = 0)] data: &CamData,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] scene_info: &SceneInfo,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] vertex_buffer: &[Vertex],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] triangle_buffer: &[(u32, u32, u32)],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] object_buffer: &[Object],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] instance_buffer: &[Instance],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)] bvh_buffer: &[Bvh],
) {
    let objects = ObjectInfo {
        vertex_buffer,
        triangle_buffer,
        object_buffer,
        instance_buffer,
        bvh_buffer,
    };

    if id.x >= data.canvas_width || id.y >= data.canvas_height {
        // Out of bounds, skip processing.
        return;
    }

    // let seed: f32 = in_frag_coord.x
    //     + in_frag_coord.y * 255.0
    //     + in_frag_coord.z * 255.0 * 255.0
    //     + in_frag_coord.w * 255.0 * 255.0 * 255.0;
    // let seed = seed as u32;
    let seed = id.x + id.y;
    // let seed = (data.frame * 13) ^ (in_frag_coord.x as u32 * 29) ^ (in_frag_coord.y as u32 * 67);

    let color = modules::trace::Ray::get_color(
        (id.x as usize, id.y as usize),
        seed,
        data,
        scene_info,
        &objects,
    );


    // output[out_coord as usize] = Vec4::new(color.x, color.y, color.z, 1.0)
    unsafe { output.write(id.xy(), color) }
}
