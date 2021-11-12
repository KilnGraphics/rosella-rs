use std::sync::{Arc, LockResult, Mutex};
use crate::rosella::DeviceContext;

use ash::vk;
use crate::init::device::VulkanQueue;

pub mod commands;
pub mod ops;
pub mod ops_compile;
pub mod placeholder_objects;
pub mod memory;
pub mod executable;

mod object_manager;
mod resource_state;
mod static_resource_state;

pub struct ExecutionEngine {
    device: Arc<DeviceContext>,
    command_pools: Box<[Mutex<vk::CommandPool>]>,
    queues: Box<[Arc<VulkanQueue>]>,
}

impl ExecutionEngine {
    pub fn new(device: Arc<DeviceContext>, queues: Box<[Arc<VulkanQueue>]>) -> Result<Self, vk::Result> {
        let mut command_pools = Vec::new();
        command_pools.resize_with(queues.len(), || Mutex::new(vk::CommandPool::null()));

        for (i, queue) in queues.iter().enumerate() {
            if i != queue.get_queue_family_index() {
                panic!("Yes this is not very good TODO fix this") // TODO fix this
            }

            let create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(i as u32);

            let pool = unsafe{ device.vk().create_command_pool(&create_info.build(), None) }?;
            *command_pools.get_mut(i).unwrap().get_mut().unwrap() = pool;
        }

        Ok(Self{ device, queues, command_pools: command_pools.into_boxed_slice() })
    }

    fn get_queues(&self) -> &[Arc<VulkanQueue>] {
        self.queues.as_ref()
    }

    fn get_command_pools(&self) -> &[Mutex<vk::CommandPool>] {
        self.command_pools.as_ref()
    }
}

impl Drop for ExecutionEngine {
    fn drop(&mut self) {
        for pool in self.command_pools.iter_mut() {
            let pool = match pool.get_mut() {
                Ok(p) => p,
                Err(err) => err.into_inner()
            };

            unsafe{ self.device.vk().destroy_command_pool(*pool, None) };
        }
    }
}