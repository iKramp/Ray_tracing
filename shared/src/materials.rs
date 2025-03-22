use spirv_std::glam::Vec3;

pub struct DiffuseMaterial {
    pub color: Vec3,
}

impl DiffuseMaterial {
    pub const fn new(color: Vec3) -> Self {
        DiffuseMaterial { color }
    }
}
