//! Ported to Rust from <https://github.com/Tw1ddle/Sky-Shader/blob/master/src/shaders/glsl/sky.fragment>

#![cfg_attr(target_arch = "spirv", no_std, feature(lang_items))]

use core::f32::consts::PI;
use glam::{vec3, Vec3};

pub use spirv_std::glam;

// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub canvas_height: usize,
    pub canvas_width: usize,
    pub fov: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub orientation_x: f32,
    pub orientation_y: f32,
    pub orientation_z: f32,
    pub samples: u32,
}

pub fn saturate(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

pub fn pow(v: Vec3, power: f32) -> Vec3 {
    vec3(v.x.powf(power), v.y.powf(power), v.z.powf(power))
}

pub fn exp(v: Vec3) -> Vec3 {
    vec3(v.x.exp(), v.y.exp(), v.z.exp())
}

/// Based on: <https://seblagarde.wordpress.com/2014/12/01/inverse-trigonometric-functions-gpu-optimization-for-amd-gcn-architecture/>
pub fn acos_approx(v: f32) -> f32 {
    let x = v.abs();
    let mut res = -0.155972 * x + 1.56467; // p(x)
    res *= (1.0f32 - x).sqrt();

    if v >= 0.0 {
        res
    } else {
        PI - res
    }
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    // Scale, bias and saturate x to 0..1 range
    let x = saturate((x - edge0) / (edge1 - edge0));
    // Evaluate polynomial
    x * x * (3.0 - 2.0 * x)
}

//own code
pub use vector3d::Vector3d;

#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub struct PositionedVector3d {
    pub pos: glam::Vec4,
    pub orientation: glam::Vec4,
}


#[derive(Copy, Clone, PartialEq)]
pub struct CamData {
    pub pos: glam::Vec4,
    pub orientation: glam::Vec4,
    //pub transform: PositionedVector3d,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub fov: f32,
    pub samples: u32,
}


pub struct SceneInfo {
    pub sun_orientation: Vector3d,
    //pub hittable_objects: Vec<Box<dyn HitObject>>,
    pub hittable_objects: [Sphere; 1],
}

impl Default for SceneInfo {
    fn default() -> Self {
        SceneInfo {
            sun_orientation: Vector3d::new(1.0, -1.0, 1.0),
            /*hittable_objects: Vec::new()*//*vec![parse_obj_file(
                "program/src/resources/teapot.obj",
                0.25,
                vector3d::new(-0.05, 0.25, 0.0),
            )]*/
            hittable_objects: [Sphere::new(Vector3d::new(0.0, 0.0, 1.0), 0.5)],
            //sphere: Sphere::new(Vector3d::new(0.0, 0.0, 0.0), 0.5),
        }
    }
}

pub struct Sphere {
    pub pos: Vector3d,
    pub radius: f64,
    pub padding: f64,
    //pub material: Box<Rc<dyn Material>>,
}

impl Sphere {
    pub fn new(pos: Vector3d, radius: f64 /*, material: Box<Rc<dyn Material>>*/) -> Self {
        Self {
            radius,
            pos,
            padding: 0.0,
            //material,
        }
    }
}