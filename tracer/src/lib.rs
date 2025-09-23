#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::type_complexity)]
#![allow(unexpected_cfgs)]
#![allow(clippy::too_many_arguments)]
#![feature(stmt_expr_attributes)]

use shared::{
    glam::{UVec2, Vec3},
    Bvh, CamData, Instance, Object, SceneInfo, Vertex,
};

use crate::modules::{
    get_seed, material::RayReturnState, rand_float, trace::{claculate_vec_dir_from_cam, Ray}, xor_shift, ObjectInfo
};
pub mod debug_points;
pub mod modules;

pub fn tracer_main(
    trace_ray: Ray,
    rand_seed: u32,
    color: Vec3,

    data: &CamData,
    scene_info: &SceneInfo,

    vertex_buffer: &[Vertex],
    triangle_buffer: &[(u32, u32, u32)],
    bvh_buffer: &[Bvh],
    object_buffer: &[Object],
    instance_buffer: &[Instance],

    debug_points_array: &mut [Vertex],
) -> (Vec3, Ray, RayReturnState) {
    let objects = ObjectInfo {
        vertex_buffer,
        triangle_buffer,
        object_buffer,
        instance_buffer,
        bvh_buffer,
    };

    modules::trace::get_color(
        trace_ray,
        rand_seed,
        color,
        data,
        scene_info,
        &objects,
        debug_points_array,
    )
}

pub fn trace_single_ray(
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
    let mut seed = get_seed(px_coords.0, px_coords.1, data.random_seed);

    let mut vec = claculate_vec_dir_from_cam(
        data,
        (
            px_coords.0 as f32 + rand_float(&mut seed, (0.0, 1.0)),
            px_coords.1 as f32 + rand_float(&mut seed, (0.0, 1.0)),
        ),
    );

    if data.debug_information == shared::DebugInformation::RecordPoints {
        debug_points_array[0] = Vertex::new(vec.pos);
        debug_points_array[1] = Vertex::new(vec.pos + vec.orientation * 100.0);
    }

    let mut acc_color = Vec3::ONE;

    loop {
        let res = tracer_main(
            vec,
            seed,
            Vec3::ONE,
            data,
            scene_info,
            vertex_buffer,
            triangle_buffer,
            bvh_buffer,
            object_buffer,
            instance_buffer,
            debug_points_array,
        );

        acc_color *= res.0;

        match res.2 {
            RayReturnState::Killed => return Vec3::NAN,
            RayReturnState::Stop => return acc_color,
            RayReturnState::Ray => {
                vec = res.1;
                xor_shift(&mut seed);
            },
        }

    }
}
