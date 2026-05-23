use shared::{
    glam::{UVec3, Vec2, Vec3},
    Face, Vec2Aligned, Vec3Aligned, Vertex,
};

pub fn parse_obj_file(file: &str) -> (Vec<Vertex>, Vec<Face>, Vec<Vec3Aligned>, Vec<Vec2Aligned>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    let mut normals: Vec<Vec3Aligned> = Vec::new();
    let mut uvs: Vec<Vec2Aligned> = Vec::new();

    for line in file.lines() {
        let mut line = line.split_whitespace();
        match line.next() {
            Some("v") => {
                let x = line.next().unwrap().parse::<f32>().unwrap();
                let y = line.next().unwrap().parse::<f32>().unwrap();
                let z = line.next().unwrap().parse::<f32>().unwrap();
                vertices.push(Vertex::new(Vec3::new(x, y, z)));
            }
            Some("f") => {
                let obj_faces = line
                    .map(|part| {
                        let indices: Vec<&str> = part.split('/').collect();
                        let v_idx = indices[0].parse::<u32>().unwrap() - 1;
                        let vt_idx = indices
                            .get(1)
                            .unwrap_or(&"")
                            .parse::<u32>()
                            .unwrap_or(0)
                            .overflowing_sub(1)
                            .0;
                        let vn_idx = indices
                            .get(2)
                            .unwrap_or(&"")
                            .parse::<u32>()
                            .unwrap_or(0)
                            .overflowing_sub(1)
                            .0;
                        (v_idx, vt_idx, vn_idx)
                    })
                    .collect::<Vec<(u32, u32, u32)>>();
                let first_vert = obj_faces[0];
                for verts in obj_faces.windows(2).skip(1) {
                    let second_vert = verts[0];
                    let third_vert = verts[1];
                    faces.push(Face {
                        vert: UVec3::new(first_vert.0, second_vert.0, third_vert.0),
                        uv: UVec3::new(first_vert.1, second_vert.1, third_vert.1),
                        normal: UVec3::new(first_vert.2, second_vert.2, third_vert.2),
                        #[cfg(not(target_arch = "spirv"))]
                        _padding_1: [0; 4],
                        #[cfg(not(target_arch = "spirv"))]
                        _padding_2: [0; 4],
                        #[cfg(not(target_arch = "spirv"))]
                        _padding_3: [0; 4],
                    });
                }
            }
            Some("vn") => {
                let x = line.next().unwrap().parse::<f32>().unwrap();
                let y = line.next().unwrap().parse::<f32>().unwrap();
                let z = line.next().unwrap().parse::<f32>().unwrap();
                normals.push(Vec3Aligned::new(Vec3::new(x, y, z)));
            }
            Some("vt") => {
                let u = line.next().unwrap().parse::<f32>().unwrap();
                let v = line.next().unwrap().parse::<f32>().unwrap();
                uvs.push(Vec2Aligned::new(Vec2::new(u, v)));
            }
            _ => {}
        }
    }

    (vertices, faces, normals, uvs)
}
