use crate::shader::vertex::VertexFormat;
use ash::vk::{ShaderModule, ShaderModuleCreateInfo};
use ash::{Device, Entry};
use shaderc::{CompileOptions, Compiler, ShaderKind, TargetEnv};
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
use crate::rosella::DeviceContext;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Uniform {
    pub name: String,
    //TODO: the rest of this
}

pub struct GraphicsContext {
    /// Uniforms which will be changing constantly. For example any object moving in the scene will have their Transformation Matrix here.
    pub mutable_uniforms: HashSet<Uniform>,
    /// Uniforms which stay mostly constant. For example the ProjectionMatrix wont change much and is a good candidate for this.
    pub push_uniforms: HashSet<Uniform>,
    /// The format vertices supplied will be in.
    pub vertex_format: VertexFormat,
}


pub struct ShaderStage {}

/// Context relating to compute shaders. For example Inputs, Outputs, etc
pub struct ComputeContext {
    /// The stage at when the compute shader will be run.
    pub state: ShaderStage,
}

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
                        .expect("Failed to compile the FragmentShader.")
                        .as_binary(),
                ),
                None,
            )
        }.unwrap();

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
