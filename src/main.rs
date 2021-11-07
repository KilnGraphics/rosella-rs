extern crate ash_window;
extern crate winit;

use std::borrow::Borrow;
use std::collections::HashSet;
use std::ops::BitAnd;
use std::rc::Rc;

use ash::extensions::khr::Swapchain;
use ash::vk::{BufferCreateFlags, BufferCreateInfo, BufferUsageFlags, Format, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, MemoryRequirements, PhysicalDeviceMemoryProperties, QueueFlags, SharingMode};
use ash::Instance;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use rosella_rs::init::device::{ApplicationFeature, DeviceMeta};
use rosella_rs::init::initialization_registry::InitializationRegistry;
use rosella_rs::rosella::Rosella;
use rosella_rs::window::{RosellaSurface, RosellaWindow};
use rosella_rs::{ALLOCATION_CALLBACKS, NamedID};
use rosella_rs::shader::{ComputeContext, ComputeShader, GraphicsContext, GraphicsShader};
use rosella_rs::shader::vertex::{VertexFormat, VertexFormatBuilder, VertexFormatElement};
use rosella_rs::shader::vertex::data_type;

struct QueueFamilyIndices {
    graphics_family: i32,
    present_family: i32,
}

struct QueueFeature;

impl ApplicationFeature for QueueFeature {
    fn get_feature_name(&self) -> NamedID {
        NamedID::new("QueueFeature".to_string())
    }

    fn is_supported(&self, _: &DeviceMeta) -> bool {
        true
    }

    fn enable(&self, meta: &mut DeviceMeta, instance: &Instance, surface: &RosellaSurface) {
        let mut features = meta.feature_builder.vulkan_features.features;
        features.sampler_anisotropy = ash::vk::TRUE;
        features.depth_clamp = ash::vk::TRUE;

        meta.enable_extension(Swapchain::name().as_ptr());

        //TODO: this way of getting queue's gives us a disadvantage. Take advantage of Queue's as much as we can? I will experiment with this once We get "Multithreading capable" parts in. Coding rays feel free to take a look -hydos
        let mut queue_family_indices = QueueFamilyIndices {
            graphics_family: -1,
            present_family: -1,
        };

        let families = unsafe { instance.get_physical_device_queue_family_properties(meta.physical_device) };
        for i in 0..families.len() {
            let family = families
                .get(i)
                .expect("Managed to get broken value while looping over queue families.");

            if queue_family_indices.graphics_family == -1 || queue_family_indices.present_family == -1 {
                if family.queue_flags.contains(QueueFlags::GRAPHICS) {
                    queue_family_indices.graphics_family = i as i32;
                }

                if unsafe {
                    surface
                        .ash_surface
                        .get_physical_device_surface_support(meta.physical_device, i as u32, surface.khr_surface)
                }
                    .unwrap()
                {
                    queue_family_indices.present_family = i as i32;
                }
            }
        }
        meta.add_queue_request(queue_family_indices.graphics_family);
        meta.add_queue_request(queue_family_indices.present_family);
    }

    fn get_dependencies(&self) -> HashSet<NamedID> {
        HashSet::new()
    }
}

fn setup_rosella(window: &RosellaWindow) -> Rosella {
    let mut registry = InitializationRegistry::new();
    registry.add_required_instance_layer("VK_LAYER_KHRONOS_validation".to_string());
    let queue_feature = QueueFeature {};
    registry.register_application_feature(Rc::new(queue_feature)).unwrap();
    registry.add_required_application_feature(QueueFeature {}.get_feature_name());
    Rosella::new(registry, window, "new_new_rosella_example_scene_1")
}

pub fn find_memorytype_index(
    memory_prop: &PhysicalDeviceMemoryProperties,
    flags: MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}


fn main() {
    let window = RosellaWindow::new("New New Rosella in Rust tm", 1396.0, 752.0);
    let mut rosella = setup_rosella(&window);

    // Application Setup usually goes here. Anything in the window loop is either for closing or for looping.
    let basic_vertex_format = VertexFormatBuilder::new()
        .element(data_type::FLOAT, 3)
        .build();

    let triangle_shader = GraphicsShader::new(rosella.device.clone(), include_str!("test_resources/triangle.vert").to_string(), include_str!("test_resources/triangle.frag").to_string(), GraphicsContext {
        mutable_uniforms: HashSet::new(),
        push_uniforms: HashSet::new(),
        vertex_format: basic_vertex_format,
    });
    println!("Successfully created shaders.");

    ///=======================================
    /// COMPUTE TESTING START.
    /// ======================================

    //TODO: a better way of getting the compute queue.
    let compute_queue = unsafe { rosella.device.get_device_queue(0, 0) };

    let buffer_length: usize = 4096;
    let buffer_size: u64 = (buffer_length * 4) as u64;

    let mem_properties = unsafe { rosella.instance.get_physical_device_memory_properties(rosella.device.physical_device) };

    let memory_alloc_info = MemoryAllocateInfo::builder()
        .allocation_size(buffer_size)
        .memory_type_index(find_memorytype_index(&mem_properties, MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT).expect("Oh No..."));

    let input_device_memory = unsafe { rosella.device.allocate_memory(&memory_alloc_info, ALLOCATION_CALLBACKS) }.expect("Failed to allocate memory!");
    let mapped_input_mem = unsafe { rosella.device.map_memory(input_device_memory, 0, buffer_size, MemoryMapFlags::empty()) }.expect("Failed to map memory!");

    let output_device_memory = unsafe { rosella.device.allocate_memory(&memory_alloc_info, ALLOCATION_CALLBACKS) }.expect("Failed to allocate memory!");

    // Fill the memory with 69. Nice
    let mut i = 0;
    while i < buffer_size / 4 { // 4 is the size of i32
        unsafe { *(mapped_input_mem as *mut i8) = 69; }
        i += 1;
    }

    unsafe { rosella.device.unmap_memory(input_device_memory) }

    let mut buffer_create_info = BufferCreateInfo::builder()
        .flags(BufferCreateFlags::empty())
        .size(buffer_size)
        .usage(BufferUsageFlags::STORAGE_BUFFER)
        .sharing_mode(SharingMode::EXCLUSIVE)
        .queue_family_indices(Default::default());

    let input_buffer = unsafe { rosella.device.create_buffer(&buffer_create_info, ALLOCATION_CALLBACKS) }.expect("Failed to create input buffer!");
    unsafe { rosella.device.bind_buffer_memory(input_buffer, input_device_memory, 0); }

    let output_buffer = unsafe { rosella.device.create_buffer(&buffer_create_info, ALLOCATION_CALLBACKS) }.expect("Failed to create output buffer!");
    unsafe { rosella.device.bind_buffer_memory(output_buffer, output_device_memory, 0); }

    ComputeShader::new(rosella.device.clone(), include_str!("test_resources/compute.comp").to_string(), ComputeContext {});

    /// COMPUTE TESTING END.

    window.event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    rosella.recreate_swapchain(new_size.width, new_size.height);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                rosella.window_update();
            }
            _ => (),
        }
    });
}
