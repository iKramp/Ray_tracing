extern crate vector3d;
use super::trace::*;
use vector3d::Vector3d;

pub struct HitRecord {
    pub pos: Vector3d<f64>,
    pub normal: Vector3d<f64>,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new() -> Self {
        HitRecord {
            pos: Vector3d::new(0.0, 0.0, 0.0),
            normal: Vector3d::new(0.0, 0.0, 0.0),
            t: f64::INFINITY,
            front_face: true,
        }
    }

    pub fn try_add(&mut self, pos: Vector3d<f64>, normal: Vector3d<f64>, t: f64, ray: &Ray) {
        if t < self.t {
            self.t = t;
            self.pos = pos;
            self.front_face = ray.orientation.dot(normal) < 0.0;
            self.normal = if self.front_face { normal } else {-normal};
        }
    }
}

pub trait HitObject {
    fn hit(&self, ray: &Ray, t_clamp: (f64, f64), record: &mut HitRecord) -> bool;
    fn calculate_normal(&self, hit: Vector3d<f64>) -> Vector3d<f64>;
}

pub struct Sphere {
    pub pos: Vector3d<f64>,
    pub radius: f64,
}

impl Sphere {
    pub fn new(pos: Vector3d<f64>, radius: f64) -> Self {
        Sphere { pos, radius }
    }
}

impl HitObject for Sphere {
    fn hit(&self, ray: &Ray, t_clamp: (f64, f64), record: &mut HitRecord) -> bool {
        //some black magic math idk
        let oc = ray.pos - self.pos;
        let a = ray.orientation.norm2();
        let half_b = oc.dot(ray.orientation);
        let c = oc.norm2() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant > 0.0 {
            let root = (-half_b - discriminant.sqrt()) / a;
            if root > t_clamp.0 && root < t_clamp.1 {
                let hit = ray.pos + ray.orientation * root;
                let normal = self.calculate_normal(hit);
                record.try_add(hit, normal, root, ray);
                return true;
            }
            let root = (-half_b + discriminant.sqrt()) / a; //uncomment if you want to show sphere backfaces
            if root > t_clamp.0 && root < t_clamp.1 {
                let hit = ray.pos + ray.orientation * root;
                let normal = self.calculate_normal(hit);
                record.try_add(hit, normal, root, ray);
                return true;
            }
        }
        false
    }

    fn calculate_normal(&self, hit: Vector3d<f64>) -> Vector3d<f64> {
        (hit - self.pos) / self.radius
    }
}
