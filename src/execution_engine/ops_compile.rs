use std::cmp::Ordering;
use crate::execution_engine::placeholder_objects::*;
use crate::execution_engine::ops::{ OpList };

#[derive(Copy, Clone, Eq, PartialEq, Ord)]
struct OpIndex {
    pub op_list: u32,
    pub op: u32,
}

impl PartialOrd for OpIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self.op_list.cmp(&other.op_list) {
            Ordering::Equal => self.op.cmp(&other.op),
            o => o,
        })
    }
}

#[derive(Copy, Clone)]
struct BufferMetadata {
    first_used: OpIndex,
    last_used: OpIndex,
}

#[derive(Copy, Clone)]
struct ImageMetadata {
    first_used: OpIndex,
    last_used: OpIndex,
}

pub struct OpsCompiler<'p, 'o> {
    object_set: &'p PlaceholderObjectSet,
    ops: &'o Vec<OpList>,
    buffer_metadata: Vec<Option<BufferMetadata>>,
    image_metadata: Vec<Option<ImageMetadata>>,
}

impl<'p, 'o> OpsCompiler<'p, 'o> {
    pub fn new(ops: &'o Vec<OpList>, object_set: &'p PlaceholderObjectSet) -> Self {
        let mut buffer_metadata = Vec::new();
        buffer_metadata.resize_with(object_set.get_buffer_count(), None);

        let mut image_metadata = Vec::new();
        image_metadata.resize_with(object_set.get_image_count(), None);

        Self{
            ops,
            object_set,
            buffer_metadata,
            image_metadata,
        }
    }

    fn get_buffer_metadata(&self, id: BufferId) -> Result<&Option<BufferMetadata>, &'static str> {
        if !self.object_set.owns_object(id) {
            return Err("BufferId is not owned by used placeholder object pool");
        }

        // Index out of range means the id itself is invalid which is a serious error so we panic
        Ok(self.buffer_metadata.get(id.get_local_id() as usize).unwrap())
    }

    fn get_buffer_metadata_mut(&mut self, id: BufferId) -> Result<&mut Option<BufferMetadata>, &'static str> {
        if !self.object_set.owns_object(id) {
            return Err("BufferId is not owned by used placeholder object pool");
        }

        // Index out of range means the id itself is invalid which is a serious error so we panic
        Ok(self.buffer_metadata.get_mut(id.get_local_id() as usize).unwrap())
    }

    fn get_image_metadata(&self, id: ImageId) -> Result<&Option<ImageMetadata>, &'static str> {
        if !self.object_set.owns_object(id) {
            return Err("ImageId is not owned by used placeholder object pool");
        }

        // Index out of range means the id itself is invalid which is a serious error so we panic
        Ok(self.image_metadata.get(id.get_local_id() as usize).unwrap())
    }

    fn get_image_metadata_mut(&mut self, id: ImageId) -> Result<&mut Option<ImageMetadata>, &'static str> {
        if !self.object_set.owns_object(id) {
            return Err("ImageId is not owned by used placeholder object pool");
        }

        // Index out of range means the id itself is invalid which is a serious error so we panic
        Ok(self.image_metadata.get_mut(id.get_local_id() as usize).unwrap())
    }
}