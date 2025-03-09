pub mod vulkan;

use shared::*;

pub fn parse_obj_file(file: &str, transform: Vector3d) -> (Vec<Vertex>, Vec<(u32, u32, u32)>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut faces: Vec<(u32, u32, u32)> = Vec::new();

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
                let v1 = line.next().unwrap().parse::<u32>().unwrap() - 1;
                let v2 = line.next().unwrap().parse::<u32>().unwrap() - 1;
                let v3 = line.next().unwrap().parse::<u32>().unwrap() - 1;
                faces.push((v1, v2, v3));
            }
            _ => {}
        }
    }

    (vertices, faces)
}
