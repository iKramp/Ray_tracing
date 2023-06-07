extern crate rand;
extern crate vector3d;

use super::hit::*;
use rand::prelude::*;
use vector3d::Vector3d;

pub enum RayReturnState {
    Absorb, //return color 0, 0, 0
    Stop,   //don't trace forward, but still color the thing
    Ray(Vector3d<f64>),
}

fn mult_colors(lhs: Vector3d<f64>, rhs: Vector3d<f64>) -> Vector3d<f64> {
    Vector3d::new(
        lhs.x * rhs.x / 255.0,
        lhs.y * rhs.y / 255.0,
        lhs.z * rhs.z / 255.0,
    )
}

pub fn normalize_vec(vec: &mut Vector3d<f64>) -> Vector3d<f64> {
    let len = vec.norm2().sqrt();
    vec.x /= len;
    vec.y /= len;
    vec.z /= len;
    *vec
}

fn rand_vec_in_unit_sphere(rng: &mut ThreadRng) -> Vector3d<f64> {
    Vector3d::new(
        rng.gen_range(-1.0..1.0),
        rng.gen_range(-1.0..1.0),
        rng.gen_range(-1.0..1.0),
    )
}

pub trait Material {
    ///gets color based on its own properties and the incoming color
    fn get_color(&self, _record: &HitRecord, _next_ray_color: Vector3d<f64>) -> Vector3d<f64> {
        Vector3d::new(0.0, 0.0, 0.0)
    }
    ///gets next ray direction (not absolute position in world) and returns it
    fn get_next_ray_dir(&self, record: &HitRecord, rng: &mut ThreadRng) -> RayReturnState;
    ///gets color without the incoming color, as if the ray stopped there
    fn get_stop_color(&self, _record: &HitRecord) -> Vector3d<f64> {
        Vector3d::new(0.0, 0.0, 0.0)
    }
}

pub struct DiffuseMaterial {
    pub color: Vector3d<f64>,
}

impl DiffuseMaterial {
    pub fn new(color: Vector3d<f64>) -> Self {
        DiffuseMaterial { color }
    }
}

impl Material for DiffuseMaterial {
    fn get_color(&self, _record: &HitRecord, next_ray_color: Vector3d<f64>) -> Vector3d<f64> {
        mult_colors(next_ray_color, self.color)
    }

    fn get_next_ray_dir(&self, record: &HitRecord, rng: &mut ThreadRng) -> RayReturnState {
        let mut rand_vec = Vector3d::new(2.0, 0.0, 0.0);
        while rand_vec.norm2() > 1.0 {
            rand_vec = rand_vec_in_unit_sphere(rng);
        }
        normalize_vec(&mut rand_vec);

        RayReturnState::Ray(Vector3d::new(
            record.normal.x + rand_vec.x,
            record.normal.y + rand_vec.y,
            record.normal.z + rand_vec.z,
        ))
    }
}

pub struct MetalMaterial {
    pub color: Vector3d<f64>,
    roughness: f64,
}

impl MetalMaterial {
    pub fn new(color: Vector3d<f64>, roughness: f64) -> Self {
        MetalMaterial { color, roughness }
    }
}

impl Material for MetalMaterial {
    fn get_color(&self, _record: &HitRecord, next_ray_color: Vector3d<f64>) -> Vector3d<f64> {
        mult_colors(next_ray_color, self.color)
    }

    fn get_next_ray_dir(&self, record: &HitRecord, rng: &mut ThreadRng) -> RayReturnState {
        let old_ray = record.ray.orientation;
        let mut new_ray = old_ray - record.normal * old_ray.dot(record.normal) * 2.0;
        normalize_vec(&mut new_ray);
        new_ray = new_ray + rand_vec_in_unit_sphere(rng) * self.roughness;
        if new_ray.dot(record.normal) > 0.0 {
            RayReturnState::Ray(new_ray)
        } else {
            RayReturnState::Absorb
        }
    }
}

pub struct NormalMaterial {}

impl Material for NormalMaterial {
    fn get_stop_color(&self, record: &HitRecord) -> Vector3d<f64> {
        Vector3d::new(
            (record.normal.x + 1.0) * 255.0 / 2.0,
            (record.normal.y + 1.0) * 255.0 / 2.0,
            (record.normal.z + 1.0) * 255.0 / 2.0,
        )
    }

    fn get_next_ray_dir(&self, _record: &HitRecord, _rng: &mut ThreadRng) -> RayReturnState {
        RayReturnState::Stop
    }
}

pub struct BackgroundMaterial {}

impl Material for BackgroundMaterial {
    fn get_next_ray_dir(&self, _record: &HitRecord, _rng: &mut ThreadRng) -> RayReturnState {
        RayReturnState::Stop
    }

    fn get_stop_color(&self, record: &HitRecord) -> Vector3d<f64> {
        //return Vector3d::default();
        let mut temp = record.ray;
        temp.normalize();
        let factor = (temp.orientation.y + 0.5).clamp(0.0, 1.0);
        Vector3d::new(255.0, 255.0, 255.0) * (1.0 - factor)
            + Vector3d::new(0.5, 0.7, 1.0) * 255.0 * factor
    }
}

pub struct EmmissiveMaterial {
    pub light_color: Vector3d<f64>,
}

impl EmmissiveMaterial {
    pub fn new(light_color: Vector3d<f64>) -> Self {
        Self { light_color }
    }
}

impl Material for EmmissiveMaterial {
    fn get_next_ray_dir(&self, _record: &HitRecord, _rng: &mut ThreadRng) -> RayReturnState {
        RayReturnState::Stop
    }

    fn get_stop_color(&self, record: &HitRecord) -> Vector3d<f64> {
        let _ray_reversed = -record.ray.orientation;
        self.light_color /* * ray_reversed.dot(record.normal).sqrt()*/
    }
}

pub struct RefractiveMaterial {
    //TODO: ior not working properly
    color: Vector3d<f64>,
    ior: f64,
}

impl RefractiveMaterial {
    pub fn new(color: Vector3d<f64>, ior: f64) -> Self {
        Self { color, ior }
    }

    pub fn reflectance(&self, record: &HitRecord) -> f64 {
        //using schlick's approximation
        let mut r0 = (1.0 - self.ior) / (1.0 + self.ior);
        r0 *= r0;
        let cos_theta = record.normal.dot(-record.ray.orientation);
        r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
    }

    pub fn reflect(&self, record: &HitRecord) -> Vector3d<f64> {
        //println!("reflect");
        let old_ray = record.ray.orientation;

        old_ray - record.normal * old_ray.dot(record.normal) * 2.0
    }

    pub fn refract(&self, record: &HitRecord) -> Vector3d<f64> {
        //could do without refraction ratio but it's pointelss to recalculate
        let refraction_ratio = if record.front_face {
            1.0 / self.ior
        } else {
            self.ior
        };
        let cos_theta = record.normal.dot(-record.ray.orientation);
        let r1 = (record.ray.orientation + record.normal * cos_theta) * refraction_ratio;
        let r2 = -record.normal * (1.0 - r1.norm2()).sqrt();
        r1 + r2
    }
}

impl Material for RefractiveMaterial {
    fn get_next_ray_dir(&self, record: &HitRecord, rng: &mut ThreadRng) -> RayReturnState {
        let refraction_ratio = if record.front_face {
            1.0 / self.ior
        } else {
            self.ior
        };
        let cos_theta = record.normal.dot(-record.ray.orientation);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let reflectance = self.reflectance(record);
        if refraction_ratio * sin_theta > 1.0 || reflectance > rng.gen_range(0.0..1.0) {
            RayReturnState::Ray(self.reflect(record))
        } else {
            RayReturnState::Ray(self.refract(record))
        }
    }

    fn get_color(&self, _record: &HitRecord, next_ray_color: Vector3d<f64>) -> Vector3d<f64> {
        mult_colors(next_ray_color, self.color)
    }
}
