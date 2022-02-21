use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Pointer};
use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use ash::prelude::VkResult;

use ash::vk;

use crate::init::EnabledFeatures;
use crate::instance::InstanceContext;
use crate::objects::id::SurfaceId;
use crate::objects::surface::{Surface, SurfaceCapabilities};
use crate::util::extensions::{AsRefOption, ExtensionFunctionSet, VkExtensionInfo, VkExtensionFunctions};
use crate::{NamedUUID, UUID};
use crate::execution_engine::ops::Op;
use crate::objects::allocator::Allocator;

struct QueueCommandPool {
    standard: Mutex<vk::CommandPool>,
    one_time: Mutex<vk::CommandPool>,
}

impl QueueCommandPool {
    fn new(device: &ash::Device, queue_family: u32) -> Self {
        let standard_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family);

        let standard = unsafe {
            device.create_command_pool(&standard_info, None)
        }.unwrap();

        let one_time_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue_family);

        let one_time = unsafe {
            device.create_command_pool(&one_time_info, None)
        }.unwrap();

        Self {
            standard: Mutex::new(standard),
            one_time: Mutex::new(one_time),
        }
    }

    fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_command_pool(*self.standard.get_mut().unwrap(), None);
            device.destroy_command_pool(*self.one_time.get_mut().unwrap(), None);
        }
    }
}

struct DeviceContextImpl {
    id: NamedUUID,
    instance: InstanceContext,
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
    extensions: ExtensionFunctionSet,
    allocator: ManuallyDrop<Allocator>, // We need manually drop to ensure it is dropped before the device
    command_pools: Box<[QueueCommandPool]>,
    features: EnabledFeatures,
    surfaces: HashMap<SurfaceId, (Surface, SurfaceCapabilities)>,
}

impl Drop for DeviceContextImpl {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.allocator);

            for command_pool in self.command_pools.iter_mut() {
                command_pool.destroy(&self.device);
            }

            self.device.destroy_device(None);
        }
    }
}

#[derive(Clone)]
pub struct DeviceContext(Arc<DeviceContextImpl>);

impl DeviceContext {
    pub fn new(instance: InstanceContext, device: ash::Device, physical_device: vk::PhysicalDevice, extensions: ExtensionFunctionSet, features: EnabledFeatures, surfaces: &[Surface]) -> Self {
        let surfaces : HashMap<_, _> = surfaces.iter().map(|surface| {
            (surface.get_id(), (surface.clone(), SurfaceCapabilities::new(&instance, physical_device, surface.get_handle()).unwrap()))
        }).collect();

        let allocator = Allocator::new(instance.vk().clone(), device.clone(), physical_device);

        let queue_count = unsafe { instance.vk().get_physical_device_queue_family_properties(physical_device) }.len();
        let mut command_pools = Vec::with_capacity(queue_count);
        for i in 0..(queue_count as u32) {
            command_pools.push(QueueCommandPool::new(&device, i))
        }
        let command_pools = command_pools.into_boxed_slice();

        Self(Arc::new(DeviceContextImpl{
            id: NamedUUID::with_str("Device"),
            instance,
            device,
            physical_device,
            extensions,
            allocator: ManuallyDrop::new(allocator),
            command_pools,
            features,
            surfaces,
        }))
    }

    pub fn get_uuid(&self) -> &NamedUUID {
        &self.0.id
    }

    pub fn get_entry(&self) -> &ash::Entry {
        self.0.instance.get_entry()
    }

    pub fn get_instance(&self) -> &InstanceContext {
        &self.0.instance
    }

    pub fn vk(&self) -> &ash::Device {
        &self.0.device
    }

    pub fn get_physical_device(&self) -> &vk::PhysicalDevice {
        &self.0.physical_device
    }

    pub fn get_extension<T: VkExtensionInfo>(&self) -> Option<&T> where VkExtensionFunctions: AsRefOption<T> {
        self.0.extensions.get()
    }

    pub fn is_extension_enabled(&self, uuid: UUID) -> bool {
        self.0.extensions.contains(uuid)
    }

    pub fn get_allocator(&self) -> &Allocator {
        &self.0.allocator
    }

    pub fn get_enabled_features(&self) -> &EnabledFeatures {
        &self.0.features
    }

    pub fn get_surface(&self, id: SurfaceId) -> Option<Surface> {
        self.0.surfaces.get(&id).map(|data| data.0.clone())
    }

    pub fn get_surface_capabilities(&self, id: SurfaceId) -> Option<&SurfaceCapabilities> {
        self.0.surfaces.get(&id).map(|(_, cap)| cap)
    }

    pub fn record_standard<'a>(&self, ops: &[&'a dyn Op], queue: u32) -> VkResult<vk::CommandBuffer> {
        let guard = self.0.command_pools.get(queue as usize).unwrap().standard.lock().unwrap();

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(*guard)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe {
            self.0.device.allocate_command_buffers(&allocate_info)
        }?.remove(0);

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe {
            self.0.device.begin_command_buffer(command_buffer, &begin_info)
        }.map_err(|err| unsafe {
            self.0.device.free_command_buffers(*guard, std::slice::from_ref(&command_buffer));
            err
        })?;

        for op in ops.iter() {
            op.record(command_buffer);
        }

        unsafe {
            self.0.device.end_command_buffer(command_buffer)
        }.map_err(|err| unsafe {
            self.0.device.free_command_buffers(*guard, std::slice::from_ref(&command_buffer));
            err
        })?;

        Ok(command_buffer)
    }
}

impl PartialEq for DeviceContext {
    fn eq(&self, other: &Self) -> bool {
        self.0.id.eq(&other.0.id)
    }
}

impl Eq for DeviceContext {
}

impl PartialOrd for DeviceContext {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.id.partial_cmp(&other.0.id)
    }
}

impl Ord for DeviceContext {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.id.cmp(&other.0.id)
    }
}

impl Debug for DeviceContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}