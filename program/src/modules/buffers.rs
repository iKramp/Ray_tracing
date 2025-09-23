use std::collections::HashMap;

use crate::modules::buffers;

//COMMON_BUFFERS
pub const VERT_BUFFER: &str = "vertex_buffer";
pub const TRI_BUFFER: &str = "triangle_buffer";
pub const BVH_BUFFER: &str = "bvh_buffer";
pub const OBJ_BUFFER: &str = "object_buffer";
pub const INSTANCE_BUFFER: &str = "instance_buffer";
pub const DEBUG_POINTS_BUFFER: &str = "debug_points_buffer";
pub const RAY_STATE_BUFFER: &str = "ray_state_buffer";
pub const NORMAL_BUFFER: &str = "normal_buffer";
pub const UV_BUFFER: &str = "uv_buffer";

#[derive(Debug)]
pub struct Buffer {
    pub data: Vec<u8>,
    pub num_elements: usize,
    pub index: usize,
    pub alignment: usize,
}

impl Buffer {
    pub fn new<T>(data: Vec<T>, index: usize) -> Self {
        let num_elements = data.len();
        let data = unsafe {
            let byte_ptr = data.as_ptr() as *const u8;
            let byte_len = data.len() * std::mem::size_of::<T>();
            let byte_capacity = data.capacity() * std::mem::size_of::<T>();
            std::mem::forget(data);
            Vec::from_raw_parts(byte_ptr as *mut u8, byte_len, byte_capacity)
        };
        let alignment = std::mem::align_of::<T>();
        Buffer { data, num_elements, index, alignment }
    }

    pub fn append(&mut self, other: &[u8], other_len: usize) {
        self.data.extend_from_slice(other);
        self.num_elements += other_len;
        
    }
}

#[derive(Debug)]
pub struct BufferHolder {
    buffers: HashMap<String, Buffer>,
    changed: bool,
}

impl BufferHolder {
    pub fn new() -> Self {
        BufferHolder { buffers: HashMap::new(), changed: false }
    }

    pub fn print(&self) {
        for buffer in self.buffers.iter() {
            println!("Buffer {}: {} elements, {} bytes", buffer.0, buffer.1.num_elements, buffer.1.data.len());
            //print elements if less than 250
            if buffer.1.num_elements == 12 {
                let elem_size = buffer.1.data.len() / buffer.1.num_elements;
                for i in 0..buffer.1.num_elements {
                    let start = i * elem_size;
                    let end = start + elem_size;
                    let elem = &buffer.1.data[start..end];
                    print!("{:?}, ", elem);
                }
                println!();
            } else {
                println!("Too many elements to display");
            }
        }
    }

    pub fn insert<T>(&mut self, name: &str, data: Vec<T>) {
        let existing = self.buffers.get(name);
        let buffer = if let Some(existing) = existing {
            //replace
            Buffer::new(data, existing.index)
        } else {
            Buffer::new(data, self.buffers.len())
        };

        self.buffers.insert(name.to_string(), buffer);
        self.changed = true;
    }

    pub fn num_buffers(&self) -> usize {
        self.buffers.len()
    }

    pub fn get(&self, name: &str) -> Option<&Buffer> {
        self.buffers.get(name)
    }

    pub fn get_num_elements(&self, name: &str) -> Option<usize> {
        self.buffers.get(name).map(|buffer| buffer.num_elements)
    }

    pub fn get_length_unchecked(&self, name: &str) -> usize {
        self.get_num_elements(name).unwrap_or(0)
    }

    pub fn append<T: Clone>(&mut self, name: &str, data: &[T]) {
        if let Some(buffer) = self.buffers.get_mut(name) {
            let byte_data = unsafe {
                std::slice::from_raw_parts(
                    data.as_ptr() as *const u8,
                    std::mem::size_of_val(data)
                )
            };
            buffer.append(byte_data, data.len());
            self.changed = true;
        } else {
            self.insert(name, data.to_vec());
        }
    }

    pub fn get_all_buffers(&self) -> impl Iterator<Item = &Buffer> {
        let mut a = self.buffers.values().collect::<Vec<_>>();
        a.sort_by_key(|buffer| buffer.index);
        a.into_iter()
    }

    pub fn has_changed(&self) -> bool {
        self.changed
    }

    pub fn clear_changed(&mut self) {
        self.changed = false;
    }
}

impl Default for BufferHolder {
    fn default() -> Self {
        Self::new()
    }
}

