use glam::Vec3;
use nalgebra_glm::Vec4;
use shared::{
    glam::{Affine3A, UVec3},
    *,
};
use tracer::modules::{material::GenericMaterial, trace::Ray};

use crate::{
    modules::{buffers::*, bvh, obj_parser::parse_obj_file},
    HEIGHT, WIDTH,
};

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

        builder
            .buffers
            .insert("acc_image", vec![Vec4::zeros(); WIDTH * HEIGHT]);
        builder
            .buffers
            .insert("acc_per_pixel", vec![Vec4::zeros(); WIDTH * HEIGHT]);
        builder.buffers.insert(VERT_BUFFER, Vec::<Vertex>::new());
        builder
            .buffers
            .insert(NORMAL_BUFFER, Vec::<Vec3Aligned>::new());
        builder.buffers.insert(UV_BUFFER, Vec::<Vec2Aligned>::new());
        builder.buffers.insert(TRI_BUFFER, Vec::<Face>::new());
        builder.buffers.insert(BVH_BUFFER, Vec::<Bvh>::new());
        builder.buffers.insert(OBJ_BUFFER, Vec::<Object>::new());
        builder
            .buffers
            .insert(INSTANCE_BUFFER, Vec::<Instance>::new());
        builder
            .buffers
            .insert(RAY_STATE_BUFFER, vec![Ray::NAN; WIDTH * HEIGHT]);
        builder
            .buffers
            .insert(DEBUG_POINTS_BUFFER, vec![Vec3Aligned::new(Vec3::ZERO); 2]);
        builder
            .buffers
            .insert(IMAGE_INFO_BUFFER, Vec::<ImageInfo>::new());
        builder.buffers.insert(IMAGE_DATA_BUFFER, Vec::<u8>::new());
        builder
            .buffers
            .insert(MATERIAL_BUFFER, Vec::<GenericMaterial>::new());
        builder
    }

    pub fn add_material(mut self, material: GenericMaterial) -> Self {
        self.buffers.append(MATERIAL_BUFFER, &[material]);
        self
    }

    pub fn add_obj_file(
        mut self,
        file: &str,
        instance_matrices: &[Affine3A],
        material_id: u32,
    ) -> Self {
        let (vertices, mut tris, normals, uvs) = parse_obj_file(file);
        println!(
            "Adding {} vertices and {} triangles from OBJ file",
            vertices.len(),
            tris.len()
        );
        let bvh = bvh::create_bvh(&vertices, tris.as_mut());

        self.append_buffers(
            &vertices,
            &normals,
            &uvs,
            &tris,
            &bvh,
            &[Object {
                bvh_root: 0,
                material_id,
                triangle_start: 0,
                num_triangles: tris.len() as u32,
            }],
            &instance_matrices
                .iter()
                .map(|&matrix| Instance {
                    transform: matrix,
                    object_id: 0,
                })
                .collect::<Vec<Instance>>(),
        );

        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn append_buffers(
        &mut self,
        vert_buffer: &[Vertex],
        normal_buffer: &[Vec3Aligned],
        uv_buffer: &[Vec2Aligned],
        tri_buffer: &[Face],
        bvh_buffer: &[Bvh],
        obj_buffer: &[Object],
        instance_buffer: &[Instance],
    ) {
        let vert_offset = self.buffers.get_length_unchecked(VERT_BUFFER) as u32;
        let bvh_offset = self.buffers.get_length_unchecked(BVH_BUFFER) as u32;
        let tri_offset = self.buffers.get_length_unchecked(TRI_BUFFER) as u32;
        let object_offset = self.buffers.get_length_unchecked(OBJ_BUFFER) as u32;
        let normal_offset = self.buffers.get_length_unchecked(NORMAL_BUFFER) as u32;
        let uv_offset = self.buffers.get_length_unchecked(UV_BUFFER) as u32;

        self.buffers.append(VERT_BUFFER, vert_buffer);
        self.buffers.append(NORMAL_BUFFER, normal_buffer);
        self.buffers.append(UV_BUFFER, uv_buffer);

        self.buffers.append(
            TRI_BUFFER,
            &tri_buffer
                .iter()
                .map(|face| Face {
                    vert: UVec3::new(
                        face.vert.x + vert_offset,
                        face.vert.y + vert_offset,
                        face.vert.z + vert_offset,
                    ),
                    uv: if face.uv.x != u32::MAX {
                        UVec3::new(
                            face.uv.x + uv_offset,
                            face.uv.y + uv_offset,
                            face.uv.z + uv_offset,
                        )
                    } else {
                        UVec3::new(u32::MAX, u32::MAX, u32::MAX)
                    },
                    normal: if face.normal.x != u32::MAX {
                        UVec3::new(
                            face.normal.x + normal_offset,
                            face.normal.y + normal_offset,
                            face.normal.z + normal_offset,
                        )
                    } else {
                        UVec3::new(u32::MAX, u32::MAX, u32::MAX)
                    },
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
            &bvh_buffer
                .iter()
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
            &obj_buffer
                .iter()
                .map(|obj| Object {
                    bvh_root: obj.bvh_root + bvh_offset,
                    material_id: obj.material_id,
                    triangle_start: obj.triangle_start + tri_offset,
                    num_triangles: obj.num_triangles,
                })
                .collect::<Vec<Object>>(),
        );

        self.buffers.append(
            INSTANCE_BUFFER,
            &instance_buffer
                .iter()
                .map(|instance| Instance {
                    transform: instance.transform,
                    object_id: instance.object_id + object_offset,
                })
                .collect::<Vec<Instance>>(),
        );
    }

    pub fn add_image(&mut self, size_x: u32, size_y: u32, data: Vec<u8>) -> u32 {
        let image_index = self.buffers.get_length_unchecked(IMAGE_INFO_BUFFER) as u32;
        let data_index = self.buffers.get(IMAGE_DATA_BUFFER).unwrap().data.len() as u32;
        let image_info = ImageInfo {
            data_index,
            width: size_x,
            height: size_y,
        };
        self.buffers.append(IMAGE_INFO_BUFFER, &[image_info]);
        self.buffers.append(IMAGE_DATA_BUFFER, &data);
        image_index
    }

    pub fn sun_orientation(mut self, orientation: Vec3) -> Self {
        self.sun_orientation = orientation;
        self
    }

    pub fn build(mut self) -> (SceneInfo, BufferHolder) {
        let num_emissive_instances = self.sort_by_emission();

        let scene_info = SceneInfo {
            num_instances: self.buffers.get_length_unchecked(INSTANCE_BUFFER) as u32,
            num_bvh_nodes: self.buffers.get_length_unchecked(BVH_BUFFER) as u32,
            num_triangles: self.buffers.get_length_unchecked(TRI_BUFFER) as u32,
            num_materials: self.buffers.get_length_unchecked(MATERIAL_BUFFER) as u32,
            num_emissive_instances: num_emissive_instances as u32,
            sun_orientation: self.sun_orientation,
        };

        (scene_info, self.buffers)
    }

    //returns number of emissive objects
    pub fn sort_by_emission(&mut self) -> usize {
        let mut final_instance_indexes =
            (0..self.buffers.get_num_elements(INSTANCE_BUFFER).unwrap())
                .map(|i| i as u32)
                .collect::<Vec<u32>>();

        let num_materials = self.buffers.get_num_elements(MATERIAL_BUFFER).unwrap() as u32;

        final_instance_indexes.sort_by(|&a, &b| {
            let instance_a = self
                .buffers
                .get_element::<Instance>(INSTANCE_BUFFER, a as usize)
                .unwrap();
            let instance_b = self
                .buffers
                .get_element::<Instance>(INSTANCE_BUFFER, b as usize)
                .unwrap();

            let obj_a = self
                .buffers
                .get_element::<Object>(OBJ_BUFFER, instance_a.object_id as usize)
                .unwrap();
            let obj_b = self
                .buffers
                .get_element::<Object>(OBJ_BUFFER, instance_b.object_id as usize)
                .unwrap();

            if obj_a.material_id >= num_materials {
                panic!()
            }
            if obj_b.material_id >= num_materials {
                panic!()
            }

            let mat_a = self
                .buffers
                .get_element::<GenericMaterial>(MATERIAL_BUFFER, obj_b.material_id as usize)
                .unwrap();
            let mat_b = self
                .buffers
                .get_element::<GenericMaterial>(MATERIAL_BUFFER, obj_a.material_id as usize)
                .unwrap();

            let emission_a = mat_a.color_emissive.length();
            let emission_b = mat_b.color_emissive.length();

            emission_a.partial_cmp(&emission_b).unwrap()
        });

        let mut new_instance_buffer: Vec<Instance> = Vec::new();

        for i in 0..final_instance_indexes.len() {
            let instance = self
                .buffers
                .get_element::<Instance>(INSTANCE_BUFFER, final_instance_indexes[i] as usize)
                .unwrap();
            new_instance_buffer.push(instance.clone());
        }

        self.buffers.insert(INSTANCE_BUFFER, new_instance_buffer);

        println!("Sorted instances by emission: {final_instance_indexes:?}");

        println!("Final instance order: ");
        for &index in &final_instance_indexes {
            let instance = self
                .buffers
                .get_element::<Instance>(INSTANCE_BUFFER, index as usize)
                .unwrap();
            let obj = self
                .buffers
                .get_element::<Object>(OBJ_BUFFER, instance.object_id as usize)
                .unwrap();
            if obj.material_id >= num_materials {
                println!(
                    "Instance {}: Object {}, Material ID {}, Emission {}",
                    index, instance.object_id, obj.material_id, 0.0
                );
                continue;
            }
            let mat = self
                .buffers
                .get_element::<GenericMaterial>(MATERIAL_BUFFER, obj.material_id as usize)
                .unwrap();
            let emission = mat.color_emissive.length();
            println!(
                "Instance {}: Object {}, Material ID {}, Emission {}",
                index, instance.object_id, obj.material_id, emission
            );
        }

        let num_emissive_instances = final_instance_indexes
            .iter()
            .filter(|&&index| {
                let instance = self
                    .buffers
                    .get_element::<Instance>(INSTANCE_BUFFER, index as usize)
                    .unwrap();
                let obj = self
                    .buffers
                    .get_element::<Object>(OBJ_BUFFER, instance.object_id as usize)
                    .unwrap();
                if obj.material_id >= num_materials {
                    return false;
                }
                let mat = self
                    .buffers
                    .get_element::<GenericMaterial>(MATERIAL_BUFFER, obj.material_id as usize)
                    .unwrap();
                mat.color_emissive.length() > 0.0
            })
            .count();

        println!("Number of emissive instances: {num_emissive_instances}");

        num_emissive_instances
    }
}
