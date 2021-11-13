use std::cmp::Ordering;
use crate::execution_engine::placeholder_objects::*;
use crate::execution_engine::ops::{ObjectUsageRegistry, OpList};
use crate::objects::id::{BufferId, BufferViewId, GenericId, ImageId, ImageViewId, ObjectType};

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

impl BufferMetadata {
    fn new(initial_used: OpIndex) -> Result<Self, &'static str> {
        Ok(BufferMetadata{ first_used: initial_used, last_used: initial_used })
    }

    fn update_usage(&mut self, usage: OpIndex) {
        if self.first_used > usage {
            self.first_used = usage;
        }
        if self.last_used < usage {
            self.last_used = usage;
        }
    }
}

#[derive(Copy, Clone)]
struct ImageMetadata {
    first_used: OpIndex,
    last_used: OpIndex,
}

impl ImageMetadata {
    fn new(initial_used: OpIndex) -> Result<Self, &'static str> {
        Ok(ImageMetadata{ first_used: initial_used, last_used: initial_used })
    }

    fn update_usage(&mut self, usage: OpIndex) {
        if self.first_used > usage {
            self.first_used = usage;
        }
        if self.last_used < usage {
            self.last_used = usage;
        }
    }
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
        buffer_metadata.resize_with(object_set.get_buffer_count(), || None);

        let mut image_metadata = Vec::new();
        image_metadata.resize_with(object_set.get_image_count(), || None);

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

    fn build_object_usages(&mut self) -> Result<(), &'static str> {
        CompilerUsageRegistry::new(self).build()
    }
}

struct CompilerUsageRegistry<'c, 'p, 'o> {
    compiler: &'c mut OpsCompiler<'p, 'o>,
    current_index: OpIndex,
}

impl<'c, 'p, 'o> CompilerUsageRegistry<'c, 'p, 'o> {
    fn new(compiler: &'c mut OpsCompiler<'p, 'o>) -> Self {
        CompilerUsageRegistry {
            compiler,
            current_index: OpIndex{ op_list: 0, op: 0 },
        }
    }

    fn build(mut self) -> Result<(), &'static str> {
        for op_list in self.compiler.ops {
            for entry in op_list.get_entries() {
                entry.op.register_object_usage(&self)?;

                self.current_index.op += 1u32;
            }

            self.current_index.op = 0u32;
            self.current_index.op_list += 1u32;
        }

        Ok(())
    }

    fn on_buffer_used(&mut self, buffer: BufferId) -> Result<(), &'static str> {
        let metadata = self.compiler.get_buffer_metadata_mut(buffer)?;

        match metadata {
            None => { metadata.replace(BufferMetadata::new(self.current_index)?); },
            Some(meta) => meta.update_usage(self.current_index),
        }

        Ok(())
    }

    fn on_buffer_view_used(&mut self, buffer_view: BufferViewId) -> Result<(), &'static str> {
        let info = self.compiler.object_set.get_buffer_view_info(buffer_view).ok_or("Unable to find buffer view in used placeholder object set")?;
        self.on_buffer_used(info.get_buffer())
    }

    fn on_image_used(&mut self, image: ImageId) -> Result<(), &'static str> {
        let metadata = self.compiler.get_image_metadata_mut(image)?;

        match metadata {
            None => { metadata.replace(ImageMetadata::new(self.current_index)?); },
            Some(meta) => meta.update_usage(self.current_index),
        }

        Ok(())
    }

    fn on_image_view_used(&mut self, image_view: ImageViewId) -> Result<(), &'static str> {
        let info = self.compiler.object_set.get_image_view_info(image_view).ok_or("Unable to find image view in used placeholder object set")?;
        self.on_image_used(info.get_image())
    }
}

impl<'c, 'p, 'o> ObjectUsageRegistry for CompilerUsageRegistry<'c, 'p, 'o> {
    fn register_object_usage(&mut self, object: GenericId) -> Result<(), &'static str> {
        match object.get_type() {
            ObjectType::BUFFER => self.on_buffer_used(object.downcast().unwrap()),
            ObjectType::BUFFER_VIEW => self.on_buffer_view_used(object.downcast().unwrap()),
            ObjectType::IMAGE => self.on_image_view_used(object.downcast().unwrap()),
            ObjectType::IMAGE_VIEW => self.on_image_view_used(object.downcast().unwrap()),
            _ => Ok(()),
        }
    }
}