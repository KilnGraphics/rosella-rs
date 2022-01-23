use crate::shader::vertex::VertexFormat;
use ash::vk::{ShaderModule, ShaderModuleCreateInfo};
use ash::vk::{DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType, GraphicsPipelineCreateInfo, PipelineShaderStageCreateInfo, Sampler, ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags};
use ash::{Device, Entry};
use shaderc::{CompileOptions, Compiler, ShaderKind, TargetEnv};
use std::collections::HashSet;
use std::ffi::CString;
use std::rc::Rc;
use std::sync::Arc;
use crate::rosella::{DeviceContext, Rosella};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Uniform {
    pub name: String,
    pub binding: u32,
    pub stage: ShaderStage,
    pub uniform_type: UniformType,
    pub immutable_samplers: Option<Vec<Sampler>>,
}

pub struct GraphicsContext {
    /// Uniforms which will be changing constantly. For example any object moving in the scene will have their Transformation Matrix here.
    pub mutable_uniforms: HashSet<Uniform>,
    /// Uniforms which stay mostly constant. For example the ProjectionMatrix wont change much and is a good candidate for this.
    pub push_uniforms: HashSet<Uniform>,
    /// The format vertices supplied will be in.
    pub vertex_format: VertexFormat,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum UniformType {
    ImageSampler,
    StorageImage,
    StorageBuffer,
    DynamicStorageBuffer,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex,
    Geometry,
    Fragment,
    Compute,
    All,
    AllGraphics,
}

/// Context relating to compute shaders. For example Inputs, Outputs, etc
pub struct ComputeContext {}

/// Shaders & context needed to render a object.
pub struct GraphicsShader {
    pub device: DeviceContext,
    pub graphics_context: GraphicsContext,
    pub vertex_shader: ShaderModule,
    pub fragment_shader: ShaderModule,
}

/// Shaders & context needed to run compute operations through shaders.
pub struct ComputeShader {
    pub compute_context: ComputeContext,
    pub compute_shader: ShaderModule,
}

impl ComputeShader {
    /// Creates a new ComputeShader based on a glsl shader.
    pub fn new(device: Arc<DeviceContext>, compute_shader: String, compute_context: ComputeContext) -> ComputeShader {
        let mut compiler = Compiler::new().unwrap();
        let mut options = CompileOptions::new().unwrap();

        options.set_target_env(
            TargetEnv::Vulkan,
            Entry::new().try_enumerate_instance_version().ok().flatten().unwrap(),
        );

        let compute_shader = unsafe {
            device.create_shader_module(
                &ShaderModuleCreateInfo::builder().code(
                    compiler
                        .compile_into_spirv(&compute_shader, ShaderKind::Compute, "compute.glsl", "main", Some(&options))
                        .expect("Failed to compile the ComputeShader.")
                        .as_binary(),
                ),
                ALLOCATION_CALLBACKS,
            )
        }.unwrap();

        ComputeShader {
            compute_context,
            compute_shader,
        }
    }
}

impl GraphicsShader {
    /// Creates a new GraphicsShader based on glsl shaders.
    pub fn new(
        device: DeviceContext,
        vertex_shader: String,
        fragment_shader: String,
        graphics_context: GraphicsContext,
    ) -> GraphicsShader {
        let mut compiler = Compiler::new().unwrap();
        let mut options = CompileOptions::new().unwrap();

        options.set_target_env(
            TargetEnv::Vulkan,
            device.get_entry().try_enumerate_instance_version().ok().flatten().unwrap(),
        );

        let vertex_shader = unsafe {
            device.vk().create_shader_module(
                &ShaderModuleCreateInfo::builder().code(
                    compiler
                        .compile_into_spirv(&vertex_shader, ShaderKind::Vertex, "vertex.glsl", "main", Some(&options))
                        .expect("Failed to compile the VertexShader.")
                        .as_binary(),
                ),
                None,
            )
        }.unwrap();

        let fragment_shader = unsafe {
            device.vk().create_shader_module(
                &ShaderModuleCreateInfo::builder().code(
                    compiler
                        .compile_into_spirv(&fragment_shader, ShaderKind::Fragment, "fragment.glsl", "main", Some(&options))
                        .expect("Failed to compile the Fragment Shader.")
                        .as_binary(),
                ),
                None,
            )
        }.unwrap();

        let stage_name = CString::new("main").unwrap();

        // TODO: geometry shader support somehow...
        let stages = vec![
            PipelineShaderStageCreateInfo::builder()
                .stage(ShaderStageFlags::VERTEX)
                .module(vertex_shader)
                .name(&stage_name)
                .build(),
            PipelineShaderStageCreateInfo::builder()
                .stage(ShaderStageFlags::FRAGMENT)
                .module(fragment_shader)
                .name(&stage_name)
                .build(),
        ];

        // TODO: finish
        let graphics_pipeline_create_info = GraphicsPipelineCreateInfo::builder()
            .stages(stages.as_slice());

        GraphicsShader {
            device,
            graphics_context,
            vertex_shader,
            fragment_shader,
        }
    }

    /// Sends a command to run the compute shader.
    pub(crate) fn dispatch() {}
}

impl GraphicsContext {
    pub fn new(vertex_format: VertexFormat) -> GraphicsContext {
        GraphicsContext {
            mutable_uniforms: HashSet::new(),
            push_uniforms: HashSet::new(),
            vertex_format,
        }
    }

    pub fn create_layout(&self, rosella: &Rosella) -> DescriptorSetLayout {
        let mut bindings = vec![];

        for uniform in self.mutable_uniforms.iter() {
            let descriptor_type = match uniform.uniform_type {
                UniformType::ImageSampler => DescriptorType::COMBINED_IMAGE_SAMPLER,
                UniformType::StorageImage => DescriptorType::STORAGE_IMAGE,
                UniformType::StorageBuffer => DescriptorType::STORAGE_BUFFER,
                UniformType::DynamicStorageBuffer => DescriptorType::STORAGE_BUFFER_DYNAMIC,
            };

            let stage = match uniform.stage {
                ShaderStage::Vertex => ShaderStageFlags::VERTEX,
                ShaderStage::Geometry => ShaderStageFlags::GEOMETRY,
                ShaderStage::Fragment => ShaderStageFlags::FRAGMENT,
                ShaderStage::Compute => ShaderStageFlags::COMPUTE,
                ShaderStage::All => ShaderStageFlags::ALL,
                ShaderStage::AllGraphics => ShaderStageFlags::ALL_GRAPHICS,
            };

            bindings.push(DescriptorSetLayoutBinding::builder()
                .binding(uniform.binding)
                .descriptor_type(descriptor_type)
                .descriptor_count(1)
                .stage_flags(stage)
                .immutable_samplers(Default::default())
                .build());
        }

        let layout_create_info = DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings.as_slice()); // The count is handled here for us by the builder.

        unsafe { rosella.device.create_descriptor_set_layout(&layout_create_info, ALLOCATION_CALLBACKS) }.expect("Failed to create VkDescriptorSetLayout.")
    }
}

impl Drop for GraphicsShader {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_shader_module(self.vertex_shader, None);
            self.device.vk().destroy_shader_module(self.fragment_shader, None);
        }
    }
}

impl Drop for ComputeShader {
    fn drop(&mut self) {}
}
