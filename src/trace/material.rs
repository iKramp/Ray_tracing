extern crate vector3d;
extern crate rand;

use vector3d::Vector3d;
use super::hit::*;
use super::trace::*;
use rand::prelude::*;


fn mult_colors(lhs: Vector3d<f64>, rhs: Vector3d<f64>) -> Vector3d<f64>{
    Vector3d::new(
        lhs.x * rhs.x / 255.0,
        lhs.y * rhs.y / 255.0,
        lhs.z * rhs.z / 255.0
    )
}

fn normalize_vec(vec: &mut Vector3d<f64>) {
    let len = vec.norm2().sqrt();
    vec.x /= len;
    vec.y /= len;
    vec.z /= len;
}

fn rand_vec_in_unit_sphere(rng: &mut ThreadRng) -> Vector3d<f64> {
    Vector3d::new(
        rng.gen_range(-1.0..1.0),
        rng.gen_range(-1.0..1.0),
        rng.gen_range(-1.0..1.0),
    )
}


pub trait Material {
    fn get_color(&self, record: &HitRecord, next_ray_color: Vector3d<f64>) -> Vector3d<f64> {Vector3d::new(0.0, 0.0, 0.0)}
    ///gets next ray direction (not absolute position in world) and returns it
    fn get_next_ray_dir(&self, record: &HitRecord, old_ray: &Ray, rng: &mut ThreadRng) -> Vector3d<f64> {Vector3d::new(0.0, 0.0, 0.0)}
}

pub struct EmptyMaterial {}

impl Material for EmptyMaterial {}

pub struct DiffuseMaterial {
    pub color: Vector3d<f64>,
    rng: ThreadRng
}

impl DiffuseMaterial {
    pub fn new(color: Vector3d<f64>) -> Self {
        DiffuseMaterial { color, rng: rand::thread_rng() }
    }
}

impl Material for DiffuseMaterial {
    fn get_color(&self, record: &HitRecord, next_ray_color: Vector3d<f64>) -> Vector3d<f64> {
        mult_colors(next_ray_color, self.color)
    }
    
    fn get_next_ray_dir(&self, record: &HitRecord, old_ray: &Ray, rng: &mut ThreadRng) -> Vector3d<f64> {
        let mut rand_vec = Vector3d::new(2.0, 0.0, 0.0);
        while rand_vec.norm2() > 1.0 {
            rand_vec = rand_vec_in_unit_sphere(rng);
        }
        normalize_vec(&mut rand_vec);
        
        Vector3d::new(record.normal.x + rand_vec.x, record.normal.y + rand_vec.y, record.normal.z + rand_vec.z)
    }
}

pub struct MetalMaterial {
    pub color: Vector3d<f64>,
    rng: ThreadRng,
    fuzz: f64,
}

impl MetalMaterial {
    pub fn new(color: Vector3d<f64>, fuzz: f64) -> Self {
        MetalMaterial { color, rng: rand::thread_rng(), fuzz }
    }
}

impl Material for MetalMaterial {
    fn get_color(&self, record: &HitRecord, next_ray_color: Vector3d<f64>) -> Vector3d<f64> {
        mult_colors(next_ray_color, self.color)
    }

    fn get_next_ray_dir(&self, record: &HitRecord, old_ray: &Ray, rng: &mut ThreadRng) -> Vector3d<f64> {
        let old_ray = old_ray.orientation;
        let mut new_ray = old_ray - record.normal * old_ray.dot(record.normal) * 2.0;
        normalize_vec(&mut new_ray);
        new_ray = new_ray + rand_vec_in_unit_sphere(rng) * self.fuzz;
        return if new_ray.dot(record.normal) > 0.0 {
            new_ray
        } else {
            Vector3d::new(0.0, 0.0, 0.0)
        }
    }
}

pub struct NormalMaterial{}

impl Material for NormalMaterial {
    fn get_color(&self, record: &HitRecord, next_ray_color: Vector3d<f64>) -> Vector3d<f64> {
        return Vector3d::new(
            (record.normal.x + 1.0) * 255.0 / 2.0,
            (record.normal.y + 1.0) * 255.0 / 2.0,
            (record.normal.z + 1.0) * 255.0 / 2.0,
        );
    }
}