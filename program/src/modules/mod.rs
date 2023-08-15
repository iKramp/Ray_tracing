pub mod vulkan;

use sdl2::keyboard::Keycode::F;
use shared::*;

pub fn parse_obj_file(path: &str, scale: f64, transform: Vector3d) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut faces: Vec<(u32, u32, u32)> = Vec::new();

    let file = std::fs::read_to_string(path).unwrap();
    for line in file.lines() {
        let mut line = line.split_whitespace();
        match line.next() {
            Some("v") => {
                let x = line.next().unwrap().parse::<f64>().unwrap() * scale + transform.x;
                let y = line.next().unwrap().parse::<f64>().unwrap() * scale + transform.y;
                let z = line.next().unwrap().parse::<f64>().unwrap() * scale + transform.z;
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

}

pub struct Vertex {
    #[allow(dead_code)]
    pub(crate) pos: Vector3d,
    #[allow(dead_code)]
    uv: (f64, f64),
}

impl Vertex {
    pub fn new(pos: Vector3d, uv: (f64, f64)) -> Self {
        Vertex { pos, uv }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            pos: Vector3d::default(),
            uv: (0.0, 0.0),
        }
    }
}