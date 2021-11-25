extern crate ash_window;
extern crate winit;

use std::collections::HashSet;
use std::ffi::CString;
use std::rc::Rc;

use ash::extensions::khr::Swapchain;
use ash::vk::{CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel, CommandBufferResetFlags, CommandBufferUsageFlags, CommandPoolCreateFlags, CommandPoolCreateInfo, ComponentMapping, ComponentSwizzle, ComputePipelineCreateInfo, DependencyFlags, DescriptorBufferInfo, DescriptorImageInfo, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSetAllocateInfo, DescriptorSetLayoutBinding, DescriptorSetLayoutBindingBuilder, DescriptorSetLayoutCreateInfo, DescriptorType, DeviceMemory, Extent3D, Fence, FenceCreateFlags, FenceCreateInfo, Format, Handle, ImageAspectFlags, ImageCreateInfo, ImageLayout, ImageMemoryBarrier, ImageSubresourceLayers, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageViewCreateInfo, ImageViewType, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, MemoryRequirements, PhysicalDeviceMemoryProperties, PipelineCache, PipelineCreateFlags, PipelineLayoutCreateInfo, PipelineShaderStageCreateInfo, PipelineStageFlags, Queue, QueueFlags, SampleCountFlags, Semaphore, ShaderStageFlags, SharingMode, SubmitInfo, WriteDescriptorSet};
use ash::Instance;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use rosella_rs::init::device::{ApplicationFeature, DeviceMeta};
use rosella_rs::init::initialization_registry::InitializationRegistry;
use rosella_rs::rosella::Rosella;
use rosella_rs::window::{RosellaSurface, RosellaWindow};
use rosella_rs::{ALLOCATION_CALLBACKS, NamedID};
use rosella_rs::shader::{ComputeContext, ComputeShader, GraphicsContext, GraphicsShader};
use rosella_rs::shader::vertex::{VertexFormatBuilder};
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
    let window = RosellaWindow::new("Rosella Tests", 800.0, 500.0);
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
    let compute_shader = ComputeShader::new(rosella.device.clone(), include_str!("test_resources/compute.comp").to_string(), ComputeContext {});

    /*let image = unsafe { load_image(include_bytes!("test_resources/help_me_16.png"), &rosella.device, &mem_properties) };
    let buffer_size = image.0;

    let memory_alloc_info = MemoryAllocateInfo::builder()
        .allocation_size(buffer_size)
        .memory_type_index(find_memorytype_index(&mem_properties, MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT).expect("Oh No..."));

    let output_device_memory = unsafe { rosella.device.allocate_memory(&memory_alloc_info, ALLOCATION_CALLBACKS) }.expect("Failed to allocate memory!");

    let mut buffer_create_info = BufferCreateInfo::builder()
        .flags(BufferCreateFlags::empty())
        .size(buffer_size)
        .usage(BufferUsageFlags::StorageBuffer)
        .sharing_mode(SharingMode::EXCLUSIVE)
        .queue_family_indices(Default::default());

    // Binding input buffer memory is already done in the image func for the input image.
    let output_buffer = unsafe { rosella.device.create_buffer(&buffer_create_info, ALLOCATION_CALLBACKS) }.expect("Failed to create output buffer!");
    unsafe { rosella.device.bind_buffer_memory(output_buffer, output_device_memory, 0); }*/

    let mut bindings = vec![];

    // The input buffer.
    bindings.push(DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(DescriptorType::STORAGE_IMAGE)
        .descriptor_count(1)
        .stage_flags(ShaderStageFlags::COMPUTE)
        .immutable_samplers(Default::default())
        .build());

    // The output buffer.
    bindings.push(DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(DescriptorType::STORAGE_IMAGE)
        .descriptor_count(1)
        .stage_flags(ShaderStageFlags::COMPUTE)
        .immutable_samplers(Default::default())
        .build());

    let layout_create_info = DescriptorSetLayoutCreateInfo::builder()
        .bindings(bindings.as_slice()); // The count is handled here for us by the builder.

    let descriptor_set_layout = unsafe { rosella.device.create_descriptor_set_layout(&layout_create_info, ALLOCATION_CALLBACKS) }.expect("Failed to create VkDescriptorSetLayout.");
    let descriptor_set_layouts = vec![descriptor_set_layout]; // Thanks, Ash

    // Pipeline stuff
    let pipeline_layout_create_info = PipelineLayoutCreateInfo::builder()
        .set_layouts(descriptor_set_layouts.as_slice());
    let pipeline_layout = unsafe { rosella.device.create_pipeline_layout(&pipeline_layout_create_info, ALLOCATION_CALLBACKS) }.expect("Failed to create VkPipelineLayout");

    let stage_name = CString::new("main").unwrap();
    let compute_pipeline_create_info = ComputePipelineCreateInfo::builder()
        .stage(PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::COMPUTE)
            .module(compute_shader.compute_shader)
            .name(&stage_name)
            .build()
        )
        .layout(pipeline_layout);
    let compute_pipeline_create_infos = vec![compute_pipeline_create_info.build()];
    let compute_pipeline = unsafe { rosella.device.create_compute_pipelines(PipelineCache::default(), compute_pipeline_create_infos.as_slice(), ALLOCATION_CALLBACKS) }.expect("Failed to create VkPipeline");

    let pool_sizes = vec![DescriptorPoolSize::builder()
        .descriptor_count(2)
        .build()
    ];

    let pool_create_info = DescriptorPoolCreateInfo::builder()
        .max_sets(1)
        .pool_sizes(pool_sizes.as_slice());
    let descriptor_pool = unsafe { rosella.device.create_descriptor_pool(&pool_create_info, ALLOCATION_CALLBACKS) }.expect("Failed to create VkDescriptorPool!");

    let descriptor_set_alloc_info = DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(descriptor_set_layouts.as_slice());
    let descriptor_set = unsafe { rosella.device.allocate_descriptor_sets(&descriptor_set_alloc_info) }.expect("Fail.");

    // Command Stuff for image stuff
    let pool_create_info = CommandPoolCreateInfo::builder()
        .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(0);

    let pool = unsafe { rosella.device.create_command_pool(&pool_create_info, None) }.unwrap();

    let command_buffer_allocate_info = CommandBufferAllocateInfo::builder()
        .command_buffer_count(2)
        .command_pool(pool)
        .level(CommandBufferLevel::PRIMARY);

    let command_buffers = unsafe { rosella.device.allocate_command_buffers(&command_buffer_allocate_info) }.unwrap();
    let setup_cmd_buffer = command_buffers[0];

    // Create some image info
    // unsafe { upload_image(image.2.0, image.2.1, setup_cmd_buffer, image.1, compute_queue, rosella.device.borrow(), &mem_properties) }

/*    let input_descriptor_buffer_info = DescriptorImageInfo::builder()
        .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(unsafe { rosella.device.create_image_view(&tex_image_view_info, None) }.unwrap())
        .build();

    let out_descriptor_buffer_info = DescriptorImageInfo::builder()
        .image_layout(ImageLayout::UNDEFINED)
        .image_view(unsafe { rosella.device.create_image_view(&tex_image_view_info, None) }.unwrap())
        .build();
    let descriptor_image_infos = vec![input_descriptor_buffer_info, out_descriptor_buffer_info];

    let write_descriptor_sets = vec![
        WriteDescriptorSet::builder()
            .dst_set(des)
            .dst_binding(0)
            .descriptor_type(DescriptorType::StorageImage)
            .image_info(descriptor_image_infos.as_slice()),
        WriteDescriptorSet::builder(),
    ];*/
    //TODO: Finish image support so this is possible.


    println!("If you are reading this text then Compute has finished!");
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
