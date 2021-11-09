use std::error::Error;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use ash::vk;
use ash::vk::TimelineSemaphoreSubmitInfoBuilder;
use crate::execution_engine::*;

pub enum WaitOperation {
    BinarySemaphore(vk::Semaphore),
    TimelineSemaphore(vk::Semaphore, u64),
}

pub enum SignalOperation {
    BinarySemaphore(vk::Semaphore),
    TimelineSemaphore(vk::Semaphore, u64),
}

pub struct Submission {
    queue: Arc<Mutex<vk::Queue>>,
    command_buffers: Box<[vk::CommandBufferSubmitInfoKHR]>,
    wait_mapping: Box<[usize]>,
    wait_semaphores: Box<[vk::SemaphoreSubmitInfoKHR]>,
    signal_mapping: Box<[usize]>,
    signal_semaphores: Box<[vk::SemaphoreSubmitInfoKHR]>,
}

impl Submission {
    fn update_semaphores(&mut self, wait_ops: &Vec<WaitOperation>, signal_ops: &Vec<SignalOperation>) {
        for (i, mapping) in self.wait_mapping.iter().enumerate() {
            let info = self.wait_semaphores.get_mut(i).unwrap();
            match wait_ops.get(*mapping).unwrap() {
                WaitOperation::BinarySemaphore(sem) => {
                    info.semaphore = *sem;
                }
                WaitOperation::TimelineSemaphore(sem, time) => {
                    info.semaphore = *sem;
                    info.value = *time;
                }
            }
        }

        for (i, mapping) in self.signal_mapping.iter().enumerate() {
            let info = self.signal_semaphores.get_mut(i).unwrap();
            match signal_ops.get(*mapping).unwrap() {
                SignalOperation::BinarySemaphore(sem) => {
                    info.semaphore = *sem;
                }
                SignalOperation::TimelineSemaphore(sem, time) => {
                    info.semaphore = *sem;
                    info.value = *time;
                }
            }
        }
    }

    pub fn submit(&mut self, wait_ops: &Vec<WaitOperation>, signal_ops: &Vec<SignalOperation>, submit_fn: vk::PFN_vkQueueSubmit2KHR) -> Result<(), vk::Result> {
        self.update_semaphores(wait_ops, signal_ops);

        let submit_info = vk::SubmitInfo2KHR::builder()
            .wait_semaphore_infos(&self.wait_semaphores)
            .command_buffer_infos(&self.command_buffers)
            .signal_semaphore_infos(&self.signal_semaphores);

        let queue = self.queue.lock().ok().ok_or("Poisoned queue lock")?;
        match submit_fn(*queue, 1, &submit_info.build(), vk::Fence::null()) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err)
        }
    }
}

pub struct Executable {
    submissions: Vec<Submission>,
    access_groups: memory::AccessGroupSet,
}

impl Executable {
    pub fn submit(&mut self) -> Result<(), &'static str> {
        let access_info = self.access_groups.enqueue_access()?;
        Err("")
    }
}