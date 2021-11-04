//! Placeholder objects are used in the ops IR to represent vulkan objects. A placeholder object
//! can either be a placeholder or fully defined.
//!
//! A placeholder can later be specialized into different objects at a commands level without needing
//! to recompile the entire program. Since memory allocation takes place during the ops compile stage
//! a placeholder object must be specialized by an external object.
//!
//! Fully defined objects on the other hand will be fixed after the ops compile stage. They can either
//! be dynamically allocated by the ops compiler or be set to some external object.

use std::fmt::{Debug, Formatter, Pointer};
use std::sync::atomic::{AtomicU64, Ordering};
use shaderc::ResourceKind::Buffer;
use crate::objects::BufferSpec;

#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectType(u8);

impl ObjectType {
    const fn new(v: u8) -> Self {
        if v > Self::MAX {
            // panic!("ObjectType value must be less than Self::MAX");
        }

        ObjectType(v)
    }

    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    pub const fn as_str(&self) -> &'static str {
        match *self {
            Self::OTHER => "UNKNOWN",
            Self::BUFFER => "BUFFER",
            Self::BUFFER_VIEW => "BUFFER_VIEW",
            Self::IMAGE => "IMAGE",
            Self::IMAGE_VIEW => "IMAGE_VIEW",
            _ => "INVALID" // Replace with panic once const fn panic is stabilized
        }
    }

    pub const BITS: u32 = 4u32;
    pub const MAX: u8 = (1u8 << Self::BITS) - 1u8;

    pub const OTHER: ObjectType = ObjectType::new(Self::MAX);
    pub const BUFFER: ObjectType = ObjectType::new(0u8);
    pub const BUFFER_VIEW: ObjectType = ObjectType::new(1u8);
    pub const IMAGE: ObjectType = ObjectType::new(2u8);
    pub const IMAGE_VIEW: ObjectType = ObjectType::new(3u8);
}

impl Debug for ObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub trait ObjectId : Into<u64> {
    fn get_local_id(&self) -> u64;

    fn get_type(&self) -> ObjectType;

    fn get_global_id(&self) -> u64;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ObjectIdCommon(u64);

impl ObjectIdCommon {
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

    const fn make_type(object_type: ObjectType) -> u64 {
        (object_type.as_u8() as u64) << Self::TYPE_OFFSET
    }

    const fn make_global(global_id: u64) -> u64 {
        if global_id > Self::GLOBAL_ID_MAX {
            // panic!("Global id is out of range: {}", global_id);
        }
        // We do range validation so no need to mask
        global_id << Self::GLOBAL_ID_OFFSET
    }

    const fn new(local_id: u64, global_id: u64, object_type: ObjectType) -> Self {
        ObjectIdCommon(Self::make_local(local_id) | Self::make_type(object_type) | Self::make_global(global_id))
    }

    const fn get_local_id(&self) -> u64 {
        (self.0 & Self::LOCAL_ID_MASK) >> Self::LOCAL_ID_OFFSET
    }

    const fn get_type(&self) -> ObjectType {
        ObjectType::new(((self.0 & Self::TYPE_MASK) >> Self::TYPE_OFFSET) as u8)
    }

    const fn get_global_id(&self) -> u64 {
        (self.0 & Self::GLOBAL_ID_MASK) >> Self::GLOBAL_ID_OFFSET
    }

    const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Into<u64> for ObjectIdCommon {
    fn into(self) -> u64 {
        self.0
    }
}

impl ObjectId for ObjectIdCommon {
    fn get_local_id(&self) -> u64 {
        self.get_local_id()
    }

    fn get_type(&self) -> ObjectType {
        self.get_type()
    }

    fn get_global_id(&self) -> u64 {
        self.get_global_id()
    }
}

impl Debug for ObjectIdCommon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectId")
            .field("type", &self.get_type())
            .field("local_id", &self.get_local_id())
            .field("global_id", &self.get_global_id())
            .finish()
    }
}

macro_rules! define_object_id {
    ($id_type: ident, $obj_type: expr) => {
        #[doc = concat!("A unique id referencing a ", stringify!($name))]
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $id_type(ObjectIdCommon);

        impl $id_type {
            const fn new(local_id: u64, global_id: u64) -> Self {
                Self(ObjectIdCommon::new(local_id, global_id, $obj_type))
            }

            pub const fn get_local_id(&self) -> u64 {
                self.0.get_local_id()
            }

            pub const fn get_type(&self) -> ObjectType {
                $obj_type
            }

            pub const fn get_global_id(&self) -> u64 {
                self.0.get_global_id()
            }

            pub const fn as_u64(&self) -> u64 {
                self.0.as_u64()
            }
        }

        impl Debug for $id_type {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    }
}

define_object_id!(BufferId, ObjectType::BUFFER);
define_object_id!(BufferViewId, ObjectType::BUFFER_VIEW);
define_object_id!(ImageId, ObjectType::IMAGE);
define_object_id!(ImageViewId, ObjectType::IMAGE_VIEW);

#[derive(Copy, Clone)]
pub struct ExternalBufferInfo {
    spec: crate::objects::BufferSpec,
    allowed_usage_flags: ash::vk::BufferUsageFlags,
}

#[derive(Copy, Clone)]
pub struct InternalBufferInfo {
    spec: crate::objects::BufferSpec,
    additional_usage_flags: ash::vk::BufferUsageFlags,
    required_memory_properties: ash::vk::MemoryPropertyFlags,
    preferred_memory_properties: ash::vk::MemoryPropertyFlags,
    // TODO memory_type_restrictions: ash::vk::MemoryTypeFlags,
}

#[derive(Copy, Clone)]
pub enum BufferInfo {
    Placeholder(),
    External(ExternalBufferInfo),
    Internal(InternalBufferInfo),
}

#[derive(Copy, Clone)]
pub enum BufferViewInfo {
    Placeholder(),
    External(),
    Internal(),
}

#[derive(Copy, Clone)]
pub enum ImageInfo {
    Placeholder(),
    External(),
    Internal(),
}

#[derive(Copy, Clone)]
pub enum ImageViewInfo {
    Placeholder(),
    External(),
    Internal(),
}

static NEXT_GLOBAL_ID : AtomicU64 = AtomicU64::new(1);

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
        PlaceholderObjectSet{
            global_id: make_global_id(),
            buffers: Vec::new(),
            buffer_views: Vec::new(),
            images: Vec::new(),
            image_views: Vec::new(),
        }
    }

    fn push_buffer(&mut self, buffer: BufferInfo) -> Result<BufferId, &'static str> {
        let index : u64 = self.buffers.len() as u64;
        if index > ObjectIdCommon::LOCAL_ID_MAX {
            return Err("Too many buffers in PlaceholderObjectSet");
        }
        self.buffers.push(buffer);
        Ok(BufferId::new(index, self.global_id))
    }

    fn push_buffer_view(&mut self, buffer_view: BufferViewInfo) -> Result<BufferViewId, &'static str> {
        let index : u64 = self.buffer_views.len() as u64;
        if index > ObjectIdCommon::LOCAL_ID_MAX {
            return Err("Too many buffer views in PlaceholderObjectSet");
        }
        self.buffer_views.push(buffer_view);
        Ok(BufferViewId::new(index, self.global_id))
    }

    pub fn define_placeholder_buffer(&mut self) -> Result<BufferId, &'static str> {
        self.push_buffer(BufferInfo::Placeholder())
    }

    pub fn define_external_buffer(&mut self, info: ExternalBufferInfo) -> Result<BufferId, &'static str> {
        todo!()
    }

    pub fn define_internal_buffer(&mut self, info: InternalBufferInfo) -> Result<BufferId, &'static str> {
        todo!()
    }

    pub fn get_buffer_info(&self, id: BufferId) -> Option<&BufferInfo> {
        if id.get_global_id() != self.global_id {
            panic!("BufferId belongs to different PlaceholderObjectSet");
        }
        self.buffers.get(id.get_local_id() as usize)
    }
}

mod test {
    use crate::execution_engine::placeholder_objects::ObjectIdCommon;

    #[test]
    fn test_object_id_common() {
        let id = ObjectIdCommon::new(25, 182, super::ObjectType::IMAGE);
        assert_eq!(id.get_local_id(), 25u64);
        assert_eq!(id.get_global_id(), 182u64);
        assert_eq!(id.get_type(), super::ObjectType::IMAGE);

        let id = ObjectIdCommon::new(ObjectIdCommon::LOCAL_ID_MAX, ObjectIdCommon::GLOBAL_ID_MAX, super::ObjectType::OTHER);
        assert_eq!(id.get_local_id(), ObjectIdCommon::LOCAL_ID_MAX);
        assert_eq!(id.get_global_id(), ObjectIdCommon::GLOBAL_ID_MAX);
        assert_eq!(id.get_type(), super::ObjectType::OTHER);

        let id = ObjectIdCommon::new((ObjectIdCommon::LOCAL_ID_MAX + 1u64) >> 1u32, (ObjectIdCommon::GLOBAL_ID_MAX + 1u64) >> 1u32, super::ObjectType::BUFFER);
        assert_eq!(id.get_local_id(), (ObjectIdCommon::LOCAL_ID_MAX + 1u64) >> 1u32);
        assert_eq!(id.get_global_id(), (ObjectIdCommon::GLOBAL_ID_MAX + 1u64) >> 1u32);
        assert_eq!(id.get_type(), super::ObjectType::BUFFER);
    }
}