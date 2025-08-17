use core::f32;
use core::f32::consts::PI;

use crate::modules::trace::Ray;

//use image::GenericImageView;
use super::rand_float;
use shared::acos_approx;
use shared::glam::Vec3;
#[allow(unused_imports)] //actually used for .sqrt because we don't allow std
use spirv_std::num_traits::Float;

pub enum RayReturnState {
    Absorb, //return color 0, 0, 0
    Stop,   //don't trace forward, but still color the thing
    Ray,
}

pub struct RayReturn {
    pub state: RayReturnState,
    pub direction: Vec3,
}

fn rand_vec_in_unit_sphere(seed: &mut u32) -> Vec3 {
    let phi = rand_float(seed, (0.0, 2.0 * PI));
    let costheta = rand_float(seed, (-1.0, 1.0));

    let theta = acos_approx(costheta);
    let x = theta.sin() * phi.cos();
    let y = theta.sin() * phi.sin();
    let z = costheta;

    Vec3::new(x, y, z)
}

fn diffuse_ray_direction(seed: &mut u32, normal: Vec3, _in_ray: Vec3) -> Vec3 {
    let rand_vec = rand_vec_in_unit_sphere(seed);
    let res = rand_vec + normal;

    if res.length_squared() < f32::EPSILON {
        //if the random vector is too close to zero, just return the normal
        normal
    } else {
        res.normalize()
    }
}

pub trait Material {
    ///gets color based on its own properties and the incoming color
    fn get_color(&self, _next_ray_color: Vec3, _normal: Vec3, _uv: (f32, f32), _ray_dir: Vec3) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
    ///gets next modules direction (not absolute position in world) and returns it
    fn get_next_ray_dir(
        &self,
        seed: &mut u32,
        ray: Ray,
        normal: Vec3,
    ) -> RayReturn;
    ///gets color without the incoming color, as if the modules stopped there
    fn get_stop_color(&self, _normal: Vec3, _uv: (f32, f32), _ray_dir: Vec3) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
    fn backface_culling(&self) -> bool {
        true
    }
}

pub struct DiffuseMaterial {
    pub color: Vec3,
}

impl DiffuseMaterial {
    pub const fn new(color: Vec3) -> Self {
        DiffuseMaterial { color }
    }
}

impl Material for DiffuseMaterial {
    fn get_color(&self, next_ray_color: Vec3, _normal: Vec3, _uv: (f32, f32), _ray_dir: Vec3) -> Vec3 {
        next_ray_color * self.color
    }

    fn get_next_ray_dir(
        &self,
        seed: &mut u32,
        ray: Ray,
        normal: Vec3,
    ) -> RayReturn {
        let direction = diffuse_ray_direction(seed, normal, ray.orientation);

        RayReturn {
            state: RayReturnState::Ray,
            direction,
        }
    }
}

pub struct MetalMaterial {
    pub color: Vec3,
    roughness: f32,
}

impl MetalMaterial {
    pub const fn new(color: Vec3, roughness: f32) -> Self {
        MetalMaterial { color, roughness }
    }
}

impl Material for MetalMaterial {
    fn get_color(&self, next_ray_color: Vec3, _normal: Vec3, _uv: (f32, f32), _ray_dir: Vec3) -> Vec3 {
        next_ray_color * self.color
    }

    fn get_next_ray_dir(
        &self,
        seed: &mut u32,
        ray: Ray,
        normal: Vec3,
    ) -> RayReturn {
        let old_ray = ray.orientation;
        let mut new_ray = old_ray.reflect(normal).normalize();
        let rand_vec = diffuse_ray_direction(seed, normal, old_ray).normalize();
        new_ray = new_ray.lerp(rand_vec, self.roughness);

        RayReturn {
            state: RayReturnState::Ray,
            direction: new_ray,
        }
    }
}

pub struct NormalMaterial {}

impl NormalMaterial {
    fn get_color(&self, normal: Vec3) -> Vec3 {
        Vec3::new(
            (normal.x + 1.0) / 2.0,
            (normal.y + 1.0) / 2.0,
            (normal.z + 1.0) / 2.0,
        )
    }
}

impl Material for NormalMaterial {
    fn get_next_ray_dir(
        &self,
        seed: &mut u32,
        ray: Ray,
        normal: Vec3,
    ) -> RayReturn {
        RayReturn {
            state: RayReturnState::Ray,
            direction: diffuse_ray_direction(seed, normal, ray.orientation),
        }
    }

    fn get_color(&self, next_ray_color: Vec3, normal: Vec3, _uv: (f32, f32), _ray_dir: Vec3) -> Vec3 {
        next_ray_color * self.get_color(normal)
    }

}

pub struct BackgroundMaterial {}

impl Material for BackgroundMaterial {
    fn get_next_ray_dir(
        &self,
        _seed: &mut u32,
        _ray: Ray,
        _normal: Vec3,
    ) -> RayReturn {
        RayReturn {
            state: RayReturnState::Stop,
            direction: Vec3::default(),
        }
    }

    fn get_stop_color(&self, _normal: Vec3, _uv: (f32, f32), ray_dir: Vec3) -> Vec3 {
        let temp = ray_dir.normalize();

        let factor = (temp.y + 0.5).clamp(0.0, 1.0);
        Vec3::new(1.0, 1.0, 1.0) * (1.0 - factor) + Vec3::new(0.5, 0.7, 1.0) * factor
    }
}

pub struct EmmissiveMaterial {
    pub light_color: Vec3,
}

impl EmmissiveMaterial {
    pub const fn new(light_color: Vec3) -> Self {
        Self { light_color }
    }
}

impl Material for EmmissiveMaterial {
    fn get_next_ray_dir(
        &self,
        _seed: &mut u32,
        _ray: Ray,
        _normal: Vec3,
    ) -> RayReturn {
        RayReturn {
            state: RayReturnState::Stop,
            direction: Vec3::default(),
        }
    }

    fn get_stop_color(&self, normal: Vec3, _uv: (f32, f32), ray_dir: Vec3) -> Vec3 {
        let ray_reversed = -ray_dir.normalize();

        let dot_product = ray_reversed.dot(normal).abs(); //abs for weird geometries that have haps
                                                          //into backface triangles

        self.light_color * dot_product.sqrt()
    }
}

pub struct RefractiveMaterial {
    color: Vec3,
    ior: f32,
}

impl RefractiveMaterial {
    pub fn new(color: Vec3, ior: f32) -> Self {
        Self { color, ior }
    }

    pub fn reflectance(&self, normal: Vec3, ray: Ray) -> f32 {
        //using schlick's approximation
        let mut r0 = (1.0 - self.ior) / (1.0 + self.ior);
        r0 *= r0;
        let cos_theta = normal.dot(-ray.orientation);
        r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
    }

    pub fn reflect(&self, normal: Vec3, ray: Ray) -> Vec3 {
        let old_ray = ray.orientation;

        old_ray - normal * old_ray.dot(normal) * 2.0
    }

    pub fn refract(&self, normal: Vec3, ray: Ray) -> Vec3 {
        let front_face = ray.orientation.dot(normal) < 0.0;

        let refraction_ratio = if front_face { 1.0 / self.ior } else { self.ior };
        let cos_theta = normal.dot(-ray.orientation);
        let r1 = (ray.orientation + normal * cos_theta) * refraction_ratio;
        let r2 = -normal * (1.0 - r1.length_squared()).sqrt();
        r1 + r2
    }
}

impl Material for RefractiveMaterial {
    fn get_color(&self, next_ray_color: Vec3, _normal: Vec3, _uv: (f32, f32), _ray_dir: Vec3) -> Vec3 {
        next_ray_color * self.color
    }

    fn get_next_ray_dir(
        &self,
        seed: &mut u32,
        ray: Ray,
        normal: Vec3,
    ) -> RayReturn {
        let front_face = ray.orientation.dot(normal) < 0.0;

        let refraction_ratio = if front_face { 1.0 / self.ior } else { self.ior };
        let cos_theta = normal.dot(-ray.orientation);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let reflectance = self.reflectance(normal, ray);
        if refraction_ratio * sin_theta > 1.0 || reflectance > rand_float(seed, (0.0, 1.0)) {
            RayReturn {
                state: RayReturnState::Ray,
                direction: self.reflect(normal, ray),
            }
        } else {
            RayReturn {
                state: RayReturnState::Ray,
                direction: self.refract(normal, ray),
            }
        }
    }
}

pub struct UVMaterial {
}

impl UVMaterial {
    pub fn new() -> Self {
        Self { }
    }
}

impl Material for UVMaterial {
    fn get_stop_color(&self, _normal: Vec3, uv: (f32, f32), _ray_dir: Vec3) -> Vec3 {
        Vec3::new(
            uv.0,
            uv.1,
            0.0,
        )
    }

    fn get_next_ray_dir(
        &self,
        _seed: &mut u32,
        _ray: Ray,
        _normal: Vec3,
    ) -> RayReturn {
        RayReturn {
            state: RayReturnState::Stop,
            direction: Vec3::default(),
        }
    }
}
