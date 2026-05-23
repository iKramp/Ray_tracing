struct GltfData {}

pub fn parse_gltf_file(path: &str) -> Option<GltfData> {
    let file_content = std::fs::read_to_string(path).ok()?;

    todo!()
}
