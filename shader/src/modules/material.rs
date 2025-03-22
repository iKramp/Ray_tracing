use super::hit::*;
//use image::GenericImageView;
use super::rand_float;
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
    pub ray: Vec3,
}

fn mult_colors(lhs: &Vec3, rhs: &Vec3) -> Vec3 {
    Vec3::new(
        lhs.x * rhs.x / 255.0,
        lhs.y * rhs.y / 255.0,
        lhs.z * rhs.z / 255.0,
    )
}

fn rand_vec_in_unit_cube(seed: &mut u32) -> Vec3 {
    Vec3::new(
        rand_float(seed, (-1.0, 1.0)),
        rand_float(seed, (-1.0, 1.0)),
        rand_float(seed, (-1.0, 1.0)),
    )
}

fn rand_vec_in_unit_sphere(seed: &mut u32) -> Vec3 {
    let mut rand_vec = rand_vec_in_unit_cube(seed);
    while rand_vec.length_squared() > 1.0 {
        rand_vec = rand_vec_in_unit_cube(seed);
    }
    rand_vec
}

fn diffuse_ray_direction(record: &HitRecord, seed: &mut u32) -> RayReturn {
    let mut rand_vec = Vec3::new(2.0, 0.0, 0.0);
    while rand_vec.length_squared() > 1.0 {
        rand_vec = rand_vec_in_unit_sphere(seed);
    }
    rand_vec = rand_vec.normalize();

    RayReturn {
        state: RayReturnState::Ray,
        ray: Vec3::new(
            record.normal.x + rand_vec.x,
            record.normal.y + rand_vec.y,
            record.normal.z + rand_vec.z,
        ),
    }
}

pub trait Material {
    ///gets color based on its own properties and the incoming color
    fn get_color(&self, _record: &HitRecord, _next_ray_color: &Vec3) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
    ///gets next modules direction (not absolute position in world) and returns it
    fn get_next_ray_dir(&self, record: &HitRecord, seed: &mut u32) -> RayReturn;
    ///gets color without the incoming color, as if the modules stopped there
    fn get_stop_color(&self, _record: &HitRecord) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
}

use shared::materials::DiffuseMaterial;

impl Material for DiffuseMaterial {
    fn get_color(&self, _record: &HitRecord, next_ray_color: &Vec3) -> Vec3 {
        mult_colors(next_ray_color, &self.color)
    }

    fn get_next_ray_dir(&self, record: &HitRecord, seed: &mut u32) -> RayReturn {
        diffuse_ray_direction(record, seed)
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
    fn get_color(&self, _record: &HitRecord, next_ray_color: &Vec3) -> Vec3 {
        mult_colors(next_ray_color, &self.color)
    }

    fn get_next_ray_dir(&self, record: &HitRecord, seed: &mut u32) -> RayReturn {
        let old_ray = record.ray.orientation;
        let mut new_ray = old_ray - record.normal * old_ray.dot(record.normal) * 2.0;
        new_ray = new_ray.normalize() + rand_vec_in_unit_sphere(seed) * self.roughness;
        if new_ray.dot(record.normal) > 0.0 {
            RayReturn {
                state: RayReturnState::Ray,
                ray: new_ray,
            }
        } else {
            RayReturn {
                state: RayReturnState::Absorb,
                ray: Vec3::default(),
            }
        }
    }
}

pub struct NormalMaterial {}

impl Material for NormalMaterial {
    fn get_next_ray_dir(&self, _record: &HitRecord, _seed: &mut u32) -> RayReturn {
        RayReturn {
            state: RayReturnState::Stop,
            ray: Vec3::default(),
        }
    }

    fn get_stop_color(&self, record: &HitRecord) -> Vec3 {
        Vec3::new(
            (record.normal.x + 1.0) * 255.0 / 2.0,
            (record.normal.y + 1.0) * 255.0 / 2.0,
            (record.normal.z + 1.0) * 255.0 / 2.0,
        )
    }
}

pub struct BackgroundMaterial {}

impl Material for BackgroundMaterial {
    fn get_next_ray_dir(&self, _record: &HitRecord, _seed: &mut u32) -> RayReturn {
        RayReturn {
            state: RayReturnState::Stop,
            ray: Vec3::default(),
        }
    }

    fn get_stop_color(&self, record: &HitRecord) -> Vec3 {
        let mut temp = record.ray;
        temp.normalize();
        let factor = (temp.orientation.y + 0.5).clamp(0.0, 0.0);
        Vec3::new(255.0, 255.0, 255.0) * (1.0 - factor)
            + Vec3::new(127.5, 178.5, 255.0) * factor
    }
}

pub struct EmmissiveMaterial {
    pub light_color: Vec3,
}

impl EmmissiveMaterial {
    pub fn new(light_color: Vec3) -> Self {
        Self { light_color }
    }
}

impl Material for EmmissiveMaterial {
    fn get_next_ray_dir(&self, _record: &HitRecord, _seed: &mut u32) -> RayReturn {
        RayReturn {
            state: RayReturnState::Stop,
            ray: Vec3::default(),
        }
    }

    fn get_stop_color(&self, record: &HitRecord) -> Vec3 {
        let _ray_reversed = -record.ray.orientation;
        self.light_color /* * ray_reversed.dot(record.normal).sqrt()*/
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

    pub fn reflectance(&self, record: &HitRecord) -> f32 {
        //using schlick's approximation
        let mut r0 = (1.0 - self.ior) / (1.0 + self.ior);
        r0 *= r0;
        let cos_theta = record.normal.dot(-record.ray.orientation);
        r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
    }

    pub fn reflect(&self, record: &HitRecord) -> Vec3 {
        //println!("reflect");
        let old_ray = record.ray.orientation;

        old_ray - record.normal * old_ray.dot(record.normal) * 2.0
    }

    pub fn refract(&self, record: &HitRecord) -> Vec3 {
        //could do without refraction ratio but it's pointless to recalculate
        let refraction_ratio = if record.front_face {
            1.0 / self.ior
        } else {
            self.ior
        };
        let cos_theta = record.normal.dot(-record.ray.orientation);
        let r1 = (record.ray.orientation + record.normal * cos_theta) * refraction_ratio;
        let r2 = -record.normal * (1.0 - r1.length_squared()).sqrt();
        r1 + r2
    }
}

impl Material for RefractiveMaterial {
    fn get_color(&self, _record: &HitRecord, next_ray_color: &Vec3) -> Vec3 {
        mult_colors(next_ray_color, &self.color)
    }

    fn get_next_ray_dir(&self, record: &HitRecord, seed: &mut u32) -> RayReturn {
        let refraction_ratio = if record.front_face {
            1.0 / self.ior
        } else {
            self.ior
        };
        let cos_theta = record.normal.dot(-record.ray.orientation);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let reflectance = self.reflectance(record);
        if refraction_ratio * sin_theta > 1.0 || reflectance > rand_float(seed, (0.0, 1.0)) {
            RayReturn {
                state: RayReturnState::Ray,
                ray: self.reflect(record),
            }
        } else {
            RayReturn {
                state: RayReturnState::Ray,
                ray: self.refract(record),
            }
        }
    }
}

pub struct UVMaterial {
    _color: Vec3,
}

impl UVMaterial {
    pub fn new(_color: Vec3) -> Self {
        Self { _color }
    }
}

impl Material for UVMaterial {
    fn get_color(&self, _record: &HitRecord, _next_ray_color: &Vec3) -> Vec3 {
        /*let image = &record.resources.earth;
        let uv: (f32, f32) = (
            (record.uv.0) * image.width() as f32,
            (record.uv.1) * image.height() as f32,
        );
        let pixel = image.get_pixel(uv.0 as u32, uv.1 as u32);

        let color = vector3d::new(
            *pixel.0.first().unwrap() as f32,
            *pixel.0.get(1).unwrap() as f32,
            *pixel.0.get(2).unwrap() as f32,
        );
        mult_colors(color, next_ray_color)*/
        Vec3::default()
    }

    fn get_next_ray_dir(&self, record: &HitRecord, seed: &mut u32) -> RayReturn {
        diffuse_ray_direction(record, seed)
    }
}
