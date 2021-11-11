//! Commands are the last intermediate representation before conversion into vulkan command buffers.
//! The IR is designed to be a direct mapping to vulkan commands with only placeholders for
//! specializable resources and external synchronization for them left unresolved.

use std::sync::{Arc, MutexGuard};
use ash::vk;
use ash::vk::Queue;
use crate::execution_engine::placeholder_objects::*;

pub struct QueueRecorder<'a> {
    device: &'a Arc<ash::Device>,
    command_buffer: vk::CommandBuffer,
}

impl<'a> QueueRecorder<'a> {
    pub fn begin(device: &'a Arc<ash::Device>, command_buffer: vk::CommandBuffer) -> Result<Self, vk::Result> {
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty());

        unsafe{ device.begin_command_buffer(command_buffer, &begin_info.build())? };
        Ok(Self{ device, command_buffer })
    }

    pub fn get_device(&self) -> &Arc<ash::Device> {
        &self.device
    }

    pub fn get_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }

    pub fn end(self) -> Result<vk::CommandBuffer, vk::Result> {
        unsafe{ self.device.end_command_buffer(self.command_buffer)? };
        Ok(self.command_buffer)
    }
}

pub trait Command {
    fn record(&self, recorder: &mut QueueRecorder, specialization_set: &SpecializationSet) -> Result<(), &'static str>;
}

pub struct CommandList {
    commands: Vec<Box<dyn Command>>,
}

impl CommandList {
    pub fn record<'a>(&self, recorder: &mut QueueRecorder<'a>, specialization_set: &SpecializationSet) -> Result<(), &'static str> {
        for command in &self.commands {
            command.record(recorder, specialization_set)?;
        }
        Ok(())
    }
}