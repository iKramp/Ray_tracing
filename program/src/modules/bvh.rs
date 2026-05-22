use shared::{BoundingBox, Bvh, ChildTriangleMode, Face, Vertex};

const N_BUCKETS: usize = 64;
const MAX_TRIANGLES_PER_LEAF: usize = 64;
const MIN_TRIANGLES_PER_LEAF: usize = 1;

pub fn create_bvh(vertices: &[Vertex], triangles: &mut [Face]) -> Vec<Bvh> {
    let mut bvh_nodes = Vec::new();
    let bounding_box = find_bounding_box(triangles, vertices);
    bvh_nodes.push(Bvh {
        bounding_box,
        child_1_or_first_tri: 0,
        child_2_or_last_tri: (triangles.len() - 1) as u32,
        mode: ChildTriangleMode::Triangles,
    });

    if triangles.len() > 5 {
        create_bvh_iterative(vertices, 0, triangles, &mut bvh_nodes, 0);
    }

    println!(
        "BVH: {} triangles, {} nodes",
        triangles.len(),
        bvh_nodes.len()
    );

    bvh_nodes
}

fn create_bvh_iterative(
    vertices: &[Vertex],
    start_index: u32,
    triangles: &mut [Face],
    bvh_nodes: &mut Vec<Bvh>,
    root_node_index: u32,
) {
    struct StackEntry {
        parent_node_index: u32,
        start_index: u32,
        tri_start: usize, // start offset into triangles slice
        tri_len: usize,   // length of triangles slice
    }

    let mut max_stack_size = 0;

    let mut stack = vec![StackEntry {
        parent_node_index: root_node_index,
        start_index,
        tri_start: 0,
        tri_len: triangles.len(),
    }];

    while let Some(StackEntry {
        parent_node_index,
        start_index,
        tri_start,
        tri_len,
    }) = stack.pop()
    {
        max_stack_size = max_stack_size.max(stack.len());

        if tri_len <= MIN_TRIANGLES_PER_LEAF {
            continue;
        }

        let parent_bounding_box = bvh_nodes[parent_node_index as usize].bounding_box;
        let tris = &mut triangles[tri_start..tri_start + tri_len];
        let Some((split_index, (first_bound, second_bound))) =
            find_ideal_split(tris, vertices, parent_bounding_box)
        else {
            continue;
        };

        let child_1 = bvh_nodes.len() as u32;
        let child_2 = child_1 + 1;
        bvh_nodes[parent_node_index as usize].child_1_or_first_tri = child_1;
        bvh_nodes[parent_node_index as usize].child_2_or_last_tri = child_2;
        bvh_nodes[parent_node_index as usize].mode = ChildTriangleMode::Children;
        bvh_nodes.push(Bvh {
            bounding_box: first_bound,
            child_1_or_first_tri: start_index,
            child_2_or_last_tri: start_index + split_index as u32 - 1,
            mode: ChildTriangleMode::Triangles,
        });
        bvh_nodes.push(Bvh {
            bounding_box: second_bound,
            child_1_or_first_tri: start_index + split_index as u32,
            child_2_or_last_tri: start_index + tri_len as u32 - 1,
            mode: ChildTriangleMode::Triangles,
        });

        // Push second child first so first child is processed first (LIFO)
        stack.push(StackEntry {
            parent_node_index: child_2,
            start_index: start_index + split_index as u32,
            tri_start: tri_start + split_index,
            tri_len: tri_len - split_index,
        });
        stack.push(StackEntry {
            parent_node_index: child_1,
            start_index,
            tri_start,
            tri_len: split_index,
        });
    }

    println!("Max BVH stack size: {max_stack_size}");
}

//returns:
// - the integer of the first triangle that is in the second half of the split
// - the bounding boxes of the two halves of the split
fn find_ideal_split(
    triangles: &mut [Face],
    vertices: &[Vertex],
    parent_bounding_box: BoundingBox,
) -> Option<(usize, (BoundingBox, BoundingBox))> {
    //[[(bounding box, first triangle index); N_BUCKETS]; 3]
    let mut buckets = [[(BoundingBox::default_invalid(), 0_usize); N_BUCKETS]; 3];

    let parent_min = parent_bounding_box.min();
    let parent_max = parent_bounding_box.max();

    let x_bucket_size = (parent_max.x - parent_min.x) / N_BUCKETS as f32;
    let y_bucket_size = (parent_max.y - parent_min.y) / N_BUCKETS as f32;
    let z_bucket_size = (parent_max.z - parent_min.z) / N_BUCKETS as f32;
    let bucket_sizes = [x_bucket_size, y_bucket_size, z_bucket_size];

    for triangle in &mut *triangles {
        let v1 = vertices[triangle.vert.x as usize].pos;
        let v2 = vertices[triangle.vert.y as usize].pos;
        let v3 = vertices[triangle.vert.z as usize].pos;
        let center = (v1 + v2 + v3) / 3.0;
        let min = v1.min(v2).min(v3);
        let max = v1.max(v2).max(v3);
        let bounding_box = BoundingBox::new(min, max);
        for axis in 0..3 {
            let axis_min = parent_min[axis];
            let bucket_size = bucket_sizes[axis];
            let bucket_index = ((center[axis] - axis_min) / bucket_size).floor() as usize;
            let clamped_index = bucket_index.clamp(0, N_BUCKETS - 1);
            buckets[axis][clamped_index].0.union(&bounding_box);
            buckets[axis][clamped_index].1 += 1;
        }
    }

    //[[(bounding box of first half, bounding box of second half); N_BUCKETS - 1]; 3]
    let mut split_boxes = [[(
        (BoundingBox::default_invalid(), 0_usize),
        (BoundingBox::default_invalid(), 0_usize),
    ); N_BUCKETS - 1]; 3];

    for axis in 0..3 {
        let mut current_box = BoundingBox::default_invalid();
        let mut current_count = 0;
        for i in 0..N_BUCKETS - 1 {
            let bucket_box = &buckets[axis][i];
            if !bucket_box.0.is_valid() {
                //no triangles in this bucket, skip it
                continue;
            }
            current_box.union(&bucket_box.0);
            split_boxes[axis][i].0 .0 = current_box;
            current_count += bucket_box.1;
            split_boxes[axis][i].0 .1 = current_count;
        }

        current_box = BoundingBox::default_invalid();
        current_count = 0;
        for i in (1..N_BUCKETS).rev() {
            let bucket_box = &buckets[axis][i];
            if !bucket_box.0.is_valid() {
                //no triangles in this bucket, skip it
                continue;
            }
            current_box.union(&bucket_box.0);
            split_boxes[axis][i - 1].1 .0 = current_box;
            current_count += bucket_box.1;
            split_boxes[axis][i - 1].1 .1 = current_count;
        }
    }

    let mut best_result = f32::MAX;
    let mut best_axis = 0;
    let mut best_split = 0;
    let mut best_first_half_count = 0;

    for axis in 0..3 {
        for i in 0..N_BUCKETS - 1 {
            let box_1 = split_boxes[axis][i].0;
            let box_2 = split_boxes[axis][i].1;
            if !box_1.0.is_valid() || !box_2.0.is_valid() {
                //one of the halves is empty, skip this split
                continue;
            }

            let surface_area_1 = split_boxes[axis][i].0 .0.surface_area();
            let surface_area_2 = split_boxes[axis][i].1 .0.surface_area();
            let cnt_1 = split_boxes[axis][i].0 .1 as f32;
            let cnt_2 = split_boxes[axis][i].1 .1 as f32;

            let cost = (surface_area_1 * cnt_1 + surface_area_2 * cnt_2)
                / parent_bounding_box.surface_area()
                + 0.125;
            if cost < best_result {
                best_result = cost;
                best_axis = axis;
                best_split = i;
                best_first_half_count = cnt_1 as usize;
            }
        }
    }

    let parent_cost = triangles.len() as f32;

    if best_result > parent_cost && triangles.len() <= MAX_TRIANGLES_PER_LEAF {
        // No good split found, return None to create a leaf node
        return None;
    }

    sort_by_axis(triangles, vertices, best_axis);

    Some((
        best_first_half_count,
        (
            split_boxes[best_axis][best_split].0 .0,
            split_boxes[best_axis][best_split].1 .0,
        ),
    ))
}

fn sort_by_axis(triangles: &mut [Face], vertices: &[Vertex], axis: usize) {
    triangles.sort_unstable_by(|a, b| {
        let a_center = vertices[a.vert.x as usize].pos[axis]
            + vertices[a.vert.y as usize].pos[axis]
            + vertices[a.vert.z as usize].pos[axis];
        let b_center = vertices[b.vert.x as usize].pos[axis]
            + vertices[b.vert.y as usize].pos[axis]
            + vertices[b.vert.z as usize].pos[axis];
        a_center.partial_cmp(&b_center).unwrap()
    });
}

pub fn find_bounding_box(triangles: &[Face], vertices: &[Vertex]) -> BoundingBox {
    let mut bounding_box = BoundingBox::default_invalid();
    for triangle in triangles {
        let v1 = vertices[triangle.vert.x as usize].pos;
        let v2 = vertices[triangle.vert.y as usize].pos;
        let v3 = vertices[triangle.vert.z as usize].pos;
        let min = v1.min(v2).min(v3);
        let max = v1.max(v2).max(v3);
        bounding_box.union(&BoundingBox::new(min, max));
    }
    bounding_box
}
