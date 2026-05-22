#![no_std]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(unexpected_cfgs)]
#![feature(stmt_expr_attributes)]
#![feature(iter_map_windows)]

use glam::{UVec3, Vec3Swizzles};
use shared::glam::Vec3;
use shared::glam::Vec4Swizzles;
use shared::*;
#[allow(unused_imports)]
use spirv_std::glam::{vec2, vec4, Vec2, Vec4};
use spirv_std::image;
use spirv_std::spirv;

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use tracer::debug_points::check_points_proximity;
use tracer::modules::get_seed;
use tracer::modules::is_ray_nan;
use tracer::modules::is_vec_3_nan;
use tracer::modules::material::GenericMaterial;
use tracer::modules::material::RayReturnState;
use tracer::modules::rand_float;
use tracer::modules::trace::claculate_vec_dir_from_cam;
use tracer::modules::trace::Ray;
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
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] pixel_acc_buffer: &mut [Vec4],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] vertex_buffer: &[Vertex],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)] normal_buffer: &[Vec3Aligned],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 7)] uv_buffer: &[Vec2Aligned],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 8)] triangle_buffer: &[Face],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 9)] bvh_buffer: &[Bvh],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 10)] object_buffer: &[Object],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 11)] instance_buffer: &[Instance],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 12)] ray_buffer: &mut [Ray],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 13)]
    debug_points_array: &mut [Vec3Aligned],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 14)] image_info_buffer: &[ImageInfo],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 15)] image_data_buffer: &[u8],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 16)]
    material_buffer: &[GenericMaterial],
) {
    if id.x >= data.canvas_width || id.y >= data.canvas_height {
        // Out of bounds, skip processing.
        return;
    }

    let res = check_points_proximity(data, debug_points_array, id.xy());
    if !is_vec_3_nan(&res) {
        unsafe { res_output.write(id.xy(), Vec4::new(res.x, res.y, res.z, 1.0)) }
        return;
    }

    let mut seed = get_seed(id.x, id.y, data.random_seed);
    let coord_index = (id.x + id.y * data.canvas_width) as usize;

    if data.reset == 1 {
        pixel_acc_buffer[coord_index] = vec4(1.0, 1.0, 1.0, 0.0);
        ray_buffer[coord_index] = Ray::NAN;
        acc_buffer[coord_index] = vec4(0.0, 0.0, 0.0, 0.0);
        unsafe { res_output.write(id.xy(), Vec4::ZERO) };
    }

    let (ray, mut curr_color) = if !is_ray_nan(&ray_buffer[coord_index]) {
        (ray_buffer[coord_index], pixel_acc_buffer[coord_index])
    } else {
        let mut vec = claculate_vec_dir_from_cam(
            data,
            (
                id.x as f32 + rand_float(&mut seed, (0.0, 1.0)),
                id.y as f32 + rand_float(&mut seed, (0.0, 1.0)),
            ),
        );
        vec.normalize();
        (vec, vec4(1.0, 1.0, 1.0, 0.0))
    };

    let mut ret = tracer_main(
        ray,
        seed,
        curr_color.xyz(),
        data,
        scene_info,
        vertex_buffer,
        normal_buffer,
        uv_buffer,
        triangle_buffer,
        bvh_buffer,
        object_buffer,
        instance_buffer,
        image_info_buffer,
        material_buffer,
        debug_points_array,
    );

    if ret.0.x < f32::EPSILON * 10.0
        || ret.0.y < f32::EPSILON * 10.0
        || ret.0.z < f32::EPSILON * 10.0
    {
        ret.2 = RayReturnState::Stop;
        ret.0 = Vec3::ZERO;
    }

    let rendered_color = Vec4::new(ret.0.x, ret.0.y, ret.0.z, 1.0);
    curr_color = rendered_color;
    pixel_acc_buffer[coord_index] = curr_color;

    match ret.2 {
        RayReturnState::Killed => {
            ray_buffer[coord_index] = Ray::NAN;
            return;
        }
        RayReturnState::Ray => {
            ray_buffer[coord_index] = ret.1;
            return;
        }
        RayReturnState::Stop => {
            ray_buffer[coord_index] = Ray::NAN;
        }
    }

    let acc_color = acc_buffer[coord_index] + curr_color;
    acc_buffer[coord_index] = acc_color;

    let new_color = acc_color.xyz() / acc_color.w;

    //gamma correct
    let present_color = Vec4::new(
        new_color.x.powf(1.0 / 2.2),
        new_color.y.powf(1.0 / 2.2),
        new_color.z.powf(1.0 / 2.2),
        1.0,
    );

    unsafe { res_output.write(id.xy(), present_color) }
}
