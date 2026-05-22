use core::f32;
use core::f32::consts::PI;

use crate::modules::trace::Ray;

//use image::GenericImageView;
use super::rand_float;
use shared::acos_approx;
use shared::glam::{Vec2, Vec3};
#[allow(unused_imports)] //actually used for .sqrt because we don't allow std
use spirv_std::num_traits::Float;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RayReturnState {
    Killed,
    Stop,
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
        uv: Vec2,
        t: f32,
        seed: &mut u32,
    ) -> MaterialReturn;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GenericMaterial {
    pub color_surface: Vec3,
    #[cfg(not(target_arch = "spirv"))]
    pub padding_1: [u8; 4],
    pub color_emissive: Vec3,
    #[cfg(not(target_arch = "spirv"))]
    pub padding_2: [u8; 4],
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

    fn can_refract(curr_ior: f32, next_ior: f32, cos_theta: f32) -> bool {
        let sin2_theta = 1.0 - cos_theta * cos_theta;
        let ratio = curr_ior / next_ior;
        ratio * ratio * sin2_theta <= 1.0
    }

    fn schlick_reflectance(ior1: f32, ior2: f32, cos_theta: f32) -> f32 {
        let cos_theta = cos_theta.clamp(0.0, 1.0);
        let r0 = (ior1 - ior2) / (ior1 + ior2);
        let r0_squared = r0 * r0;
        r0_squared + (1.0 - r0_squared) * (1.0 - cos_theta).powi(5)
    }

    fn reflect_specular(
        &self,
        curr_color: Vec3,
        in_ray: Ray,
        normal: Vec3,
        t: f32,
        seed: &mut u32,
    ) -> MaterialReturn {
        let new_ray = Self::reflect(in_ray.orientation, normal, self.specular_roughness, seed);

        //color doesn't change
        MaterialReturn {
            ray_return_state: RayReturnState::Ray,
            new_ray: Ray::new(in_ray.pos + in_ray.orientation * t, new_ray),
            next_color: curr_color,
        }
    }

    fn reflect_regular(
        &self,
        curr_color: Vec3,
        in_ray: Ray,
        normal: Vec3,
        t: f32,
        seed: &mut u32,
    ) -> MaterialReturn {
        let new_ray = Self::reflect(in_ray.orientation, normal, self.roughness, seed);

        MaterialReturn {
            ray_return_state: RayReturnState::Ray,
            new_ray: Ray::new(in_ray.pos + in_ray.orientation * t, new_ray),
            next_color: curr_color * self.color_surface,
        }
    }

    fn refract(
        &self,
        curr_color: Vec3,
        in_ray: Ray,
        normal: Vec3,
        t: f32,
        _seed: &mut u32,
    ) -> MaterialReturn {
        let front_face = in_ray.orientation.dot(normal) < 0.0;
        //normal vector on the incoming side of the surface
        let normal_incoming = if front_face { normal } else { -normal };

        let refraction_ratio = if front_face { 1.0 / self.ior } else { self.ior };

        let next_ray_perfect = in_ray
            .orientation
            .refract(normal_incoming, refraction_ratio)
            .normalize();

        MaterialReturn {
            ray_return_state: RayReturnState::Ray,
            new_ray: Ray::new(in_ray.pos + in_ray.orientation * t, next_ray_perfect),
            next_color: curr_color,
        }
    }
}

impl Material for GenericMaterial {
    fn bxdf(
        &self,
        curr_color: Vec3,
        in_ray: Ray,
        normal: Vec3,
        _uv: Vec2,
        t: f32,
        seed: &mut u32,
    ) -> MaterialReturn {
        if self.color_emissive.length() > f32::EPSILON {
            return MaterialReturn {
                ray_return_state: RayReturnState::Stop,
                new_ray: Ray::new(in_ray.pos + in_ray.orientation * t, Vec3::ZERO),
                next_color: curr_color * self.color_emissive,
            };
        }

        let entering = in_ray.orientation.dot(normal) < 0.0;

        if self.specular > f32::EPSILON {
            let rand_specular = rand_float(seed, (0.0, 1.0));
            if self.specular > rand_specular {
                return self.reflect_specular(curr_color, in_ray, normal, t, seed);
            }
        }

        if self.ior > f32::EPSILON {
            let (ior1, ior2, cos_theta) = if entering {
                let cos_theta = -in_ray.orientation.dot(normal);
                (1.0, self.ior, cos_theta)
            } else {
                let cos_theta = in_ray.orientation.dot(normal);
                (self.ior, 1.0, cos_theta)
            };

            let reflectance = Self::schlick_reflectance(ior1, ior2, cos_theta).min(1.0);

            let rand_refract = rand_float(seed, (0.0, 1.0));
            if rand_refract > reflectance && Self::can_refract(ior1, ior2, cos_theta) {
                return self.refract(curr_color, in_ray, normal, t, seed);
            } else {
                return self.reflect_specular(curr_color, in_ray, normal, t, seed);
            }
        }

        self.reflect_regular(curr_color, in_ray, normal, t, seed)
    }

    fn backface_culling(&self) -> bool {
        true
    }
}

pub struct NormalMaterial {}

impl NormalMaterial {
    fn get_next_ray_dir(&self, seed: &mut u32, ray: Ray, normal: Vec3) -> RayReturn {
        //check if backface
        if ray.orientation.dot(normal) > 0.0 {
            return RayReturn {
                state: RayReturnState::Ray,
                direction: ray.orientation,
            };
        }

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
            if normal.dot(Vec3::new(0.0, 1.0, 0.0)).abs() > 0.9 {
                //top/bottom
                Vec3::new(1.0, 1.0, 1.0) //white
            } else if normal.dot(Vec3::new(-1.0, 0.0, 0.0)) > 0.9 {
                //right
                Vec3::new(0.1, 1.0, 0.1) //green
            } else if normal.dot(Vec3::new(1.0, 0.0, 0.0)) > 0.9 {
                //left
                Vec3::new(0.1, 0.1, 1.0) //red
            } else if normal.dot(Vec3::new(0.0, 0.0, -1.0)) > 0.9 {
                //front
                Vec3::new(1.0, 0.1, 0.1) //blue
            } else if normal.dot(Vec3::new(0.0, 0.0, 1.0)) > 0.9 {
                //back
                Vec3::ONE //black
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
        uv: Vec2,
        t: f32,
        seed: &mut u32,
    ) -> MaterialReturn {
        let next_ray_return = self.get_next_ray_dir(seed, in_ray, normal);
        let next_color = self.get_color(curr_color, normal, (uv.x, uv.y), in_ray.orientation);
        //checkerboard pattern, 10x10
        const CHECKER_NUM: f32 = 10.0;
        let checker_square =
            ((uv.x * CHECKER_NUM).floor() as i32 + (uv.y * CHECKER_NUM).floor() as i32) % 2;
        let checker_color = if checker_square == 0 {
            Vec3::new(0.9, 0.9, 0.9) //light gray
        } else {
            Vec3::new(0.5, 0.5, 0.5) //dark gray
        };

        MaterialReturn {
            ray_return_state: next_ray_return.state,
            new_ray: Ray::new(
                in_ray.pos + in_ray.orientation * t,
                next_ray_return.direction,
            ),
            next_color: next_color * checker_color,
        }
    }
}

pub struct BackgroundMaterial {}

impl BackgroundMaterial {
    pub fn get_stop_color(&self, _normal: Vec3, _uv: (f32, f32), ray_dir: Vec3) -> Vec3 {
        let temp = ray_dir.normalize();

        let gradient_factor = (temp.y + 0.5).clamp(0.0, 1.0);
        let brightness_factor = 0.25;
        (Vec3::new(1.0, 1.0, 1.0) * (1.0 - gradient_factor)
            + Vec3::new(0.5, 0.7, 1.0) * gradient_factor)
            * brightness_factor
    }
}
