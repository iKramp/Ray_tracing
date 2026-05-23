//! Ported to Rust from <https://github.com/Tw1ddle/Sky-Shader/blob/master/src/shaders/glsl/sky.fragment>
#![allow(unexpected_cfgs)]
#![cfg_attr(target_arch = "spirv", no_std, feature(lang_items))]

use core::f32::consts::PI;
use glam::{vec3, Vec3};

pub use spirv_std::glam;

use spirv_std::glam::{UVec2, UVec3, Vec2};
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
    pub depth: u32,
    pub transform: glam::Affine3A,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub fov: f32,
    pub frame: u32,
    pub debug_number: u32,
    pub debug_information: DebugInformation,
    pub debug_point_color: Vec3Aligned,
    pub reset: u32,
    pub random_seed: u32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DebugInformation {
    None,
    TriangleIntersection,
    BvhIntersection,
    RecordPoints,
}

#[repr(C)]
pub struct SceneInfo {
    pub num_instances: u32,
    pub num_bvh_nodes: u32,
    pub num_triangles: u32,
    pub num_materials: u32,
    pub num_emissive_instances: u32,
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

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Face {
    pub vert: UVec3,
    #[cfg(not(target_arch = "spirv"))]
    pub _padding_1: [u8; 4],
    pub normal: UVec3,
    #[cfg(not(target_arch = "spirv"))]
    pub _padding_2: [u8; 4],
    pub uv: UVec3,
    #[cfg(not(target_arch = "spirv"))]
    pub _padding_3: [u8; 4],
}

#[derive(Debug, Default, Copy)]
#[repr(C, align(16))]
pub struct Vertex {
    pub pos: Vec3,
    #[cfg(not(target_arch = "spirv"))]
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

impl Clone for Vertex {
    #[allow(clippy::non_canonical_clone_impl)] //because spirv
    fn clone(&self) -> Self {
        Self::new(self.pos)
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(align(16))]
pub struct Vec3Aligned(pub Vec3);

impl Vec3Aligned {
    pub fn new(v: Vec3) -> Self {
        Self(v)
    }
}

impl core::ops::Deref for Vec3Aligned {
    type Target = Vec3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for Vec3Aligned {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(align(16))]
pub struct Vec2Aligned(pub Vec2);

impl Vec2Aligned {
    pub fn new(v: Vec2) -> Self {
        Self(v)
    }
}

impl core::ops::Deref for Vec2Aligned {
    type Target = Vec2;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for Vec2Aligned {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct BoundingBox {
    min: Vec3,
    #[cfg(not(target_arch = "spirv"))]
    padding_1: [u8; 4],
    max: Vec3,
    #[cfg(not(target_arch = "spirv"))]
    invalid: u32,
}

impl BoundingBox {
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn union(&mut self, other: &BoundingBox) {
        #[cfg(not(target_arch = "spirv"))]
        if self.invalid != 0 {
            *self = *other;
            return;
        }
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    pub fn default_invalid() -> Self {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
            #[cfg(not(target_arch = "spirv"))]
            padding_1: [0; 4],
            #[cfg(not(target_arch = "spirv"))]
            invalid: 1,
        }
    }

    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min,
            max,
            #[cfg(not(target_arch = "spirv"))]
            padding_1: [0; 4],
            #[cfg(not(target_arch = "spirv"))]
            invalid: 0,
        }
    }

    pub fn surface_area(&self) -> f32 {
        let size_x = (self.max.x - self.min.x).abs();
        let size_y = (self.max.y - self.min.y).abs();
        let size_z = (self.max.z - self.min.z).abs();
        2.0 * (size_x * size_y + size_x * size_z + size_y * size_z)
    }

    pub fn min(&self) -> Vec3 {
        self.min
    }

    pub fn max(&self) -> Vec3 {
        self.max
    }

    #[cfg(not(target_arch = "spirv"))]
    pub fn is_valid(&self) -> bool {
        self.invalid == 0
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    pub bvh_root: u32,
    pub num_triangles: u32,
    pub triangle_start: u32,
    pub material_id: u32,
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub transform: glam::Affine3A,
    pub object_id: u32,
}

#[derive(Debug, Clone)]
#[repr(C, align(16))]
pub struct Bvh {
    pub bounding_box: BoundingBox,
    pub child_1_or_first_tri: u32,
    pub child_2_or_last_tri: u32,
    pub mode: ChildTriangleMode,
}

#[derive(Debug, Clone)]
#[repr(u32)]
pub enum ChildTriangleMode {
    Children = 0,
    Triangles = 1,
}

#[derive(Debug, Clone)]
#[repr(C, align(16))]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub data_index: u32,
}
