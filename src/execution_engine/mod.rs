use std::sync::{Arc, LockResult, Mutex};
use crate::rosella::DeviceContext;

use ash::vk;

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
}

impl ExecutionEngine {
    pub fn new(device: Arc<DeviceContext>, queue_family_count: u32) -> Result<Self, vk::Result> {
        let mut command_pools = Vec::new();
        command_pools.resize_with(queue_family_count as usize, || Mutex::new(vk::CommandPool::null()));

        for i in 0..queue_family_count {
            let create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(i);

            let pool = unsafe{ device.vk().create_command_pool(&create_info.build(), None) }?;
            *command_pools.get_mut(i as usize).unwrap().get_mut().unwrap() = pool;
        }

        Ok(Self{ device, command_pools: command_pools.into_boxed_slice() })
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