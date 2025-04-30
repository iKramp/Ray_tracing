use shared::{glam::Vec3, BoundingBox, Bvh, ChildTriangleMode, Vertex};

pub fn create_bvh(vertices: &[Vertex], triangles: &mut [(u32, u32, u32)]) -> Vec<Bvh> {
    let mut bvh_nodes = Vec::new();
    let bounding_box = find_bounding_box(0, triangles.len() as u32 - 1, triangles, vertices);
    bvh_nodes.push(Bvh {
        bounding_box,
        child_1_or_first_tri: 0,
        child_2_or_last_tri: (triangles.len() - 1) as u32,
        mode: ChildTriangleMode::Triangles,
    });
    if triangles.len() > 5 {
        create_bvh_recursive(vertices, triangles, &mut bvh_nodes, 0);
    }

    //we allow max 16 depth per object due to gpu stack constraints
    assert!(bvh_nodes.len() <= 2_usize.pow(16));

    bvh_nodes
}

fn create_bvh_recursive(
    vertices: &[Vertex],
    triangles: &mut [(u32, u32, u32)],
    bvh_nodes: &mut Vec<Bvh>,
    parent_node_index: u32,
) {
    let parent_node = &mut bvh_nodes[parent_node_index as usize];
    let bounding_box = &parent_node.bounding_box;
    let size_x = bounding_box.max.x - bounding_box.min.x;
    let size_y = bounding_box.max.y - bounding_box.min.y;
    let size_z = bounding_box.max.z - bounding_box.min.z;
    let split_axis = if size_x > size_y && size_x > size_z {
        SplitAxis::X
    } else if size_y > size_x && size_y > size_z {
        SplitAxis::Y
    } else {
        SplitAxis::Z
    };

    //sort triangles
    sort_by_axis(&mut triangles[parent_node.child_1_or_first_tri as usize..=parent_node.child_2_or_last_tri as usize], vertices, split_axis as usize);

    let first_triangle = parent_node.child_1_or_first_tri;
    let mid = (parent_node.child_1_or_first_tri + parent_node.child_2_or_last_tri) / 2;
    let last_triangle = parent_node.child_2_or_last_tri;
    let first_box = find_bounding_box(
        parent_node.child_1_or_first_tri,
        mid,
        triangles,
        vertices,
    );
    let second_box = find_bounding_box(
        mid + 1,
        parent_node.child_2_or_last_tri,
        triangles,
        vertices,
    );

    let child_1 = bvh_nodes.len() as u32;
    let child_2 = child_1 + 1;
    bvh_nodes[parent_node_index as usize].child_1_or_first_tri = child_1;
    bvh_nodes[parent_node_index as usize].child_2_or_last_tri = child_2;
    bvh_nodes[parent_node_index as usize].mode = ChildTriangleMode::Children;
    bvh_nodes.push(Bvh {
        bounding_box: first_box,
        child_1_or_first_tri: first_triangle,
        child_2_or_last_tri: mid,
        mode: ChildTriangleMode::Triangles,
    });
    bvh_nodes.push(Bvh {
        bounding_box: second_box,
        child_1_or_first_tri: mid + 1,
        child_2_or_last_tri: last_triangle,
        mode: ChildTriangleMode::Triangles,
    });

    if last_triangle - first_triangle > 10 {
        create_bvh_recursive(
            vertices,
            triangles,
            bvh_nodes,
            child_1,
        );
        create_bvh_recursive(
            vertices,
            triangles,
            bvh_nodes,
            child_2,
        );
    }

}

enum SplitAxis {
    X = 0,
    Y = 1,
    Z = 2,
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

pub fn find_bounding_box(first_triangle: u32, last_triangle: u32, triangles: &[(u32, u32, u32)], vertices: &[Vertex]) -> BoundingBox {
    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

    for i in first_triangle..=last_triangle {
        let triangle = &triangles[i as usize];
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
