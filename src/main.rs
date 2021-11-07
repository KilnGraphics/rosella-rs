extern crate ash_window;
extern crate winit;

use std::borrow::Borrow;
use std::collections::HashSet;
use std::ffi::CString;
use std::ops::BitAnd;
use std::rc::Rc;

use ash::extensions::khr::Swapchain;
use ash::vk::{AccessFlags, Buffer, BufferCreateFlags, BufferCreateInfo, BufferImageCopy, BufferUsageFlags, ComponentMapping, ComponentSwizzle, ComputePipelineCreateInfo, DependencyFlags, DescriptorBufferInfo, DescriptorImageInfo, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSetAllocateInfo, DescriptorSetLayoutBinding, DescriptorSetLayoutBindingBuilder, DescriptorSetLayoutCreateInfo, DescriptorType, DeviceMemory, Extent3D, Format, Handle, ImageAspectFlags, ImageCreateInfo, ImageLayout, ImageMemoryBarrier, ImageSubresourceLayers, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageViewCreateInfo, ImageViewType, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, MemoryRequirements, PhysicalDeviceMemoryProperties, PipelineCache, PipelineCreateFlags, PipelineLayoutCreateInfo, PipelineShaderStageCreateInfo, PipelineStageFlags, QueueFlags, SampleCountFlags, ShaderStageFlags, SharingMode, WriteDescriptorSet};
use ash::Instance;
use ash::util::Align;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use rosella_rs::init::device::{ApplicationFeature, DeviceMeta, RosellaDevice};
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

unsafe fn load_image(buffer: &[u8], device: &RosellaDevice, device_memory_properties: &PhysicalDeviceMemoryProperties) -> (u64, Buffer, (u32, u32)) {
    let image = image::load_from_memory(buffer)
        .unwrap()
        .to_rgba8();
    let image_dimensions = image.dimensions();
    let image_data = image.into_raw();
    let size = (std::mem::size_of::<u8>() * image_data.len()) as u64;
    let mut buffer_create_info = BufferCreateInfo::builder()
        .flags(BufferCreateFlags::empty())
        .size(size)
        .usage(BufferUsageFlags::STORAGE_BUFFER)
        .sharing_mode(SharingMode::EXCLUSIVE)
        .queue_family_indices(Default::default());
    let image_buffer = device.create_buffer(&buffer_create_info, None).unwrap();
    let image_buffer_memory_req = device.get_buffer_memory_requirements(image_buffer);
    let image_buffer_memory_index = find_memorytype_index(
        &device_memory_properties,
        MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
    ).expect("Unable to find suitable memorytype for the vertex buffer.");

    let image_buffer_allocate_info = MemoryAllocateInfo {
        allocation_size: image_buffer_memory_req.size,
        memory_type_index: image_buffer_memory_index,
        ..Default::default()
    };
    let image_buffer_memory = device
        .allocate_memory(&image_buffer_allocate_info, None)
        .unwrap();
    let image_ptr = device
        .map_memory(
            image_buffer_memory,
            0,
            image_buffer_memory_req.size,
            MemoryMapFlags::empty(),
        )
        .unwrap();
    let mut image_slice = Align::new(
        image_ptr,
        std::mem::align_of::<u8>() as u64,
        image_buffer_memory_req.size,
    );
    image_slice.copy_from_slice(&image_data);
    device.unmap_memory(image_buffer_memory);
    device
        .bind_buffer_memory(image_buffer, image_buffer_memory, 0)
        .unwrap();

    (size, image_buffer, image_dimensions)
}

/* TODO: FINISH. A port of the example code from Ash. It needs to be able to take stuff from the above function and create an Image with it.
unsafe fn upload_image(width: u32, height: u32, device: RosellaDevice, properties: PhysicalDeviceMemoryProperties) {
    let texture_create_info = ImageCreateInfo {
        image_type: ImageType::TYPE_2D,
        format: Format::R8G8B8A8_UNORM,
        extent: Extent3D {
            width,
            height,
            depth: 1,
        },
        mip_levels: 1,
        array_layers: 1,
        samples: SampleCountFlags::TYPE_1,
        tiling: ImageTiling::OPTIMAL,
        usage: ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED,
        sharing_mode: SharingMode::EXCLUSIVE,
        ..Default::default()
    };
    let texture_image = device
        .create_image(&texture_create_info, None)
        .unwrap();
    let texture_memory_req = device.get_image_memory_requirements(texture_image);
    let texture_memory_index = find_memorytype_index(
        &properties,
        MemoryPropertyFlags::DEVICE_LOCAL,
    )
        .expect("Unable to find suitable memory index for depth image.");

    let texture_allocate_info = MemoryAllocateInfo {
        allocation_size: texture_memory_req.size,
        memory_type_index: texture_memory_index,
        ..Default::default()
    };
    let texture_memory = device
        .allocate_memory(&texture_allocate_info, None)
        .unwrap();
    device
        .bind_image_memory(texture_image, texture_memory, 0)
        .expect("Unable to bind depth image memory");

    record_submit_commandbuffer(
        &device,
        base.setup_command_buffer,
        base.setup_commands_reuse_fence,
        base.present_queue,
        &[],
        &[],
        &[],
        |device, texture_command_buffer| {
            let texture_barrier = ImageMemoryBarrier {
                dst_access_mask: AccessFlags::TRANSFER_WRITE,
                new_layout: ImageLayout::TRANSFER_DST_OPTIMAL,
                image: texture_image,
                subresource_range: ImageSubresourceRange {
                    aspect_mask: ImageAspectFlags::COLOR,
                    level_count: 1,
                    layer_count: 1,
                    ..Default::default()
                },
                ..Default::default()
            };
            device.cmd_pipeline_barrier(
                texture_command_buffer,
                PipelineStageFlags::BOTTOM_OF_PIPE,
                PipelineStageFlags::TRANSFER,
                DependencyFlags::empty(),
                &[],
                &[],
                &[texture_barrier],
            );
            let buffer_copy_regions = BufferImageCopy::builder()
                .image_subresource(
                    ImageSubresourceLayers::builder()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .layer_count(1)
                        .build(),
                )
                .image_extent(Extent3D {
                    width,
                    height,
                    depth: 1,
                });

            device.cmd_copy_buffer_to_image(
                texture_command_buffer,
                image_buffer,
                texture_image,
                ImageLayout::TRANSFER_DST_OPTIMAL,
                &[buffer_copy_regions.build()],
            );
            let texture_barrier_end = ImageMemoryBarrier {
                src_access_mask: AccessFlags::TRANSFER_WRITE,
                dst_access_mask: AccessFlags::SHADER_READ,
                old_layout: ImageLayout::TRANSFER_DST_OPTIMAL,
                new_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                image: texture_image,
                subresource_range: ImageSubresourceRange {
                    aspect_mask: ImageAspectFlags::COLOR,
                    level_count: 1,
                    layer_count: 1,
                    ..Default::default()
                },
                ..Default::default()
            };
            device.cmd_pipeline_barrier(
                texture_command_buffer,
                PipelineStageFlags::TRANSFER,
                PipelineStageFlags::FRAGMENT_SHADER,
                DependencyFlags::empty(),
                &[],
                &[],
                &[texture_barrier_end],
            );
        },
    );
}*/

fn main() {
    let window = RosellaWindow::new("New New Rosella in Rust tm", 800.0, 500.0);
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

    let mem_properties = unsafe { rosella.instance.get_physical_device_memory_properties(rosella.device.physical_device) };
    let image = unsafe { load_image(include_bytes!("test_resources/help_me_16.png"), &rosella.device, &mem_properties) };
    let buffer_size = image.0;

    let memory_alloc_info = MemoryAllocateInfo::builder()
        .allocation_size(buffer_size)
        .memory_type_index(find_memorytype_index(&mem_properties, MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT).expect("Oh No..."));

    let output_device_memory = unsafe { rosella.device.allocate_memory(&memory_alloc_info, ALLOCATION_CALLBACKS) }.expect("Failed to allocate memory!");

    let mut buffer_create_info = BufferCreateInfo::builder()
        .flags(BufferCreateFlags::empty())
        .size(buffer_size)
        .usage(BufferUsageFlags::STORAGE_BUFFER)
        .sharing_mode(SharingMode::EXCLUSIVE)
        .queue_family_indices(Default::default());

    // Binding input buffer memory is already done in the image func for the input image.
    let output_buffer = unsafe { rosella.device.create_buffer(&buffer_create_info, ALLOCATION_CALLBACKS) }.expect("Failed to create output buffer!");
    unsafe { rosella.device.bind_buffer_memory(output_buffer, output_device_memory, 0); }

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

    // Create some image info
    let tex_image_view_info = ImageViewCreateInfo {
        view_type: ImageViewType::TYPE_2D,
        format: texture_create_info.format,
        components: ComponentMapping {
            r: ComponentSwizzle::R,
            g: ComponentSwizzle::G,
            b: ComponentSwizzle::B,
            a: ComponentSwizzle::A,
        },
        subresource_range: ImageSubresourceRange {
            aspect_mask: ImageAspectFlags::COLOR,
            level_count: 1,
            layer_count: 1,
            ..Default::default()
        },
        image: texture_image,
        ..Default::default()
    };

    let input_descriptor_buffer_info = DescriptorImageInfo::builder()
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
            .descriptor_type(DescriptorType::STORAGE_IMAGE)
            .image_info(descriptor_image_infos.as_slice()),
        WriteDescriptorSet::builder(),
    ];
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
