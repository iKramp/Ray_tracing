use std::ops::Div;

use shared::{glam::Vec3, BoundingBox, Bvh, ChildTriangleMode, Vertex};

const MAX_DEPTH: u8 = 32;

pub fn create_bvh(vertices: &[Vertex], triangles: &mut [(u32, u32, u32)]) -> Vec<Bvh> {
    let mut bvh_nodes = Vec::new();
    let bounding_box = find_bounding_box(triangles, vertices);
    bvh_nodes.push(Bvh {
        bounding_box,
        child_1_or_first_tri: 0,
        child_2_or_last_tri: (triangles.len() - 1) as u32,
        mode: ChildTriangleMode::Triangles,
    });

    if triangles.len() > 5 {
        create_bvh_recursive(vertices, 0, triangles, &mut bvh_nodes, 0, 1);
    }

    println!(
        "BVH: {} triangles, {} nodes",
        triangles.len(),
        bvh_nodes.len()
    );

    bvh_nodes
}

fn create_bvh_recursive(
    vertices: &[Vertex],
    start_index: u32,
    triangles: &mut [(u32, u32, u32)],
    bvh_nodes: &mut Vec<Bvh>,
    parent_node_index: u32,
    depth: u8,
) {

    let (split_axis, split_index) = find_ideal_split(triangles, vertices, triangles.len() / 4);

    //sort triangles
    sort_by_axis(triangles, vertices, split_axis as usize);

    let first_box = find_bounding_box(
        &triangles[..split_index],
        vertices,
    );
    let second_box = find_bounding_box(
        &triangles[split_index..],
        vertices,
    );

    let child_1 = bvh_nodes.len() as u32;
    let child_2 = child_1 + 1;
    bvh_nodes[parent_node_index as usize].child_1_or_first_tri = child_1;
    bvh_nodes[parent_node_index as usize].child_2_or_last_tri = child_2;
    bvh_nodes[parent_node_index as usize].mode = ChildTriangleMode::Children;
    bvh_nodes.push(Bvh {
        bounding_box: first_box,
        child_1_or_first_tri: start_index,
        child_2_or_last_tri: start_index + split_index as u32 - 1,
        mode: ChildTriangleMode::Triangles,
    });
    bvh_nodes.push(Bvh {
        bounding_box: second_box,
        child_1_or_first_tri: start_index + split_index as u32,
        child_2_or_last_tri: start_index + triangles.len() as u32 - 1,
        mode: ChildTriangleMode::Triangles,
    });

    // if triangles.len() > 10 {
    //     create_bvh_recursive(vertices, start_index, &mut triangles[..split_index], bvh_nodes, child_1);
    //     create_bvh_recursive(vertices, start_index + split_index as u32, &mut triangles[split_index..], bvh_nodes, child_2);
    // }
    if split_index > 16 && depth < MAX_DEPTH {
        create_bvh_recursive(
            vertices,
            start_index,
            &mut triangles[..split_index],
            bvh_nodes,
            child_1,
            depth + 1,
        );
    }
    if triangles.len() - split_index > 16 && depth < MAX_DEPTH {
        create_bvh_recursive(
            vertices,
            start_index + split_index as u32,
            &mut triangles[split_index..],
            bvh_nodes,
            child_2,
            depth + 1,
        );
    }
}

fn find_ideal_split(triangles: &mut [(u32, u32, u32)], vertices: &[Vertex], splits: usize) -> (u32, usize) {
    let mut best_result = f32::MAX;
    let mut best_axis = 0;
    let mut best_split = 0;

    for axis in 0..3 {
        sort_by_axis(triangles, vertices, axis);
        let chunk_size = triangles.len() as f32 / (splits + 1) as f32;
        for i in 0..splits {
            let split_index = ((i as f32 + 1.0) * chunk_size).round() as usize;
            let first_box = find_bounding_box(&triangles[..split_index], vertices);
            let second_box = find_bounding_box(&triangles[split_index..], vertices);
            let split_cost = box_srface_area(&first_box) * split_index as f32 + box_srface_area(&second_box) * (triangles.len() as f32 - split_index as f32);

            if split_cost < best_result {
                best_result = split_cost;
                best_axis = axis;
                best_split = split_index;
            }
        }
    }

    (best_axis as u32, best_split)
}

fn box_srface_area(bounding_box: &BoundingBox) -> f32 {
    let size_x = (bounding_box.max.x - bounding_box.min.x).abs();
    let size_y = (bounding_box.max.y - bounding_box.min.y).abs();
    let size_z = (bounding_box.max.z - bounding_box.min.z).abs();
    2.0 * (size_x * size_y + size_x * size_z + size_y * size_z)
}

fn sort_by_axis(triangles: &mut [(u32, u32, u32)], vertices: &[Vertex], axis: usize) {
    triangles.sort_unstable_by(|a, b| {
        let a_center = (vertices[a.0 as usize].pos[axis]
            + vertices[a.1 as usize].pos[axis]
            + vertices[a.2 as usize].pos[axis])
            / 3.0;
        let b_center = (vertices[b.0 as usize].pos[axis]
            + vertices[b.1 as usize].pos[axis]
            + vertices[b.2 as usize].pos[axis])
            / 3.0;
        a_center.partial_cmp(&b_center).unwrap()
    });
}

pub fn find_bounding_box(triangles: &[(u32, u32, u32)], vertices: &[Vertex]) -> BoundingBox {
    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

    for triangle in triangles {
        let v1 = vertices[triangle.0 as usize].pos;
        let v2 = vertices[triangle.1 as usize].pos;
        let v3 = vertices[triangle.2 as usize].pos;
        let triangle_min = v1.min(v2).min(v3);
        let triangle_max = v1.max(v2).max(v3);
        min = min.min(triangle_min);
        max = max.max(triangle_max);
    }

    BoundingBox {
        min,
        padding_1: [0; 4],
        max,
        padding_2: [0; 4],
    }
}
