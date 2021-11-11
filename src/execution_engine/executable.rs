use std::error::Error;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use ash::vk;
use ash::vk::TimelineSemaphoreSubmitInfoBuilder;
use crate::execution_engine::*;

#[non_exhaustive]
pub enum ExecutionError {
    AccessError(&'static str),
    PoisonedQueueLock,
    SubmitFailed(vk::Result),
}

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

    pub fn submit(&mut self, wait_ops: &Vec<WaitOperation>, signal_ops: &Vec<SignalOperation>, device: &crate::rosella::DeviceContext) -> Result<(), ExecutionError> {
        self.update_semaphores(wait_ops, signal_ops);

        let submit_info = vk::SubmitInfo2KHR::builder()
            .wait_semaphore_infos(&self.wait_semaphores)
            .command_buffer_infos(&self.command_buffers)
            .signal_semaphore_infos(&self.signal_semaphores);

        let queue = self.queue.lock().ok().ok_or(ExecutionError::PoisonedQueueLock)?;
        unsafe{
            device.get_synchronization_2().queue_submit2(*queue, std::slice::from_ref(&submit_info.build()), vk::Fence::null())
        }.map_err(|err| ExecutionError::SubmitFailed(err))
    }
}

pub struct Executable {
    device: Arc<crate::rosella::DeviceContext>,
    submissions: Vec<Submission>,
    access_groups: memory::AccessGroupSet,
}

impl Executable {
    fn make_wait_ops(accesses: &Vec<memory::AccessInfo>) -> Vec<WaitOperation> {
        let mut result = Vec::with_capacity(accesses.len());
        for access in accesses {
            result.push(WaitOperation::TimelineSemaphore(access.semaphore, access.base_access));
        }

        result
    }

    fn make_signal_ops(accesses: &Vec<memory::AccessInfo>) -> Vec<SignalOperation> {
        let mut result = Vec::with_capacity(accesses.len());
        for access in accesses {
            result.push(SignalOperation::TimelineSemaphore(access.semaphore, access.base_access));
        }

        result
    }

    pub fn submit(&mut self) -> Result<(), ExecutionError> {
        let access_info = self.access_groups.enqueue_access().map_err(|msg| ExecutionError::AccessError(msg))?;
        let wait_ops = Self::make_wait_ops(&access_info);
        let signal_ops = Self::make_signal_ops(&access_info);

        for submission in &mut self.submissions {
            submission.submit(&wait_ops, &signal_ops, self.device.as_ref())?;
        }

        Ok(())
    }
}