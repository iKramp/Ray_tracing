use core::f32;
use core::f32::consts::PI;

use crate::modules::is_vec_3_nan;
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

fn diffuse_ray_direction(seed: &mut u32, normal: Vec3) -> Vec3 {
    let rand_vec = rand_vec_in_unit_sphere(seed);
    let res = rand_vec + normal;

    if res.length_squared() < f32::EPSILON {
        //if the random vector is too close to zero, just return the normal
        normal
    } else {
        res.normalize()
    }
}

pub struct MaterialReturn {
    pub ray_return_state: RayReturnState,
    pub new_ray: Ray,
    pub next_color: Vec3,
}

pub trait Material {
    fn backface_culling(&self) -> bool {
        true
    }

    fn bxdf(
        &self,
        curr_color: Vec3,
        in_ray: Ray,
        normal: Vec3,
        uv: (f32, f32),
        t: f32,
        seed: &mut u32,
    ) -> MaterialReturn;
}

pub struct GenericMaterial {
    pub color: Vec3,
    pub specular: f32,
    pub specular_roughness: f32,
    pub roughness: f32,
    pub ior: f32,
}

impl GenericMaterial {
    fn reflect(in_dir: Vec3, normal: Vec3, roughness: f32, seed: &mut u32) -> Vec3 {
        let dot_product = in_dir.dot(normal);
        let mut new_ray = (in_dir - normal * (2.0 * dot_product)).normalize();
        let rand_vec = diffuse_ray_direction(seed, normal).normalize();
        new_ray = new_ray.lerp(rand_vec, roughness);
        new_ray
    }

    fn can_refract(curr_ior: f32, next_ior: f32, curr_sintheta: f32) -> bool {
        let sintheta2 = curr_ior / next_ior * curr_sintheta;
        sintheta2 < 1.0
    }

    fn schlick_reflectance(ior1: f32, ior2: f32, cos_theta: f32) -> f32 {
        let r0 = (ior1 - ior2) / (ior1 + ior2);
        let r0_squared = r0 * r0;
        r0_squared + (1.0 - r0_squared) * (1.0 - cos_theta).powi(5)
    }

    fn reflect_specular(&self, curr_color: Vec3, in_ray: Ray, normal: Vec3, t: f32, seed: &mut u32) -> MaterialReturn {
        let new_ray = Self::reflect(in_ray.orientation, normal, self.specular_roughness, seed);

        //color doesn't change
        MaterialReturn {
            ray_return_state: RayReturnState::Ray,
            new_ray: Ray {
                pos: in_ray.pos + in_ray.orientation * t,
                orientation: new_ray,
            },
            next_color: curr_color,
        }
    }

    fn reflect_regular(&self, curr_color: Vec3, in_ray: Ray, normal: Vec3, t: f32, seed: &mut u32) -> MaterialReturn {
        let new_ray = Self::reflect(in_ray.orientation, normal, self.roughness, seed);

        MaterialReturn {
            ray_return_state: RayReturnState::Ray,
            new_ray: Ray {
                pos: in_ray.pos + in_ray.orientation * t,
                orientation: new_ray,
            },
            next_color: curr_color * self.color,
        }
    }

    fn refract(&self, curr_color: Vec3, in_ray: Ray, normal: Vec3, t: f32, seed: &mut u32) -> MaterialReturn {
        let front_face = in_ray.orientation.dot(normal) < 0.0;
        //normal vector on the incoming side of the surface
        let normal_incoming = if front_face {
            normal
        } else {
            -normal
        };

        let refraction_ratio = if front_face { 1.0 / self.ior } else { self.ior };

        let next_ray_perfect = in_ray.orientation.refract(normal_incoming, refraction_ratio).normalize();

        let next_ray_diffuse = diffuse_ray_direction(seed, normal).normalize();
        let next_ray = next_ray_perfect.lerp(next_ray_diffuse, self.roughness);

        MaterialReturn {
            ray_return_state: RayReturnState::Ray,
            new_ray: Ray {
                pos: in_ray.pos + in_ray.orientation * t,
                orientation: next_ray.normalize(),
            },
            next_color: curr_color
        }
    }
}

impl Material for GenericMaterial {
    fn bxdf(
        &self,
        curr_color: Vec3,
        in_ray: Ray,
        normal: Vec3,
        uv: (f32, f32),
        t: f32,
        seed: &mut u32,
    ) -> MaterialReturn {
        let cos_theta = in_ray.orientation.dot(normal);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        if self.specular > 0.0 {
            let rand_specular = rand_float(seed, (0.0, 1.0));
            if self.specular > rand_specular {
                return self.reflect_specular(curr_color, in_ray, normal, t, seed);
            }
        }

        if self.ior > 0.0 {
            let ior_parts = if in_ray.orientation.dot(normal) < 0.0 {
                (1.0, self.ior)
            } else {
                (self.ior, 1.0)
            };
            let reflectance = Self::schlick_reflectance(ior_parts.0, ior_parts.1, cos_theta.abs()).min(1.0);
            let rand_refract = rand_float(seed, (0.0, 1.0));
            if rand_refract > reflectance && Self::can_refract(ior_parts.0, ior_parts.1, sin_theta) {
                return self.refract(curr_color, in_ray, normal, t, seed);
            } else {
                return self.reflect_specular(curr_color, in_ray, normal, t, seed);
            }
        }

        self.reflect_regular(curr_color, in_ray, normal, t, seed)
    }

    fn backface_culling(&self) -> bool {
        false
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

impl DiffuseMaterial {
    fn get_color(
        &self,
        next_ray_color: Vec3,
        _normal: Vec3,
        _uv: (f32, f32),
        _ray_dir: Vec3,
    ) -> Vec3 {
        next_ray_color * self.color
    }

    fn get_next_ray_dir(&self, seed: &mut u32, ray: Ray, normal: Vec3) -> RayReturn {
        let direction = diffuse_ray_direction(seed, normal);

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

impl MetalMaterial {
    fn get_color(
        &self,
        next_ray_color: Vec3,
        _normal: Vec3,
        _uv: (f32, f32),
        _ray_dir: Vec3,
    ) -> Vec3 {
        next_ray_color * self.color
    }

    fn get_next_ray_dir(&self, seed: &mut u32, ray: Ray, normal: Vec3) -> RayReturn {
        let old_ray = ray.orientation;
        let mut new_ray = old_ray.reflect(normal).normalize();
        let rand_vec = diffuse_ray_direction(seed, normal).normalize();
        new_ray = new_ray.lerp(rand_vec, self.roughness);

        RayReturn {
            state: RayReturnState::Ray,
            direction: new_ray,
        }
    }
}

pub struct NormalMaterial {}

impl NormalMaterial {
    fn get_next_ray_dir(&self, seed: &mut u32, ray: Ray, normal: Vec3) -> RayReturn {
        RayReturn {
            state: RayReturnState::Ray,
            direction: diffuse_ray_direction(seed, normal),
        }
    }

    fn get_color(
        &self,
        next_ray_color: Vec3,
        normal: Vec3,
        _uv: (f32, f32),
        _ray_dir: Vec3,
    ) -> Vec3 {
        let color = {
            if normal.dot(Vec3::new(0.0, 1.0, 0.0)).abs() > 0.9 { //top/bottom
                Vec3::new(1.0, 1.0, 1.0) //white
            } else if normal.dot(Vec3::new(-1.0, 0.0, 0.0)) > 0.9 { //right
                Vec3::new(0.1, 1.0, 0.1) //green
            } else if normal.dot(Vec3::new(1.0, 0.0, 0.0)) > 0.9 { //left
                Vec3::new(0.1, 0.1, 1.0) //red
            } else if normal.dot(Vec3::new(0.0, 0.0, -1.0)) > 0.9 { //front
                Vec3::new(1.0, 0.1, 0.1) //blue
            } else if normal.dot(Vec3::new(0.0, 0.0, 1.0)) > 0.9 { //back
                Vec3::ZERO //black
            } else {
                Vec3::ZERO //default black
            }
        };
        next_ray_color * color
    }
}

impl Material for NormalMaterial {
    fn bxdf(
        &self,
        curr_color: Vec3,
        in_ray: Ray,
        normal: Vec3,
        uv: (f32, f32),
        t: f32,
        seed: &mut u32,
    ) -> MaterialReturn {
        let next_ray_return = self.get_next_ray_dir(seed, in_ray, normal);
        let next_color = self.get_color(curr_color, normal, uv, in_ray.orientation);

        MaterialReturn {
            ray_return_state: next_ray_return.state,
            new_ray: Ray {
                pos: in_ray.pos + in_ray.orientation * t,
                orientation: next_ray_return.direction,
            },
            next_color,
        }
    }
}

pub struct BackgroundMaterial {}

impl BackgroundMaterial {
    fn get_next_ray_dir(&self, _seed: &mut u32, _ray: Ray, _normal: Vec3) -> RayReturn {
        RayReturn {
            state: RayReturnState::Stop,
            direction: Vec3::default(),
        }
    }

    pub fn get_stop_color(&self, _normal: Vec3, _uv: (f32, f32), ray_dir: Vec3) -> Vec3 {
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

    fn get_next_ray_dir(&self, _seed: &mut u32, _ray: Ray, _normal: Vec3) -> RayReturn {
        RayReturn {
            state: RayReturnState::Stop,
            direction: Vec3::default(),
        }
    }

    fn get_stop_color(&self, normal: Vec3, _uv: (f32, f32), ray_dir: Vec3) -> Vec3 {
        let ray_reversed = -ray_dir.normalize();

        let dot_product = ray_reversed.dot(normal).abs(); //abs for weird geometries that have gaps
                                                          //into backface triangles

        self.light_color * dot_product.sqrt()
    }
}

impl Material for EmmissiveMaterial {
    fn bxdf(
        &self,
        curr_color: Vec3,
        in_ray: Ray,
        normal: Vec3,
        uv: (f32, f32),
        t: f32,
        seed: &mut u32,
    ) -> MaterialReturn {
        let next_ray_return = self.get_next_ray_dir(seed, in_ray, normal);
        let next_color = self.get_stop_color(normal, uv, in_ray.orientation);
        MaterialReturn {
            ray_return_state: next_ray_return.state,
            new_ray: Ray {
                pos: in_ray.pos + in_ray.orientation * t,
                orientation: next_ray_return.direction,
            },
            next_color: curr_color * next_color,
        }
    }
}

pub struct RefractiveMaterial {
    _color: Vec3,
    ior: f32,
}

impl RefractiveMaterial {
    pub const fn new(color: Vec3, ior: f32) -> Self {
        Self { _color: color, ior }
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
        old_ray.reflect(normal).normalize()
    }

    pub fn refract(&self, normal: Vec3, ray: Ray) -> Vec3 {
        let front_face = ray.orientation.dot(normal) < 0.0;
        // let in_ray = ray.orientation.normalize();
        //
        // let refraction_ratio = if front_face { 1.0 / self.ior } else { self.ior };
        // let cos_theta = normal.dot(-in_ray);
        // let r1 = (ray.orientation + normal * cos_theta) * refraction_ratio;
        // let r2 = -normal * (1.0 - r1.length_squared()).sqrt();
        // r1 + r2

        let unit_in = ray.orientation.normalize();
        let cos_theta = (-unit_in).dot(normal).min(1.0);

        let refraction_ratio = if front_face { 1.0 / self.ior } else { self.ior };

        // Check for total internal reflection (you said you already handle it)
        let r_out_perp = (unit_in + normal * cos_theta) * refraction_ratio;
        let r_out_parallel = -normal * (1.0 - r_out_perp.length_squared()).abs().sqrt();

        r_out_perp + r_out_parallel
    }

    fn get_color(
        &self,
        next_ray_color: Vec3,
        _normal: Vec3,
        _uv: (f32, f32),
        _ray_dir: Vec3,
    ) -> Vec3 {
        next_ray_color
    }

    fn get_next_ray_dir(&self, seed: &mut u32, ray: Ray, normal: Vec3) -> RayReturn {
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

impl Material for RefractiveMaterial {
    fn bxdf(
        &self,
        curr_color: Vec3,
        in_ray: Ray,
        normal: Vec3,
        uv: (f32, f32),
        t: f32,
        seed: &mut u32,
    ) -> MaterialReturn {
        let next_ray_return = self.get_next_ray_dir(seed, in_ray, normal);
        let next_color = self.get_color(curr_color, normal, uv, in_ray.orientation);

        MaterialReturn {
            ray_return_state: next_ray_return.state,
            new_ray: Ray {
                pos: in_ray.pos + in_ray.orientation * t,
                orientation: next_ray_return.direction,
            },
            next_color,
        }
    }
}

pub struct UVMaterial {}

impl UVMaterial {
    pub fn new() -> Self {
        Self {}
    }

    fn get_stop_color(&self, _normal: Vec3, uv: (f32, f32), _ray_dir: Vec3) -> Vec3 {
        Vec3::new(uv.0, uv.1, 0.0)
    }

    fn get_next_ray_dir(&self, _seed: &mut u32, _ray: Ray, _normal: Vec3) -> RayReturn {
        RayReturn {
            state: RayReturnState::Stop,
            direction: Vec3::default(),
        }
    }
}

impl Material for UVMaterial {
    fn bxdf(
        &self,
        curr_color: Vec3,
        in_ray: Ray,
        normal: Vec3,
        uv: (f32, f32),
        t: f32,
        seed: &mut u32,
    ) -> MaterialReturn {
        let next_ray_return = self.get_next_ray_dir(seed, in_ray, normal);
        let next_color = self.get_stop_color(normal, uv, in_ray.orientation);

        MaterialReturn {
            ray_return_state: next_ray_return.state,
            new_ray: Ray {
                pos: in_ray.pos + in_ray.orientation * t,
                orientation: next_ray_return.direction,
            },
            next_color: curr_color * next_color,
        }
    }
}
