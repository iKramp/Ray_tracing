pub mod vulkan;
pub mod bvh;
use glam::Vec3;
use shared::*;

pub fn parse_obj_file(file: &str) -> (Vec<Vertex>, Vec<(u32, u32, u32)>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut faces: Vec<(u32, u32, u32)> = Vec::new();

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
                let vertices: Vec<&str> = line.collect();
                let v1 = vertices[0].split('/').next().unwrap().parse::<i32>().unwrap() - 1;
                let mut prev = vertices[1].split('/').next().unwrap().parse::<i32>().unwrap() - 1;
                for v in vertices.iter().skip(2) {
                    let v2 = v.split('/').next().unwrap().parse::<i32>().unwrap() - 1;
                    let v1_u32 = if v1 < 0 { (vertices.len() as i32 + v1) as u32 } else { v1 as u32 };
                    let prev_u32 = if prev < 0 { (vertices.len() as i32 + prev) as u32 } else { prev as u32 };
                    let v2_u32 = if v2 < 0 { (vertices.len() as i32 + v2) as u32 } else { v2 as u32 };
                    faces.push((v1_u32, prev_u32, v2_u32));
                    prev = v2;
                }
            }
            _ => {}
        }
    }
    (vertices, faces)
}
