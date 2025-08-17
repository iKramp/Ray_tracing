//use super::material::*;
use super::trace::*;
use shared::{glam::Vec3, Bvh, Vertex};
//use crate::Resources;
#[allow(unused_imports)] //actually used for .sqrt because we don't allow std
use spirv_std::num_traits::Float;

pub struct HitRecord {
    pub triangle_id: u32,
    pub t: f32,
    pub instance_id: u32,
    #[cfg(feature = "debug")]
    pub triangle_tests: u32,
    #[cfg(feature = "debug")]
    pub box_tests: u32,
}

#[allow(clippy::new_without_default)]
impl HitRecord {
    pub fn new(/*resources: Rc<Resources>*/) -> Self {
        HitRecord {
            t: f32::INFINITY,
            triangle_id: u32::MAX,
            instance_id: 0,
            #[cfg(feature = "debug")]
            triangle_tests: 0,
            #[cfg(feature = "debug")]
            box_tests: 0,
        }
    }

    pub fn try_add(
        &mut self,
        t: f32,
        triangle_id: u32,
        instance_id: u32,
    ) {
        if t < self.t {
            self.t = t;
            self.triangle_id = triangle_id;
            self.instance_id = instance_id;
        }
    }
}

pub struct Mesh<'a> {
    pub verts: &'a [Vertex],
    pub tris: &'a [(u32, u32, u32)],
    pub material_id: u32,
    pub bvh_buffer: &'a [Bvh],
    pub bvh_root: u32,
}

impl Mesh<'_> {
    fn hit_triangle(
        &self,
        i: u32,
        ray: &Ray,
        t_clamp: (f32, f32),
        record: &mut HitRecord,
        triangle_id: u32,
        instance_id: u32,
        backface_cull: bool,
    ) {
        let triangle = self.tris[i as usize];
        let p0 = &self.verts[triangle.0 as usize];
        let p1 = &self.verts[triangle.1 as usize];
        let p2 = &self.verts[triangle.2 as usize];

        if let Some(t) = triangle_ray_intersect(p0.pos, p1.pos, p2.pos, ray, t_clamp, backface_cull) {

            record.try_add(
                t,
                triangle_id,
                instance_id,
            );
        }
    }

    fn hit_bvh(&self, ray: &Ray, t_clamp: (f32, f32), record: &mut HitRecord, instance_id: u32, backface_cull: bool) {
        let mut stack = [0_u32; 32];
        let mut stack_size = 1;
        stack[0] = self.bvh_root;
        while stack_size > 0 {
            let node = &self.bvh_buffer[stack[stack_size - 1] as usize];

            #[cfg(feature = "debug")]
            record.box_tests += 1;

            if !ray.hits_bounding(&node.bounding_box) {
                stack_size -= 1;
                continue;
            }

            if matches!(node.mode, shared::ChildTriangleMode::Children) {
                stack[stack_size - 1] = node.child_1_or_first_tri;
                stack[stack_size] = node.child_2_or_last_tri;
                stack_size += 1;
                continue;
            }

            stack_size -= 1;

            let first_triangle = node.child_1_or_first_tri;
            let last_triangle = node.child_2_or_last_tri;
            for i in first_triangle..=last_triangle {
                self.hit_triangle(i, ray, t_clamp, record, i, instance_id, backface_cull);
            }
            #[cfg(feature = "debug")]
            record.triangle_tests += last_triangle - first_triangle + 1;
        }
    }

    pub fn hit(&self, ray: &Ray, t_clamp: (f32, f32), record: &mut HitRecord, instance_id: u32, backface_cull: bool) {
        self.hit_bvh(ray, t_clamp, record, instance_id, backface_cull)
    }
}

fn triangle_ray_intersect(
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    ray: &Ray,
    t_clamp: (f32, f32),
    backface_cull: bool,
) -> Option<f32> {
    let a = p1 - p0;
    let b = p2 - p0;
    let normal = &mut a.cross(b).normalize();
    let d = -(normal.dot(p0));
    let dot_prod = normal.dot(ray.orientation);

    if dot_prod.abs() < f32::EPSILON {
        return None;
    }
    if backface_cull && dot_prod > 0.0 {
        return None;
    }
    
    let t = -(normal.dot(ray.pos) + d) / normal.dot(ray.orientation);
    if t < t_clamp.0 || t > t_clamp.1 {
        return None;
    }
    let hit = ray.pos + ray.orientation * t;
    let mut c;

    let edge0 = p1 - p0;
    let vp0 = hit - p0;
    c = edge0.cross(vp0);
    if normal.dot(c) < 0.0 {
        return None;
    }

    let edge1 = p2 - p1;
    let vp1 = hit - p1;
    c = edge1.cross(vp1);
    if normal.dot(c) < 0.0 {
        return None;
    }

    let edge2 = p0 - p2;
    let vp2 = hit - p2;
    c = edge2.cross(vp2);
    if normal.dot(c) < 0.0 {
        return None;
    }
    Some(t)
}
