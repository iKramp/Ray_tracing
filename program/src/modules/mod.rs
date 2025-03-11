pub mod vulkan;

use shared::*;

pub fn parse_obj_file(file: &str, transform: Vector3d) -> (Vec<Vertex>, Vec<(u32, u32, u32)>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut faces: Vec<(u32, u32, u32)> = Vec::new();
    print!("file: {}", file);

    for line in file.lines() {
        let mut line = line.split_whitespace();
        match line.next() {
            Some("v") => {
                let x = line.next().unwrap().parse::<f64>().unwrap() + transform.x;
                let y = line.next().unwrap().parse::<f64>().unwrap() + transform.y;
                let z = line.next().unwrap().parse::<f64>().unwrap() + transform.z;
                vertices.push(Vertex::new(Vector3d::new(x, y, z), (0.0, 0.0)));
            }
            Some("f") => {
                let vertices: Vec<&str> = line.collect();
                let v1 = vertices[0].split('/').next().unwrap().parse::<u32>().unwrap() - 1;
                let mut prev = vertices[1].split('/').next().unwrap().parse::<u32>().unwrap() - 1;
                for v in vertices.iter().skip(2) {
                    let v2 = v.split('/').next().unwrap().parse::<u32>().unwrap() - 1;
                    faces.push((v1, prev, v2));
                    prev = v2;
                }
            }
            _ => {}
        }
    }

    (vertices, faces)
}
