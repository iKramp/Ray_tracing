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
use spirv_std::image;
use spirv_std::spirv;
pub mod modules;
#[allow(unused_imports)]
use modules::material::*;

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::modules::get_seed;

#[spirv(compute(threads(16, 16)))]
pub fn main(
    #[spirv(global_invocation_id)] id: UVec3,

    #[spirv(uniform, descriptor_set = 0, binding = 0)] data: &CamData,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] scene_info: &SceneInfo,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] vertex_buffer: &[Vertex],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] triangle_buffer: &[(u32, u32, u32)],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] object_buffer: &[Object],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] instance_buffer: &[Instance],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)] bvh_buffer: &[Bvh],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 7)] acc_buffer: &mut [Vec4],

    // #[spirv(uniform_constant, descriptor_set = 0, binding = 7)] acc_output: &image::Image!(2D, sampled=false, __crate_root=crate, format=rgba32f),
    #[spirv(uniform_constant, descriptor_set = 0, binding = 8)] res_output: &image::Image!(
        2D,
        sampled = false,
        __crate_root = crate,
        format = rgba32f
    ),
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

    let coord_index = id.x + id.y * data.canvas_width;
    let prev_color = acc_buffer[coord_index as usize];

    let seed = get_seed(data.frame, id.x, id.y, prev_color.x, prev_color.y, prev_color.z);
    

    let rendered_color_3 = modules::trace::Ray::get_color(
        (id.x as usize, id.y as usize),
        seed,
        data,
        scene_info,
        &objects,
    );

    let nan = rendered_color_3.x > 1000.0 || 
        rendered_color_3.y > 1000.0 ||
        rendered_color_3.z > 1000.0;

    let rendered_color = Vec4::new(
        rendered_color_3.x,
        rendered_color_3.y,
        rendered_color_3.z,
        1.0,
    );


    let new_color;

    if data.frames_without_move < 0.5 {
        acc_buffer[coord_index as usize] = rendered_color;
        new_color = rendered_color;
    } else {
        let acc_color = if nan {
            Vec4::new(0.0, 1000000.0, 0.0, 1.0)
        } else {
            prev_color + rendered_color
        };

        acc_buffer[coord_index as usize] = acc_color;

        new_color = acc_color / (data.frames_without_move + 1.0);
    }

    //gamma correct
    let present_color = Vec4::new(
        new_color.x.powf(1.0 / 2.2),
        new_color.y.powf(1.0 / 2.2),
        new_color.z.powf(1.0 / 2.2),
        1.0,
    );

    unsafe { res_output.write(id.xy(), present_color) }
}
