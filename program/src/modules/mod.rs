#![allow(unexpected_cfgs)]

pub mod buffers;
pub mod bvh;
pub mod point_recorder;
pub mod vulkan;
use buffers::*;
use glam::Vec3;
use nalgebra_glm::Vec4;
pub use point_recorder::record_points;
use shared::{glam::{Affine3A, UVec3}, *};
use tracer::modules::trace::Ray;

use crate::{modules::buffers::BufferHolder, HEIGHT, WIDTH};

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
                            .overflowing_sub(1).0;
                        let vn_idx = indices
                            .get(2)
                            .unwrap_or(&"")
                            .parse::<u32>()
                            .unwrap_or(0)
                            .overflowing_sub(1).0;
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
                uvs.push(Vec2Aligned::new(glam::Vec2::new(u, v)));
            }
            _ => {}
        }
    }

    (vertices, faces, normals, uvs)
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
        builder.buffers.insert("acc_per_pixel", vec![Vec4::zeros(); WIDTH * HEIGHT]);
        builder.buffers.insert(VERT_BUFFER, Vec::<Vertex>::new());
        builder.buffers.insert(NORMAL_BUFFER, Vec::<Vec3Aligned>::new());
        builder.buffers.insert(UV_BUFFER, Vec::<Vec2Aligned>::new());
        builder.buffers.insert(TRI_BUFFER, Vec::<Face>::new());
        builder.buffers.insert(BVH_BUFFER, Vec::<Bvh>::new());
        builder.buffers.insert(OBJ_BUFFER, Vec::<Object>::new());
        builder.buffers.insert(INSTANCE_BUFFER, Vec::<Instance>::new());
        builder.buffers.insert(RAY_STATE_BUFFER, vec![Ray::NAN; WIDTH * HEIGHT]);
        builder.buffers.insert(DEBUG_POINTS_BUFFER, vec![Vec3Aligned::new(Vec3::ZERO); 2]);
        builder
    }

    pub fn add_obj_file(mut self, file: &str, instance_matrices: &[Affine3A]) -> Self {
        let (vertices, mut tris, normals, uvs) = parse_obj_file(file);
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
        let normal_offset = self.buffers.get_length_unchecked(NORMAL_BUFFER) as u32;
        let uv_offset = self.buffers.get_length_unchecked(UV_BUFFER) as u32;

        //print all offsets
        println!(
            "Offsets: vertices {}, bvh {}, triangles {}, objects {}, instances {}",
            vert_offset, bvh_offset, tri_offset, object_offset, instance_offset
        );

        self.buffers.append(VERT_BUFFER, &vertices);
        self.buffers.append(NORMAL_BUFFER, &normals);
        self.buffers.append(UV_BUFFER, &uvs);

        self.buffers.append(
            TRI_BUFFER,
            &tris
                .iter()
                .map(|face| Face {
                    vert: UVec3::new(
                        face.vert.x + vert_offset,
                        face.vert.y + vert_offset,
                        face.vert.z + vert_offset,
                    ),
                    uv: if face.uv.x != u32::MAX {UVec3::new(
                        face.uv.x + uv_offset,
                        face.uv.y + uv_offset,
                        face.uv.z + uv_offset,
                    )} else {UVec3::new(u32::MAX, u32::MAX, u32::MAX)},
                    normal: if face.normal.x != u32::MAX {UVec3::new(
                        face.normal.x + normal_offset,
                        face.normal.y + normal_offset,
                        face.normal.z + normal_offset,
                    )} else {UVec3::new(u32::MAX, u32::MAX, u32::MAX)},
                    #[cfg(not(target_arch = "spirv"))]
                    _padding_1: [0; 4],
                    #[cfg(not(target_arch = "spirv"))]
                    _padding_2: [0; 4],
                    #[cfg(not(target_arch = "spirv"))]
                    _padding_3: [0; 4],
                })
                .collect::<Vec<Face>>(),
        );

        self.buffers.append(
            BVH_BUFFER,
            &bvh.iter()
                .map(|bvh_node| {
                    let mut new_node = bvh_node.clone();
                    if matches!(new_node.mode, ChildTriangleMode::Children) {
                        new_node.child_1_or_first_tri += bvh_offset;
                        new_node.child_2_or_last_tri += bvh_offset;
                    } else if matches!(new_node.mode, ChildTriangleMode::Triangles) {
                        new_node.child_1_or_first_tri += tri_offset;
                        new_node.child_2_or_last_tri += tri_offset;
                    }
                    new_node
                })
                .collect::<Vec<Bvh>>(),
        );

        self.buffers.append(
            OBJ_BUFFER,
            &[Object {
                bvh_root: bvh_offset,
            }],
        );

        self.buffers.append(
            INSTANCE_BUFFER,
            &instance_matrices
                .iter()
                .map(|m| Instance {
                    transform: *m,
                    object_id: object_offset,
                })
                .collect::<Vec<Instance>>(),
        );

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
