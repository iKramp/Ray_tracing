pub mod vulkan;
pub mod bvh;
pub mod buffers;
pub mod point_recorder;
use glam::Vec3;
use nalgebra_glm::Vec4;
use shared::{glam::Affine3A, *};
use buffers::*;
pub use point_recorder::record_points;

use crate::{modules::buffers::BufferHolder, HEIGHT, WIDTH};

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

pub struct SceneBuilder {
    sun_orientation: Vec3,
    buffers: BufferHolder,
}

impl SceneBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut builder = SceneBuilder {
            buffers: BufferHolder::new(),
            sun_orientation: Vec3::new(1.0, -1.0, 1.0),
        };

        builder.buffers.insert("acc_image", vec![Vec4::zeros(); WIDTH * HEIGHT]);
        builder.buffers.insert(VERT_BUFFER, Vec::<Vertex>::new());
        builder.buffers.insert(TRI_BUFFER, Vec::<(u32, u32, u32)>::new());
        builder.buffers.insert(BVH_BUFFER, Vec::<Bvh>::new());
        builder.buffers.insert(OBJ_BUFFER, Vec::<Object>::new());
        builder.buffers.insert(INSTANCE_BUFFER, Vec::<Instance>::new());
        builder.buffers.insert(DEBUG_POINTS_BUFFER, vec![Vec3::ZERO; 2]);
        builder
    }

    pub fn add_obj_file(mut self, file: &str, instance_matrices: &[Affine3A]) -> Self {
        let (vertices, mut tris) = parse_obj_file(file);
        println!(
            "Adding {} vertices and {} triangles from OBJ file",
            vertices.len(),
            tris.len()
        );
        let bvh = bvh::create_bvh(&vertices, tris.as_mut());

        let vert_offset = self.buffers.get_length_unchecked(VERT_BUFFER) as u32;
        let bvh_offset = self.buffers.get_length_unchecked(BVH_BUFFER) as u32;
        let tri_offset = self.buffers.get_length_unchecked(TRI_BUFFER) as u32;
        let object_offset = self.buffers.get_length_unchecked(OBJ_BUFFER) as u32;
        let instance_offset = self.buffers.get_length_unchecked(INSTANCE_BUFFER) as u32;

        //print all offsets
        println!(
            "Offsets: vertices {}, bvh {}, triangles {}, objects {}, instances {}",
            vert_offset, bvh_offset, tri_offset, object_offset, instance_offset
        );

        self.buffers.append(VERT_BUFFER, &vertices);
        
        self.buffers.append(TRI_BUFFER, &tris.iter().map(|(v1, v2, v3)| {
            (v1 + vert_offset, v2 + vert_offset, v3 + vert_offset)
        }).collect::<Vec<(u32, u32, u32)>>());
        
        self.buffers.append(BVH_BUFFER, &bvh.iter().map(|bvh_node| {
            let mut new_node = bvh_node.clone();
            if matches!(new_node.mode, ChildTriangleMode::Children) {
                new_node.child_1_or_first_tri += bvh_offset;
                new_node.child_2_or_last_tri += bvh_offset;
            } else if matches!(new_node.mode, ChildTriangleMode::Triangles) {
                new_node.child_1_or_first_tri += tri_offset;
                new_node.child_2_or_last_tri += tri_offset;
            }
            new_node
        }).collect::<Vec<Bvh>>());

        self.buffers.append(OBJ_BUFFER, &[Object {
            bvh_root: bvh_offset,
        }]);

        self.buffers.append(INSTANCE_BUFFER, &instance_matrices.iter().map(|m| Instance {
            transform: *m,
            object_id: object_offset,
        }).collect::<Vec<Instance>>());

        self
    }

    pub fn sun_orientation(mut self, orientation: Vec3) -> Self {
        self.sun_orientation = orientation;
        self
    }

    pub fn build(self) -> (SceneInfo, BufferHolder) {
        let scene_info = SceneInfo {
            num_instances: self.buffers.get_length_unchecked(INSTANCE_BUFFER) as u32,
            num_bvh_nodes: self.buffers.get_length_unchecked(BVH_BUFFER) as u32,
            num_triangles: self.buffers.get_length_unchecked(TRI_BUFFER) as u32,
            sun_orientation: self.sun_orientation,
        };

        (scene_info, self.buffers)
    }
}

