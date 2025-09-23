#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::type_complexity)]
#![allow(unexpected_cfgs)]
#![allow(clippy::too_many_arguments)]
#![feature(stmt_expr_attributes)]


use shared::{glam::{UVec2, Vec3}, Bvh, CamData, Instance, Object, SceneInfo, Vertex};

use crate::modules::{get_seed, ObjectInfo};
pub mod modules;
pub mod debug_points;

pub fn tracer_main(
    id: UVec2,

    data: &CamData,
    scene_info: &SceneInfo,

    vertex_buffer: &[Vertex],
    triangle_buffer: &[(u32, u32, u32)],
    bvh_buffer: &[Bvh],
    object_buffer: &[Object],
    instance_buffer: &[Instance],

    debug_points_array: &mut [Vertex],
) -> Vec3 {
    let objects = ObjectInfo {
        vertex_buffer,
        triangle_buffer,
        object_buffer,
        instance_buffer,
        bvh_buffer,
    };

    if id.x >= data.canvas_width || id.y >= data.canvas_height {
        // Out of bounds, skip processing.
        return Vec3::ZERO;
    }

    let seed = get_seed(
        id.x,
        id.y,
        data.random_seed,
    );

    modules::trace::get_color(
        (id.x as usize, id.y as usize),
        seed,
        data,
        scene_info,
        &objects,
        debug_points_array
    )

}

pub fn trace_single_ray (
    px_coords: (u32, u32),

    data: &CamData,
    scene_info: &SceneInfo,

    vertex_buffer: &[Vertex],
    triangle_buffer: &[(u32, u32, u32)],
    bvh_buffer: &[Bvh],
    object_buffer: &[Object],
    instance_buffer: &[Instance],

    debug_points_array: &mut [Vertex],
) -> Vec3 {
    tracer_main(
        px_coords.into(),
        data,
        scene_info,
        vertex_buffer,
        triangle_buffer,
        bvh_buffer,
        object_buffer,
        instance_buffer,
        debug_points_array
    )
}

