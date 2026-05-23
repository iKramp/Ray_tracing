#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::type_complexity)]
#![allow(unexpected_cfgs)]
#![allow(clippy::too_many_arguments)]
#![feature(stmt_expr_attributes)]

use shared::{
    glam::Vec3, Bvh, CamData, Face, ImageInfo, Instance, Object, SceneInfo, Vec2Aligned,
    Vec3Aligned, Vertex,
};

use crate::modules::{
    get_seed,
    material::{GenericMaterial, RayReturnState},
    rand_float,
    trace::{claculate_vec_dir_from_cam, Ray},
    xor_shift, ObjectInfo,
};
pub mod debug_points;
pub mod modules;

//returns
//(color contribution, new ray, whether to stop or continue, direct color contribution for mis)
pub fn tracer_main(
    trace_ray: Ray,
    rand_seed: u32,
    color: Vec3,

    data: &CamData,
    scene_info: &SceneInfo,

    vertex_buffer: &[Vertex],
    normal_buffer: &[Vec3Aligned],
    uv_buffer: &[Vec2Aligned],
    triangle_buffer: &[Face],
    bvh_buffer: &[Bvh],
    object_buffer: &[Object],
    instance_buffer: &[Instance],
    image_info_buffer: &[ImageInfo],
    material_buffer: &[GenericMaterial],

    debug_points_array: &mut [Vec3Aligned],
) -> (Vec3, Ray, RayReturnState, Vec3) {
    let objects = ObjectInfo {
        vertex_buffer,
        normal_buffer,
        uv_buffer,
        triangle_buffer,
        object_buffer,
        instance_buffer,
        bvh_buffer,
        image_info_buffer,
        material_buffer,
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
    normal_buffer: &[Vec3Aligned],
    uv_buffer: &[Vec2Aligned],
    triangle_buffer: &[Face],
    bvh_buffer: &[Bvh],
    object_buffer: &[Object],
    instance_buffer: &[Instance],
    image_info_buffer: &[ImageInfo],
    material_buffer: &[GenericMaterial],

    debug_points_array: &mut [Vec3Aligned],
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
        debug_points_array[0] = Vec3Aligned::new(vec.pos);
        debug_points_array[1] = Vec3Aligned::new(vec.pos + vec.orientation * 100.0);
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

        acc_color *= res.0;

        match res.2 {
            RayReturnState::Killed => return Vec3::NAN,
            RayReturnState::Stop => return acc_color,
            RayReturnState::Ray => {
                vec = res.1;
                xor_shift(&mut seed);
            }
        }
    }
}
