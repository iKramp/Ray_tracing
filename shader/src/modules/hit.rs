//use super::material::*;
use super::trace::*;
use shared::{glam::Vec3, Bvh, Vertex};
use spirv_std::num_traits::{float::FloatCore, Zero};
//use crate::Resources;
#[allow(unused_imports)] //actually used for .sqrt because we don't allow std
use spirv_std::num_traits::Float;

pub struct HitRecord {
    pub pos: Vec3,
    pub ray: Ray,
    pub normal_fixed: bool,
    pub normal: Vec3, //points toward the modules
    pub t: f32,
    pub front_face: bool, //is the modules and normal on the front of the face?
    pub material_id: u32,
    pub uv: (f32, f32),
    //pub resources: Rc<Resources>,
}

#[allow(clippy::new_without_default)]
impl HitRecord {
    pub fn new(/*resources: Rc<Resources>*/) -> Self {
        HitRecord {
            pos: Vec3::default(),
            ray: Ray::new(Vec3::default(), Vec3::default()),
            normal_fixed: true,
            normal: Vec3::default(),
            t: f32::INFINITY,
            front_face: true,
            material_id: 0,
            uv: (0.0, 0.0),
            //resources,
        }
    }

    pub fn try_add(
        &mut self,
        pos: Vec3,
        normal: Vec3,
        t: f32,
        ray: &Ray,
        material_id: u32,
        //material: Box<Rc<dyn Material>>,
        uv: (f32, f32),
    ) {
        if t < self.t {
            self.t = t;
            self.ray = *ray;
            self.pos = pos;
            self.front_face = ray.orientation.dot(normal) < 0.0;
            self.normal_fixed = false;
            self.normal = if self.front_face { normal } else { -normal };
            self.material_id = material_id;
            //self.material = material;
            self.uv = uv;
        }
    }
}

pub trait HitObject {
    fn hit(&self, ray: &Ray, t_clamp: (f32, f32), record: &mut HitRecord);
    fn calculate_normal(&self, hit: Vec3) -> Vec3;
}

pub trait SphereObject {
    //exists so we can define impl outside of shared
    fn get_uv(&self, normal: Vec3) -> (f32, f32);
    fn try_add_to_record(
        &self,
        ray: &Ray,
        t: f32,
        record: &mut HitRecord,
        t_clamp: (f32, f32),
    ) -> bool;
}

impl SphereObject for shared::Sphere {
    fn get_uv(&self, normal: Vec3) -> (f32, f32) {
        let angle_y = ((-normal).y).asin() / core::f32::consts::PI + 0.5;
        let mut angle_xz = ((normal).x.atan2(-normal.z) / core::f32::consts::PI + 1.0) / 2.0;
        angle_xz += 1.0;
        angle_xz %= 1.0;
        (angle_xz.clamp(0.0, 1.0), angle_y.clamp(0.0, 1.0))
    }

    fn try_add_to_record(
        &self,
        ray: &Ray,
        t: f32,
        record: &mut HitRecord,
        t_clamp: (f32, f32),
    ) -> bool {
        if t < t_clamp.1 && t > t_clamp.0 {
            let hit = ray.pos + ray.orientation * t;
            let normal = self.calculate_normal(hit);
            record.try_add(
                hit,
                normal,
                t,
                ray,
                0,
                //self.material.clone(),
                self.get_uv(normal),
            );
            return true;
        }
        false
    }
}

impl HitObject for shared::Sphere {
    fn hit(&self, ray: &Ray, t_clamp: (f32, f32), record: &mut HitRecord) {
        //some black magic math idk
        let oc = ray.pos - self.pos;
        let a = ray.orientation.length_squared();
        let half_b = oc.dot(ray.orientation);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant > 0.0 {
            let root = (-half_b - discriminant.sqrt()) / a;
            if self.try_add_to_record(ray, root, record, t_clamp) {
                return;
            }

            let root = (-half_b + discriminant.sqrt()) / a;
            self.try_add_to_record(ray, root, record, t_clamp);
        }
    }

    fn calculate_normal(&self, hit: Vec3) -> Vec3 {
        (hit - self.pos) / self.radius
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
    fn hit_triangle(&self, i: u32, ray: &Ray, t_clamp: (f32, f32), record: &mut HitRecord) {
        let triangle = self.tris[i as usize];
        let p0 = &self.verts[triangle.0 as usize];
        let p1 = &self.verts[triangle.1 as usize];
        let p2 = &self.verts[triangle.2 as usize];
        if let Some(t) = triangle_ray_intersect(p0.pos, p1.pos, p2.pos, ray, t_clamp) {
            let a = p1.pos - p0.pos;
            let b = p2.pos - p0.pos;
            let normal = b.cross(a).normalize();
            record.try_add(
                ray.pos + ray.orientation * t,
                normal,
                t,
                ray,
                self.material_id,
                (0.0, 0.0),
            );
        }
    }
    
    fn hit_bvh(&self, ray: &Ray, t_clamp: (f32, f32), record: &mut HitRecord) {
        let mut stack = [0_u32; 32];
        let mut stack_size = 1;
        stack[0] = self.bvh_root;
        while stack_size > 0 {
            let node = &self.bvh_buffer[stack[stack_size - 1] as usize];

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
                self.hit_triangle(i, ray, t_clamp, record);
            }
        }
    }
}

impl HitObject for Mesh<'_> {
    fn hit(&self, ray: &Ray, t_clamp: (f32, f32), record: &mut HitRecord) {
        self.hit_bvh(ray, t_clamp, record)
    }

    fn calculate_normal(&self, _hit: Vec3) -> Vec3 {
        Vec3::default() //unused
    }
}

fn triangle_ray_intersect(
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    ray: &Ray,
    t_clamp: (f32, f32),
) -> Option<f32> {
    let a = p1 - p0;
    let b = p2 - p0;
    let normal = &mut a.cross(b);
    let d = -(normal.dot(p0));
    if normal.dot(ray.orientation).abs() < f32::EPSILON {
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

