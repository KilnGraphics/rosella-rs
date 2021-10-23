use std::sync::Arc;

/*
struct ObjectAllocator<'d> {
    device: &'d ash::vk::Device,
    allocator: vk_mem::Allocator,
}

impl ObjectAllocator {
    fn destroy_buffer(&self, handle: ash::vk::Buffer) {
        unsafe {
            self.device.destroy_buffer(handle);
        }
    }

    fn destroy_image(&self, handle: ash::vk::Image) {
        unsafe {
            self.device.destroy_image(handle);
        }
    }

    fn free_allocation(&self, allocation: &vk_mem::Allocation) {
        self.allocator.free_memory(allocation).unwrap();
    }
}

impl Drop for ObjectAllocator {
    fn drop(&mut self) {
        self.allocator.destroy();
    }
}

pub struct VulkanBufferInfo<'a, 'd: 'a> {
    allocator: &'a ObjectAllocator<'d>,
    allocation: Option<vk_mem::Allocation>,
    handle: ash::vk::Buffer,
}

impl VulkanBufferInfo {
    pub fn get_handle(&self) -> &ash::vk::Buffer {
        &self.handle
    }
}

impl Drop for VulkanBufferInfo {
    fn drop(&mut self) {
        self.allocator.destroy_buffer(self.handle);
    }
}

type VulkanBuffer<'a, 'd> = Arc<VulkanBufferInfo<'a, 'd>>;


pub struct VulkanImageInfo<'a, 'd: 'a> {
    allocator: &'a ObjectAllocator<'d>,
    handle: ash::vk::Image,
}

impl VulkanImageInfo {
    pub fn get_handle(&self) -> &ash::vk::Image {
        handle
    }
}

impl Drop for VulkanImageInfo {
    fn drop(&mut self) {
        self.allocator.destroy_image(self.handle);
    }
}

type VulkanImage<'a, 'd> = Arc<VulkanImageInfo<'a, 'd>>;
 */