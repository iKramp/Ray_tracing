pub struct DiffuseMaterial {
    pub color: super::Vector3d,
}

impl DiffuseMaterial {
    pub fn new(color: super::Vector3d) -> Self {
        DiffuseMaterial { color }
    }
}
