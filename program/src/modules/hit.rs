extern crate vector3d;
use super::material::*;
use super::trace::*;
use crate::Resources;
use std::rc::Rc;
use vector3d::Vector3d;

pub struct HitRecord {
    pub pos: Vector3d<f64>,
    pub ray: Ray,
    pub normal: Vector3d<f64>, //points toward the modules
    pub t: f64,
    pub front_face: bool, //is the modules and normal on the front of the face?
    pub material: Box<Rc<dyn Material>>,
    pub uv: (f64, f64),
    pub resources: Rc<Resources>,
}

#[allow(clippy::new_without_default)]
impl HitRecord {
    pub fn new(resources: Rc<Resources>) -> Self {
        HitRecord {
            pos: Vector3d::default(),
            ray: Ray::new(Vector3d::default(), Vector3d::default()),
            normal: Vector3d::default(),
            t: f64::INFINITY,
            front_face: true,
            material: Box::new(Rc::new(BackgroundMaterial {})),
            uv: (0.0, 0.0),
            resources,
        }
    }

    pub fn try_add(
        &mut self,
        pos: Vector3d<f64>,
        normal: Vector3d<f64>,
        t: f64,
        ray: &Ray,
        material: Box<Rc<dyn Material>>,
        uv: (f64, f64),
    ) {
        if t < self.t {
            self.t = t;
            self.ray = *ray;
            self.pos = pos;
            self.front_face = ray.orientation.dot(normal) < 0.0;
            self.normal = if self.front_face { normal } else { -normal };
            self.material = material;
            self.uv = uv;
        }
    }
}

pub trait HitObject {
    fn hit(&self, ray: &Ray, t_clamp: (f64, f64), record: &mut HitRecord);
    fn calculate_normal(&self, hit: Vector3d<f64>) -> Vector3d<f64>;
}

pub struct Sphere {
    pub pos: Vector3d<f64>,
    pub radius: f64,
    pub material: Box<Rc<dyn Material>>,
}

impl Sphere {
    pub fn new(pos: Vector3d<f64>, radius: f64, material: Box<Rc<dyn Material>>) -> Self {
        Sphere {
            pos,
            radius,
            material,
        }
    }

    fn get_uv(&self, normal: Vector3d<f64>) -> (f64, f64) {
        let angle_y = (-normal).y.asin() / core::f64::consts::PI + 0.5;
        let mut angle_xz = ((normal).x.atan2((-normal).z) / core::f64::consts::PI + 1.0) / 2.0;
        angle_xz += 1.0;
        angle_xz %= 1.0;
        (angle_xz.clamp(0.0, 1.0), angle_y.clamp(0.0, 1.0))
    }

    fn try_add_to_record(
        &self,
        ray: &Ray,
        t: f64,
        record: &mut HitRecord,
        t_clamp: (f64, f64),
    ) -> bool {
        if t < t_clamp.1 && t > t_clamp.0 {
            let hit = ray.pos + ray.orientation * t;
            let normal = self.calculate_normal(hit);
            record.try_add(
                hit,
                normal,
                t,
                ray,
                self.material.clone(),
                self.get_uv(normal),
            );
            return true;
        }
        false
    }
}

impl HitObject for Sphere {
    fn hit(&self, ray: &Ray, t_clamp: (f64, f64), record: &mut HitRecord) {
        //some black magic math idk
        let oc = ray.pos - self.pos;
        let a = ray.orientation.norm2();
        let half_b = oc.dot(ray.orientation);
        let c = oc.norm2() - self.radius * self.radius;
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

    fn calculate_normal(&self, hit: Vector3d<f64>) -> Vector3d<f64> {
        (hit - self.pos) / self.radius
    }
}

pub struct Vertex {
    pub(crate) pos: Vector3d<f64>,
    uv: (f64, f64),
}

impl Vertex {
    pub fn new(pos: Vector3d<f64>, uv: (f64, f64)) -> Self {
        Vertex { pos, uv }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            pos: Vector3d::default(),
            uv: (0.0, 0.0),
        }
    }
}

pub struct Mesh {
    verts: Vec<Vertex>,
    tris: Vec<(usize, usize, usize)>,
    material: Box<Rc<dyn Material>>,
}

impl Mesh {
    pub fn new(
        verts: Vec<Vertex>,
        tris: Vec<(usize, usize, usize)>,
        material: Box<Rc<dyn Material>>,
    ) -> Self {
        Mesh {
            verts,
            tris,
            material,
        }
    }
}

fn triangle_ray_intersect(
    p0: Vector3d<f64>,
    p1: Vector3d<f64>,
    p2: Vector3d<f64>,
    ray: &Ray,
    t_clamp: (f64, f64),
) -> Option<f64> {
    let a = p1 - p0;
    let b = p2 - p0;
    let normal = normalize_vec(&mut a.cross(b));
    let d = -(normal.dot(p0));
    if normal.dot(ray.orientation).abs() < f64::EPSILON {
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

impl HitObject for Mesh {
    fn hit(&self, ray: &Ray, t_clamp: (f64, f64), record: &mut HitRecord) {
        for triangle in &self.tris {
            let vert = Vertex::default();
            let p0 = self.verts.get(triangle.0).unwrap_or(&vert);
            let p1 = self.verts.get(triangle.1).unwrap_or(&vert);
            let p2 = self.verts.get(triangle.2).unwrap_or(&vert);
            if let Some(t) = triangle_ray_intersect(p0.pos, p1.pos, p2.pos, ray, t_clamp) {
                let a = p1.pos - p0.pos;
                let b = p2.pos - p0.pos;
                let normal = normalize_vec(&mut a.cross(b));
                record.try_add(
                    ray.pos + ray.orientation * t,
                    normal,
                    t,
                    ray,
                    self.material.clone(),
                    (0.0, 0.0),
                );
            }
        }
    }

    fn calculate_normal(&self, _hit: Vector3d<f64>) -> Vector3d<f64> {
        Vector3d::default() //unused
    }
}
