//! Placeholder objects are used in the ops IR to represent vulkan objects. A placeholder object
//! can either be a placeholder or fully defined.
//!
//! A placeholder can later be specialized into different objects at a commands level without needing
//! to recompile the entire program. Since memory allocation takes place during the ops compile stage
//! a placeholder object must be specialized by an external object.
//!
//! Fully defined objects on the other hand will be fixed after the ops compile stage. They can either
//! be dynamically allocated by the ops compiler or be set to some external object.

use std::collections::HashMap;
use ash::vk;

use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::objects::*;
use crate::objects::id::{BufferId, BufferViewId, GenericId, ImageId, ImageViewId, make_global_id, ObjectId};

#[derive(Copy, Clone)]
pub struct ExternalBufferInfo {
    pub spec: BufferSpec,
    pub allowed_usage_flags: vk::BufferUsageFlags,
}

#[derive(Copy, Clone)]
pub struct InternalBufferInfo {
    pub spec: BufferSpec,
    pub additional_usage_flags: vk::BufferUsageFlags,
    pub required_memory_properties: vk::MemoryPropertyFlags,
    pub preferred_memory_properties: vk::MemoryPropertyFlags,
    pub memory_type_restrictions: u32,
}

impl InternalBufferInfo {
    pub const fn make_unconstrained(spec: BufferSpec) -> Self {
        InternalBufferInfo {
            spec,
            additional_usage_flags: vk::BufferUsageFlags::empty(),
            required_memory_properties: vk::MemoryPropertyFlags::all(),
            preferred_memory_properties: vk::MemoryPropertyFlags::empty(),
            memory_type_restrictions: !0u32,
        }
    }
}

#[derive(Copy, Clone)]
pub enum BufferInfo {
    Placeholder(),
    External(ExternalBufferInfo),
    Internal(InternalBufferInfo),
}

#[derive(Copy, Clone)]
pub struct ExternalBufferViewInfo {
    pub buffer: BufferId,
    pub range: BufferRange,
}

#[derive(Copy, Clone)]
pub struct InternalBufferViewInfo {
    pub buffer: BufferId,
    pub range: BufferRange,
    pub format: Format,
}

#[derive(Copy, Clone)]
pub enum BufferViewInfo {
    External(ExternalBufferViewInfo),
    Internal(InternalBufferViewInfo),
}

impl BufferViewInfo {
    pub const fn get_buffer(&self) -> BufferId {
        match self {
            BufferViewInfo::External(info) => info.buffer,
            BufferViewInfo::Internal(info) => info.buffer,
        }
    }

    pub const fn get_buffer_range(&self) -> BufferRange {
        match self {
            BufferViewInfo::External(info) => info.range,
            BufferViewInfo::Internal(info) => info.range,
        }
    }
}

#[derive(Copy, Clone)]
pub struct ExternalImageInfo {
    pub spec: ImageSpec,
    pub allowed_usage_flags: vk::ImageUsageFlags,
}

#[derive(Copy, Clone)]
pub struct InternalImageInfo {
    pub spec: ImageSpec,
    pub additional_usage_flags: vk::ImageUsageFlags,
    pub required_memory_properties: vk::MemoryPropertyFlags,
    pub preferred_memory_properties: vk::MemoryPropertyFlags,
    pub memory_type_restrictions: u32,
}

impl InternalImageInfo {
    pub const fn make_unconstrained(spec: ImageSpec) -> Self {
        InternalImageInfo {
            spec,
            additional_usage_flags: vk::ImageUsageFlags::empty(),
            required_memory_properties: vk::MemoryPropertyFlags::all(),
            preferred_memory_properties: vk::MemoryPropertyFlags::empty(),
            memory_type_restrictions: !0u32,
        }
    }
}

#[derive(Copy, Clone)]
pub enum ImageInfo {
    Placeholder(),
    External(ExternalImageInfo),
    Internal(InternalImageInfo),
}

#[derive(Copy, Clone)]
pub struct ExternalImageViewInfo {
    pub image: ImageId,
    pub range: ImageSubresourceRange,
}

#[derive(Copy, Clone)]
pub struct InternalImageViewInfo {
    pub image: ImageId,
    pub range: ImageSubresourceRange,
    pub format: Format,
    pub component_mapping: vk::ComponentMapping,
}

#[derive(Copy, Clone)]
pub enum ImageViewInfo {
    External(ExternalImageViewInfo),
    Internal(InternalImageViewInfo),
}

impl ImageViewInfo {
    pub const fn get_image(&self) -> ImageId {
        match self {
            ImageViewInfo::External(info) => info.image,
            ImageViewInfo::Internal(info) => info.image,
        }
    }

    pub const fn get_image_subresource_range(&self) -> ImageSubresourceRange {
        match self {
            ImageViewInfo::External(info) => info.range,
            ImageViewInfo::Internal(info) => info.range,
        }
    }
}

pub struct PlaceholderObjectSet {
    global_id: u64,
    buffers: Vec<BufferInfo>,
    buffer_views: Vec<BufferViewInfo>,
    images: Vec<ImageInfo>,
    image_views: Vec<ImageViewInfo>,
}

impl PlaceholderObjectSet {
    pub fn new() -> Self {
        PlaceholderObjectSet {
            global_id: make_global_id(),
            buffers: Vec::new(),
            buffer_views: Vec::new(),
            images: Vec::new(),
            image_views: Vec::new(),
        }
    }

    fn push_buffer(&mut self, buffer: BufferInfo) -> Result<BufferId, &'static str> {
        let index: u64 = self.buffers.len() as u64;
        if index > GenericId::LOCAL_ID_MAX {
            return Err("Too many buffers in PlaceholderObjectSet");
        }
        self.buffers.push(buffer);
        Ok(BufferId::new(index, self.global_id))
    }

    fn push_buffer_view(&mut self, buffer_view: BufferViewInfo) -> Result<BufferViewId, &'static str> {
        let index: u64 = self.buffer_views.len() as u64;
        if index > GenericId::LOCAL_ID_MAX {
            return Err("Too many buffer views in PlaceholderObjectSet");
        }
        self.buffer_views.push(buffer_view);
        Ok(BufferViewId::new(index, self.global_id))
    }

    fn push_image(&mut self, image: ImageInfo) -> Result<ImageId, &'static str> {
        let index: u64 = self.images.len() as u64;
        if index > GenericId::LOCAL_ID_MAX {
            return Err("Too many images in PlaceholderObjectSet");
        }
        self.images.push(image);
        Ok(ImageId::new(index, self.global_id))
    }

    fn push_image_view(&mut self, image_view: ImageViewInfo) -> Result<ImageViewId, &'static str> {
        let index: u64 = self.buffer_views.len() as u64;
        if index > GenericId::LOCAL_ID_MAX {
            return Err("Too many image views in PlaceholderObjectSet");
        }
        self.image_views.push(image_view);
        Ok(ImageViewId::new(index, self.global_id))
    }

    pub fn define_placeholder_buffer(&mut self) -> Result<BufferId, &'static str> {
        self.push_buffer(BufferInfo::Placeholder())
    }

    pub fn define_external_buffer(&mut self, info: ExternalBufferInfo) -> Result<BufferId, &'static str> {
        self.push_buffer(BufferInfo::External(info))
    }

    pub fn define_internal_buffer(&mut self, info: InternalBufferInfo) -> Result<BufferId, &'static str> {
        self.push_buffer(BufferInfo::Internal(info))
    }

    pub fn define_external_buffer_view(&mut self, info: ExternalBufferViewInfo) -> Result<BufferViewId, &'static str> {
        if info.buffer.get_global_id() != self.global_id {
            return Err("Parent buffer is not part of this PlaceholderObjectSet");
        }
        self.push_buffer_view(BufferViewInfo::External(info))
    }

    pub fn define_internal_buffer_view(&mut self, info: InternalBufferViewInfo) -> Result<BufferViewId, &'static str> {
        if info.buffer.get_global_id() != self.global_id {
            return Err("Parent buffer is not part of this PlaceholderObjectSet");
        }
        self.push_buffer_view(BufferViewInfo::Internal(info))
    }

    pub fn define_placeholder_image(&mut self) -> Result<ImageId, &'static str> {
        self.push_image(ImageInfo::Placeholder())
    }

    pub fn define_external_image(&mut self, info: ExternalImageInfo) -> Result<ImageId, &'static str> {
        self.push_image(ImageInfo::External(info))
    }

    pub fn define_internal_image(&mut self, info: InternalImageInfo) -> Result<ImageId, &'static str> {
        self.push_image(ImageInfo::Internal(info))
    }

    pub fn define_external_image_view(&mut self, info: ExternalImageViewInfo) -> Result<ImageViewId, &'static str> {
        if info.image.get_global_id() != self.global_id {
            return Err("Parent image is not part of this PlaceholderObjectSet");
        }
        self.push_image_view(ImageViewInfo::External(info))
    }

    pub fn define_internal_image_view(&mut self, info: InternalImageViewInfo) -> Result<ImageViewId, &'static str> {
        if info.image.get_global_id() != self.global_id {
            return Err("Parent image is not part of this PlaceholderObjectSet");
        }
        self.push_image_view(ImageViewInfo::Internal(info))
    }

    pub fn get_buffer_info(&self, id: BufferId) -> Option<&BufferInfo> {
        if id.get_global_id() != self.global_id {
            panic!("BufferId belongs to different PlaceholderObjectSet");
        }
        self.buffers.get(id.get_local_id() as usize)
    }

    pub fn get_buffer_view_info(&self, id: BufferViewId) -> Option<&BufferViewInfo> {
        if id.get_global_id() != self.global_id {
            panic!("BufferViewId belongs to different PlaceholderObjectSet");
        }
        self.buffer_views.get(id.get_local_id() as usize)
    }

    pub fn get_image_info(&self, id: ImageId) -> Option<&ImageInfo> {
        if id.get_global_id() != self.global_id {
            panic!("ImageId belongs to different PlaceholderObjectSet");
        }
        self.images.get(id.get_local_id() as usize)
    }

    pub fn get_image_view_info(&self, id: ImageViewId) -> Option<&ImageViewInfo> {
        if id.get_global_id() != self.global_id {
            panic!("ImageViewId belongs to different PlaceholderObjectSet");
        }
        self.image_views.get(id.get_local_id() as usize)
    }

    pub fn owns_object<const TYPE: u8>(&self, id: ObjectId<TYPE>) -> bool {
        id.get_global_id() == self.global_id
    }

    pub fn get_buffer_count(&self) -> usize {
        self.buffers.len()
    }

    pub fn get_image_count(&self) -> usize {
        self.images.len()
    }
}

pub struct SpecializationSet {
    buffers: HashMap<BufferId, vk::Buffer>,
    images: HashMap<ImageId, vk::Image>,
}

impl SpecializationSet {
    pub fn set_buffer(&mut self, id: BufferId, buffer: vk::Buffer) {
        self.buffers.insert(id, buffer);
    }

    pub fn set_image(&mut self, id: ImageId, image: vk::Image) {
        self.images.insert(id, image);
    }

    pub fn get_buffer(&self, id: BufferId) -> Option<vk::Buffer> {
        self.buffers.get(&id).map(|v| *v)
    }

    pub fn get_image(&self, id: ImageId) -> Option<vk::Image> {
        self.images.get(&id).map(|v| *v)
    }
}