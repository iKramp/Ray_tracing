#![no_std]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(unexpected_cfgs)]
#![feature(stmt_expr_attributes)]
#![feature(iter_map_windows)]

use glam::{UVec3, Vec3Swizzles};
use shared::*;
#[allow(unused_imports)]
use spirv_std::glam::{vec2, vec4, Vec2, Vec4};
use spirv_std::image;
use spirv_std::spirv;

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use tracer::debug_points::check_points_proximity;
use tracer::modules::is_vec_3_nan;
use tracer::tracer_main;


#[spirv(compute(threads(16, 16)))]
pub fn render_pixel(
    #[spirv(global_invocation_id)] id: UVec3,

    #[spirv(uniform, descriptor_set = 0, binding = 0)] data: &CamData,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] scene_info: &SceneInfo,

    #[spirv(uniform_constant, descriptor_set = 0, binding = 2)] res_output: &image::Image!(
        2D,
        sampled = false,
        __crate_root = crate,
        format = rgba32f
    ),

    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] acc_buffer: &mut [Vec4],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] vertex_buffer: &[Vertex],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] triangle_buffer: &[(u32, u32, u32)],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)] bvh_buffer: &[Bvh],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 7)] object_buffer: &[Object],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 8)] instance_buffer: &[Instance],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 9)] debug_points_array: &mut [Vertex],
) {

    let res = check_points_proximity(data, debug_points_array, id.xy());
    if !is_vec_3_nan(&res) {
        unsafe { res_output.write(id.xy(), Vec4::new(res.x, res.y, res.z, 1.0)) }
        return;
    }

    let rendered_color = tracer_main(
        id.xy(),
        data,
        scene_info,
        vertex_buffer,
        triangle_buffer,
        bvh_buffer,
        object_buffer,
        instance_buffer,
        debug_points_array,
    );

    let nan =
        rendered_color.x > 1000.0 || rendered_color.y > 1000.0 || rendered_color.z > 1000.0;

    let rendered_color = Vec4::new(
        rendered_color.x,
        rendered_color.y,
        rendered_color.z,
        1.0,
    );

    let new_color;
    let coord_index = id.x + id.y * data.canvas_width;
    let prev_color = acc_buffer[coord_index as usize];

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
