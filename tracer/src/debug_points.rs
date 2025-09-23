use shared::{
    glam::{UVec2, Vec2, Vec3},
    CamData, Vertex,
};
#[allow(unused_imports)] //used for gpu
use spirv_std::num_traits::Float;

use crate::modules::{is_nan, is_vec_3_nan};

pub fn check_points_proximity(
    data: &CamData,
    points: &[Vertex],
    pixel_coord: UVec2,
) -> Vec3 {
    let mut counter = 0;
    loop {
        let point1 = points[counter].clone();
        let point2 = points[counter + 1].clone();
        if is_vec_3_nan(&point1.pos) || is_vec_3_nan(&point2.pos) {
            return Vec3::NAN;
        }
        counter += 1;

        let a = project_world_point(data, point1.pos);
        let b = project_world_point(data, point2.pos);

        if is_nan(a.0) || is_nan(a.1) || is_nan(b.0) || is_nan(b.1) {
            continue;
        }

        //check if pixel coord is less than a pixel away from the line segment ab
        let ab = Vec2::new(b.0 - a.0, b.1 - a.1);
        let ap = Vec2::new(pixel_coord.x as f32 - a.0, pixel_coord.y as f32 - a.1);
        let ab_len_sq = ab.length_squared();

        if ab_len_sq < f32::EPSILON {
            continue; // a and b are the same point
        }

        let t = ap.dot(ab) / ab_len_sq;

        if t > 0.0 && t <= 1.0 {
            let projection = Vec2::new(a.0, a.1) + ab * t;
            let dist_sq = (Vec2::new(pixel_coord.x as f32, pixel_coord.y as f32) - projection)
                .length_squared();
            if dist_sq < 1.0 {
                return data.debug_point_color.pos;
            } else if dist_sq < 4.0 {
                return Vec3::ZERO;
            }
        }
    }
}

pub fn project_world_point(data: &CamData, point: Vec3) -> (f32, f32) {
    let inv = data.transform.inverse();
    let cam_space = inv.transform_point3(point);

    if cam_space.z <= 0.0 {
        // Behind the camera → no valid projection
        return (f32::NAN, f32::NAN);
    }

    let fov = (data.fov.to_radians()) / 2.0;
    let edge_dist = fov.tan();

    // perspective projection into normalized device coordinates
    let x_ndc = (cam_space.x / cam_space.z) / edge_dist;
    let y_ndc = (cam_space.y / cam_space.z)
        / (edge_dist * data.canvas_height as f32 / data.canvas_width as f32);

    // convert NDC [-1,1] → pixel coords [0,width]/[0,height]
    let pix_x = (x_ndc + 1.0) * 0.5 * data.canvas_width as f32;
    let pix_y = (y_ndc + 1.0) * 0.5 * data.canvas_height as f32;

    (pix_x, pix_y)
}
