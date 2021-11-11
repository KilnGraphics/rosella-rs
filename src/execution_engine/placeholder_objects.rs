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

pub struct ObjectType;

// TODO Note this should be updated to a enum once adt_const_params is stabilized.
impl ObjectType {
    pub const fn as_str(ty: u8) -> &'static str {
        match ty {
            Self::BUFFER => "Buffer",
            Self::BUFFER_VIEW => "BufferView",
            Self::IMAGE => "Image",
            Self::IMAGE_VIEW => "ImageView",
            Self::SEMAPHORE => "Semaphore",
            Self::TIMELINE_SEMAPHORE => "TimelineSemaphore",
            Self::EVENT => "Event",
            _ => "Invalid",
        }
    }

    pub const BITS: u32 = 4u32;
    pub const MAX: u8 = (1u8 << Self::BITS) - 1u8;

    pub const GENERIC: u8 = Self::MAX;

    pub const BUFFER: u8 = 0u8;
    pub const BUFFER_VIEW: u8 = 1u8;
    pub const IMAGE: u8 = 2u8;
    pub const IMAGE_VIEW: u8 = 3u8;
    pub const SEMAPHORE: u8 = 4u8;
    pub const TIMELINE_SEMAPHORE: u8 = 5u8;
    pub const EVENT: u8 = 6u8;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId<const TYPE: u8>(u64);

impl<const TYPE: u8> ObjectId<TYPE> {
    const LOCAL_ID_BITS: u32 = 16u32;
    const LOCAL_ID_OFFSET: u32 = 0u32;
    const LOCAL_ID_MAX: u64 = (1u64 << Self::LOCAL_ID_BITS) - 1u64;
    const LOCAL_ID_MASK: u64 = Self::LOCAL_ID_MAX << Self::LOCAL_ID_OFFSET;

    const TYPE_BITS: u32 = ObjectType::BITS;
    const TYPE_OFFSET: u32 = Self::LOCAL_ID_OFFSET + Self::LOCAL_ID_BITS;
    const TYPE_MAX: u64 = ObjectType::MAX as u64;
    const TYPE_MASK: u64 = Self::TYPE_MAX << Self::TYPE_OFFSET;

    const GLOBAL_ID_BITS: u32 = u64::BITS - Self::LOCAL_ID_BITS - Self::TYPE_BITS;
    const GLOBAL_ID_OFFSET: u32 = Self::TYPE_OFFSET + Self::TYPE_BITS;
    const GLOBAL_ID_MAX: u64 = (1u64 << Self::GLOBAL_ID_BITS) - 1u64;
    const GLOBAL_ID_MASK: u64 = Self::GLOBAL_ID_MAX << Self::GLOBAL_ID_OFFSET;

    const fn make_local(local_id: u64) -> u64 {
        if local_id > Self::LOCAL_ID_MAX {
            // panic!("Local id is out of range: {}", local_id);
        }
        // We do range validation so no need to mask
        local_id << Self::LOCAL_ID_OFFSET
    }

    const fn make_type(object_type: u8) -> u64 {
        (object_type as u64) << Self::TYPE_OFFSET
    }

    const fn make_global(global_id: u64) -> u64 {
        if global_id > Self::GLOBAL_ID_MAX {
            // panic!("Global id is out of range: {}", global_id);
        }
        // We do range validation so no need to mask
        global_id << Self::GLOBAL_ID_OFFSET
    }

    const fn make(local_id: u64, global_id: u64, object_type: u8) -> Self {
        ObjectId(Self::make_local(local_id) | Self::make_type(object_type) | Self::make_global(global_id))
    }

    pub const fn get_local_id(&self) -> u64 {
        (self.0 & Self::LOCAL_ID_MASK) >> Self::LOCAL_ID_OFFSET
    }

    pub const fn get_type(&self) -> u8 {
        ((self.0 & Self::TYPE_MASK) >> Self::TYPE_OFFSET) as u8
    }

    pub const fn get_global_id(&self) -> u64 {
        (self.0 & Self::GLOBAL_ID_MASK) >> Self::GLOBAL_ID_OFFSET
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const fn as_generic(&self) -> ObjectId<{ ObjectType::GENERIC }> {
        ObjectId::<{ ObjectType::GENERIC }>(self.0)
    }
}

impl ObjectId<{ ObjectType::GENERIC }> {
    pub const fn downcast<const TRG: u8>(self) -> Option<ObjectId<TRG>> {
        if self.get_type() == TRG {
            Some(ObjectId::<TRG>(self.0))
        } else {
            None
        }
    }
}

impl<const TYPE: u8> Into<u64> for ObjectId<TYPE> {
    fn into(self) -> u64 {
        self.0
    }
}

impl<const TYPE: u8> Debug for ObjectId<TYPE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectId")
            .field("type", &self.get_type())
            .field("local_id", &self.get_local_id())
            .field("global_id", &self.get_global_id())
            .finish()
    }
}


impl<const TYPE: u8> Hash for ObjectId<TYPE> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl ObjectId<{ ObjectType::BUFFER }> {
    const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::BUFFER)
    }
}

impl ObjectId<{ ObjectType::BUFFER_VIEW }> {
    const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::BUFFER_VIEW)
    }
}

impl ObjectId<{ ObjectType::IMAGE }> {
    const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::IMAGE)
    }
}

impl ObjectId<{ ObjectType::IMAGE_VIEW }> {
    const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::IMAGE_VIEW)
    }
}

impl ObjectId<{ ObjectType::SEMAPHORE }> {
    const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::SEMAPHORE)
    }
}

impl ObjectId<{ ObjectType::TIMELINE_SEMAPHORE }> {
    const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::TIMELINE_SEMAPHORE)
    }
}

impl ObjectId<{ ObjectType::EVENT }> {
    const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::EVENT)
    }
}

pub type GenericId = ObjectId<{ ObjectType::GENERIC }>;
pub type BufferId = ObjectId<{ ObjectType::BUFFER }>;
pub type BufferViewId = ObjectId<{ ObjectType::BUFFER_VIEW }>;
pub type ImageId = ObjectId<{ ObjectType::IMAGE }>;
pub type ImageViewId = ObjectId<{ ObjectType::IMAGE_VIEW }>;
pub type SemaphoreId = ObjectId<{ ObjectType::SEMAPHORE }>;
pub type TimelineSemaphoreId = ObjectId<{ ObjectType::TIMELINE_SEMAPHORE }>;
pub type EventId = ObjectId<{ ObjectType::EVENT }>;

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

static NEXT_GLOBAL_ID: AtomicU64 = AtomicU64::new(1);

fn make_global_id() -> u64 {
    let id = NEXT_GLOBAL_ID.fetch_add(1, Ordering::Relaxed);
    id
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

mod test {
    use super::*;

    #[test]
    fn test_object_id_common() {
        let id = ImageId::new(25, 182).as_generic();
        assert_eq!(id.get_local_id(), 25u64);
        assert_eq!(id.get_global_id(), 182u64);
        assert_eq!(id.get_type(), ObjectType::IMAGE);
        assert!(id.downcast::<{ ObjectType::IMAGE }>().is_some());
        assert!(id.downcast::<{ObjectType::BUFFER }>().is_none());
        assert!(id.downcast::<{ObjectType::EVENT}>().is_none());
        assert!(id.downcast::<{ObjectType::SEMAPHORE}>().is_none());

        let id = EventId::new(GenericId::LOCAL_ID_MAX, GenericId::GLOBAL_ID_MAX).as_generic();
        assert_eq!(id.get_local_id(), GenericId::LOCAL_ID_MAX);
        assert_eq!(id.get_global_id(), GenericId::GLOBAL_ID_MAX);
        assert_eq!(id.get_type(), ObjectType::EVENT);
        assert!(id.downcast::<{ObjectType::EVENT}>().is_some());
        assert!(id.downcast::<{ObjectType::BUFFER}>().is_none());
        assert!(id.downcast::<{ObjectType::IMAGE_VIEW}>().is_none());
        assert!(id.downcast::<{ObjectType::SEMAPHORE}>().is_none());

        let id = BufferId::new((GenericId::LOCAL_ID_MAX + 1u64) >> 1u32, (GenericId::GLOBAL_ID_MAX + 1u64) >> 1u32).as_generic();
        assert_eq!(id.get_local_id(), (GenericId::LOCAL_ID_MAX + 1u64) >> 1u32);
        assert_eq!(id.get_global_id(), (GenericId::GLOBAL_ID_MAX + 1u64) >> 1u32);
        assert_eq!(id.get_type(), ObjectType::BUFFER);
        assert!(id.downcast::<{ObjectType::BUFFER}>().is_some());
        assert!(id.downcast::<{ObjectType::IMAGE}>().is_none());
        assert!(id.downcast::<{ObjectType::EVENT}>().is_none());
        assert!(id.downcast::<{ObjectType::TIMELINE_SEMAPHORE}>().is_none());
    }
}