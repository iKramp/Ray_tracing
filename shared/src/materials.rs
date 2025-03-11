pub struct DiffuseMaterial {
    pub color: super::Vector3d,
}

impl DiffuseMaterial {
    pub const fn new(color: super::Vector3d) -> Self {
        DiffuseMaterial { color }
    }
}
