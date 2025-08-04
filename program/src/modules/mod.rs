pub mod vulkan;
pub mod bvh;
use glam::Vec3;
use shared::{glam::Affine3A, *};

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
    vertices: Vec<Vertex>,
    tris: Vec<(u32, u32, u32)>,
    bvh: Vec<Bvh>,
    instance: Vec<Instance>,
    objects: Vec<Object>,
    sun_orientation: Vec3,
}

impl SceneBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        SceneBuilder {
            vertices: Vec::new(),
            tris: Vec::new(),
            bvh: Vec::new(),
            instance: Vec::new(),
            objects: Vec::new(),
            sun_orientation: Vec3::new(1.0, -1.0, 1.0),
        }
    }

    pub fn add_obj_file(mut self, file: &str, instance_matrices: &[Affine3A]) -> Self {
        let (mut vertices, mut tris) = parse_obj_file(file);
        println!(
            "Adding {} vertices and {} triangles from OBJ file",
            vertices.len(),
            tris.len()
        );
        let bvh = bvh::create_bvh(&vertices, tris.as_mut());

        let vert_offset = self.vertices.len() as u32;
        let bvh_offset = self.bvh.len() as u32;
        let tri_offset = self.tris.len() as u32;
        let object_offset = self.objects.len() as u32;
        let instance_offset = self.instance.len() as u32;

        //print all offsets
        println!(
            "Offsets: vertices {}, bvh {}, triangles {}, objects {}, instances {}",
            vert_offset, bvh_offset, tri_offset, object_offset, instance_offset
        );

        self.vertices.append(&mut vertices);
        for (v1, v2, v3) in tris {
            self.tris.push((v1 + vert_offset, v2 + vert_offset, v3 + vert_offset));
        }
        for mut bvh_node in bvh {
            if matches!(bvh_node.mode, ChildTriangleMode::Children) {
                bvh_node.child_1_or_first_tri += bvh_offset;
                bvh_node.child_2_or_last_tri += bvh_offset;
            } else if matches!(bvh_node.mode, ChildTriangleMode::Triangles) {
                bvh_node.child_1_or_first_tri += tri_offset;
                bvh_node.child_2_or_last_tri += tri_offset;
            }
            self.bvh.push(bvh_node);

        }
        self.objects.push(Object {
            bvh_root: bvh_offset,
        });
        self.instance.extend(
            instance_matrices
                .iter()
                .map(|m| Instance {
                    transform: *m,
                    object_id: object_offset,
                })
        );

        self
    }

    pub fn sun_orientation(mut self, orientation: Vec3) -> Self {
        self.sun_orientation = orientation;
        self
    }

    pub fn build(self) -> (SceneInfo, BufferSceneInfo) {
        let scene_info = SceneInfo {
            num_instances: self.instance.len() as u32,
            num_bvh_nodes: self.bvh.len() as u32,
            num_triangles: self.tris.len() as u32,
            sun_orientation: self.sun_orientation,
        };

        let buffer_scene_info = BufferSceneInfo {
            vertices: self.vertices,
            triangles: self.tris,
            bvh: self.bvh,
            instances: self.instance,
            objects: self.objects,
        };

        (scene_info, buffer_scene_info)
    }
}

pub struct BufferSceneInfo {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<(u32, u32, u32)>,
    pub bvh: Vec<Bvh>,
    pub instances: Vec<Instance>,
    pub objects: Vec<Object>,
}
