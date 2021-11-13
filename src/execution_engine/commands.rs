//! Commands are the last intermediate representation before conversion into vulkan command buffers.
//! The IR is designed to be a direct mapping to vulkan commands with only placeholders for
//! specializable resources and external synchronization for them left unresolved.

use std::collections::HashMap;
use std::sync::{Arc, MutexGuard};
use ash::vk;
use ash::vk::{Handle, Queue};
use crate::execution_engine::executable::ExecutableCommons;
use crate::execution_engine::placeholder_objects::*;
use crate::objects::id::{BufferId, BufferViewId, GenericId, ImageId, ImageViewId};
use crate::rosella::DeviceContext;

pub struct QueueRecorder<'a> {
    device: &'a DeviceContext,
    command_buffer: vk::CommandBuffer,
}

impl<'a> QueueRecorder<'a> {
    pub fn begin(device: &'a DeviceContext, command_buffer: vk::CommandBuffer) -> Result<Self, vk::Result> {
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty());

        unsafe{ device.vk().begin_command_buffer(command_buffer, &begin_info.build())? };
        Ok(Self{ device, command_buffer })
    }

    pub fn get_device(&self) -> &DeviceContext {
        self.device
    }

    pub fn get_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }

    pub fn end(self) -> Result<vk::CommandBuffer, vk::Result> {
        unsafe{ self.device.vk().end_command_buffer(self.command_buffer)? };
        Ok(self.command_buffer)
    }
}

pub struct HandleMap {
    map: HashMap<GenericId, u64>,
}

impl HandleMap {
    pub fn get_raw_map(&self) -> &HashMap<GenericId, u64> {
        &self.map
    }

    pub fn get_raw_map_mut(&mut self) -> &mut HashMap<GenericId, u64> {
        &mut self.map
    }

    pub fn get_buffer(&self, id: BufferId) -> Option<vk::Buffer> {
        self.map.get(&id.as_generic()).map(|v| vk::Buffer::from_raw(*v))
    }

    pub fn get_buffer_view(&self, id: BufferViewId) -> Option<vk::BufferView> {
        self.map.get(&id.as_generic()).map(|v| vk::BufferView::from_raw(*v))
    }

    pub fn get_image(&self, id: ImageId) -> Option<vk::Image> {
        self.map.get(&id.as_generic()).map(|v| vk::Image::from_raw(*v))
    }

    pub fn get_image_view(&self, id: ImageViewId) -> Option<vk::ImageView> {
        self.map.get(&id.as_generic()).map(|v| vk::ImageView::from_raw(*v))
    }
}

pub trait Command {
    fn record(&self, recorder: &mut QueueRecorder, handle_map: &HandleMap) -> Result<(), &'static str>;
}

pub struct CommandList {
    commands: Vec<Box<dyn Command>>,
    queue_family: u32,
    wait_mapping: Box<[usize]>,
    signal_mapping: Box<[usize]>,
}

impl CommandList {
    pub fn get_queue_family(&self) -> u32 {
        self.queue_family
    }

    pub fn get_wait_mapping(&self) -> &Box<[usize]> {
        &self.wait_mapping
    }

    pub fn get_signal_mapping(&self) -> &Box<[usize]> {
        &self.signal_mapping
    }

    pub fn record<'a>(&self, recorder: &mut QueueRecorder<'a>, handle_map: &HandleMap) -> Result<(), &'static str> {
        for command in &self.commands {
            command.record(recorder, handle_map)?;
        }
        Ok(())
    }
}

pub enum SemaphoreOpInfo {
    BinarySemaphore(),
    TimelineSemaphore(u64),
}

pub struct ResourceSpecializationInfo {
    specialized: HashMap<GenericId, u64>,
    pending_buffers: Box<[BufferId]>,
    pending_images: Box<[ImageId]>,
}

impl ResourceSpecializationInfo {
    pub fn specialize_resources(&self, specialization_set: &SpecializationSet) -> Result<HashMap<GenericId, u64>, &'static str> {
        let mut result = self.specialized.clone();
        for id in self.pending_buffers.iter() {
            let buffer = specialization_set.get_buffer(*id).ok_or("Missing buffer in specialization set")?;
            result.insert(id.as_generic(), buffer.as_raw());
        }
        for id in self.pending_images.iter() {
            let image = specialization_set.get_image(*id).ok_or("Missing image in specialization set")?;
            result.insert(id.as_generic(), image.as_raw());
        }

        Ok(result)
    }
}

pub struct UnspecializedExecutable {
    commons: Arc<ExecutableCommons>,
    commands: Vec<CommandList>,
    semaphore_wait_ops: Vec<SemaphoreOpInfo>,
    semaphore_signal_ops: Vec<SemaphoreOpInfo>,
    specialization_info: ResourceSpecializationInfo,
}

impl UnspecializedExecutable {
    pub fn specialize(&self, specialization_set: &SpecializationSet) -> Result<super::executable::Executable, &'static str> {
        Err("")
    }
}