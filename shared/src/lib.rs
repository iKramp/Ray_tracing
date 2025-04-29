//! Ported to Rust from <https://github.com/Tw1ddle/Sky-Shader/blob/master/src/shaders/glsl/sky.fragment>

#![cfg_attr(target_arch = "spirv", no_std, feature(lang_items))]

pub mod materials;

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

#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct CamData {
    pub samples: u32,
    pub depth: u32,
    pub transform: glam::Affine3A,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub fov: f32,
}

#[repr(C)]
pub struct SceneInfo {
    pub num_objects: u32,
    pub sun_orientation: Vec3,
}

pub struct Sphere {
    pub pos: Vec3,
    pub radius: f32,
    pub padding: f32,
}

impl Sphere {
    pub fn new(pos: Vec3, radius: f32 /*, material: Box<Rc<dyn Material>>*/) -> Self {
        Self {
            radius,
            pos,
            padding: 0.0,
            //material,
        }
    }
}

#[cfg(target_arch = "spirv")]
#[derive(Debug, Default)]
pub struct Vertex {
    pub pos: Vec3,
}

#[cfg(not(target_arch = "spirv"))]
#[derive(Debug, Default)]
pub struct Vertex {
    pub pos: Vec3,
    _padding: [u8; 4],
}

impl Vertex {
    pub fn new(pos: Vec3) -> Self {
        #[cfg(target_arch = "spirv")]
        {
            Vertex { pos }
        }

        #[cfg(not(target_arch = "spirv"))]
        {
            Vertex {
                pos,
                _padding: [0; 4],
            }
        }
    }
}

pub struct BoundingBox {
    pub min: Vec3,
    #[cfg(not(target_arch = "spirv"))]
    pub padding_1: [u8; 4],
    pub max: Vec3,
    #[cfg(not(target_arch = "spirv"))]
    pub padding_2: [u8; 4],
}

pub struct Object {
    pub first_triangle: u32,
    pub last_triangle: u32,
    pub bounding_box: BoundingBox,
}

pub struct Instance {
    pub transform: glam::Affine3A,
    pub object_id: u32,
}
