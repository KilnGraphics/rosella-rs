use ash::vk;
use ash::vk::{SemaphoreSubmitInfoKHR, SubmitInfo2KHRBuilder};
use bumpalo::Bump;
use crate::device::DeviceContext;

/// Represents a
pub trait Submittable {

    /// Submits the commands in this submittable for execution.
    fn submit(&self);

    /// Returns the queue family that this submittable needs to be submitted on.
    fn get_queue_family(&self) -> u32;

    /// Generates a submit info for this submittable
    ///
    /// The provided bump allocator should be used for dynamic heap allocation.
    ///
    /// A set of wait and signal semaphores may be provided. The reference to the wait and signal
    /// semaphores (as well as any pNext entry) must live at least as long as the allocator.
    ///
    /// Calling this function may queue accesses to synchronization groups. As such any submit info
    /// returned from this function **must** be submitted or forward progress may halt.
    unsafe fn generate_submit_info<'a>(&self, wait_semaphores: &'a [SemaphoreSubmitInfoKHR], signal_semaphores: &'a [SemaphoreSubmitInfoKHR], allocator: &'a bumpalo::Bump) -> vk::SubmitInfo2KHRBuilder<'a>;
}

struct BasicSubmittable {
    sync2: ash::extensions::khr::Synchronization2,
    q: vk::Queue,
    queue: u32,
    buffer: vk::CommandBuffer,
}

impl Submittable for BasicSubmittable {
    fn submit(&self) {
        let alloc = bumpalo::Bump::new();
        let submit_info = unsafe { self.generate_submit_info(&[], &[], &alloc) }.build();

        unsafe {
            self.sync2.queue_submit2(self.q, std::slice::from_ref(&submit_info), vk::Fence::null())
        };
    }

    fn get_queue_family(&self) -> u32 {
        self.queue
    }

    unsafe fn generate_submit_info<'a>(&self, wait_semaphores: &'a [SemaphoreSubmitInfoKHR], signal_semaphores: &'a [SemaphoreSubmitInfoKHR], allocator: &'a Bump) -> SubmitInfo2KHRBuilder<'a> {
        let buffer_info = vk::CommandBufferSubmitInfoKHR::builder()
            .command_buffer(self.buffer)
            .device_mask(0);
        let buffer_info = std::slice::from_ref(allocator.alloc(buffer_info.build()));

        vk::SubmitInfo2KHR::builder()
            .wait_semaphore_infos(wait_semaphores)
            .command_buffer_infos(buffer_info)
            .signal_semaphore_infos(signal_semaphores)
    }
}