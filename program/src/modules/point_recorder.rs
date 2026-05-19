use shared::{glam::Vec3, Face, SceneInfo, Vec2Aligned, Vec3Aligned, Vertex};
use tracer::modules::is_vec_3_nan;

use crate::modules::buffers::{
    BufferHolder, BVH_BUFFER, DEBUG_POINTS_BUFFER, INSTANCE_BUFFER, NORMAL_BUFFER, OBJ_BUFFER,
    TRI_BUFFER, UV_BUFFER, VERT_BUFFER,
};

pub fn record_points(
    buffers: &mut BufferHolder,
    coords: (u32, u32),
    cam_data: &mut shared::CamData,
    scene_info: &SceneInfo,
) {
    let vertex_buffer = buffers.get(VERT_BUFFER).unwrap();
    let normal_buffer = buffers.get(NORMAL_BUFFER).unwrap();
    let uv_buffer = buffers.get(UV_BUFFER).unwrap();
    let triangle_buffer = buffers.get(TRI_BUFFER).unwrap();
    let bvh_buffer = buffers.get(BVH_BUFFER).unwrap();
    let object_buffer = buffers.get(OBJ_BUFFER).unwrap();
    let instance_buffer = buffers.get(INSTANCE_BUFFER).unwrap();

    let vertex_buffer = unsafe {
        std::slice::from_raw_parts(
            vertex_buffer.data.as_ptr() as *const Vertex,
            vertex_buffer.num_elements,
        )
    };
    let normal_buffer = unsafe {
        std::slice::from_raw_parts(
            normal_buffer.data.as_ptr() as *const Vec3Aligned,
            normal_buffer.num_elements,
        )
    };
    let uv_buffer = unsafe {
        std::slice::from_raw_parts(
            uv_buffer.data.as_ptr() as *const Vec2Aligned,
            uv_buffer.num_elements,
        )
    };
    let triangle_buffer = unsafe {
        std::slice::from_raw_parts(
            triangle_buffer.data.as_ptr() as *const Face,
            triangle_buffer.num_elements,
        )
    };
    let bvh_buffer = unsafe {
        std::slice::from_raw_parts(
            bvh_buffer.data.as_ptr() as *const shared::Bvh,
            bvh_buffer.num_elements,
        )
    };
    let object_buffer = unsafe {
        std::slice::from_raw_parts(
            object_buffer.data.as_ptr() as *const shared::Object,
            object_buffer.num_elements,
        )
    };
    let instance_buffer = unsafe {
        std::slice::from_raw_parts(
            instance_buffer.data.as_ptr() as *const shared::Instance,
            instance_buffer.num_elements,
        )
    };

    let prev_mode = cam_data.debug_information;
    cam_data.debug_information = shared::DebugInformation::RecordPoints;

    let debug_buffer = loop {
        let mut debug_buffer = Vec::with_capacity(cam_data.depth as usize + 5);
        debug_buffer.resize(cam_data.depth as usize + 5, Vec3Aligned::new(Vec3::NAN));
        let color = tracer::trace_single_ray(
            coords,
            cam_data,
            scene_info,
            vertex_buffer,
            normal_buffer,
            uv_buffer,
            triangle_buffer,
            bvh_buffer,
            object_buffer,
            instance_buffer,
            &[],
            &mut debug_buffer,
        );
        cam_data.debug_point_color = Vec3Aligned::new(color);
        //count elements until first NAN
        let mut valid_points = 0;
        for v in &debug_buffer {
            if v.x.is_nan() || v.y.is_nan() || v.z.is_nan() {
                break;
            }
            valid_points += 1;
        }
        println!("Valid points recorded: {valid_points}");
        if !is_vec_3_nan(&color) {
            break debug_buffer;
        }
        cam_data.random_seed = cam_data.random_seed.wrapping_add(1);
    };

    cam_data.debug_information = prev_mode;

    //print points
    println!("Recorded points for pixel ({}, {}):", coords.0, coords.1);
    println!("Index\tPosition");
    for (i, v) in debug_buffer.iter().enumerate() {
        println!(
            "{}\t({:08X}, {:08X}, {:08X}), ({:.02}, {:.02}, {:.02})",
            i,
            v.x.to_bits(),
            v.y.to_bits(),
            v.z.to_bits(),
            v.x,
            v.y,
            v.z
        );
    }

    buffers.insert(DEBUG_POINTS_BUFFER, debug_buffer);
}

pub fn fake_points(buffers: &mut BufferHolder, cam_data: &shared::CamData) {
    let debug_buffer = vec![Vec3Aligned::new(Vec3::NAN); cam_data.depth as usize + 5];
    buffers.insert(DEBUG_POINTS_BUFFER, debug_buffer);
}
