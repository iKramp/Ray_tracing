use super::hit::HitObject;
use super::hit::{Mesh, Vertex};
use super::material::NormalMaterial;
use super::trace::Ray;
use std::rc::Rc;
use vector3d::Vector3d;

fn parse_obj_file(path: &str, scale: f64, transform: Vector3d<f64>) -> Box<dyn HitObject> {
    let _objects: Vec<Box<dyn HitObject>> = Vec::new();
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut faces: Vec<(usize, usize, usize)> = Vec::new();

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
                let v1 = line.next().unwrap().parse::<usize>().unwrap() - 1;
                let v2 = line.next().unwrap().parse::<usize>().unwrap() - 1;
                let v3 = line.next().unwrap().parse::<usize>().unwrap() - 1;
                faces.push((v1, v2, v3));
            }
            _ => {}
        }
    }

    Box::new(Mesh::new(
        vertices,
        faces,
        Box::new(Rc::new(NormalMaterial {})),
    ))
}

pub struct CamData {
    pub canvas_width: usize,
    pub canvas_height: usize,
    pub fov: f64,
    pub transform: Ray,
    pub samples: u32,
}

impl Default for CamData {
    fn default() -> Self {
        Self {
            canvas_width: 1280,  //247,  //1280, 498
            canvas_height: 720, //140, //720, 280
            fov: 30.0,
            transform: Ray {
                pos: Vector3d::new(0.0, 0.5, -5.0),
                orientation: Vector3d::new(0.0, 0.0, 1.0),
            },
            samples: 1,
        }
    }
}

pub struct SceneInfo {
    pub sun_orientation: Vector3d<f64>,
    pub hittable_objects: Vec<Box<dyn HitObject>>,
}

impl Default for SceneInfo {
    fn default() -> Self {
        SceneInfo {
            sun_orientation: Vector3d::new(1.0, -1.0, 1.0),
            hittable_objects: vec![parse_obj_file(
                "src/resources/teapot.obj",
                0.25,
                Vector3d::new(-0.05, 0.25, 0.0),
            )],
        }
    }
}

pub struct Resources {
    pub earth: image::DynamicImage,
}

fn col_from_frac(r: f64, g: f64, b: f64) -> Vector3d<f64> {
    Vector3d::new(r * 255.0, g * 255.0, b * 255.0)
}