use ash::vk::{Buffer, BufferUsageFlags};
use rosella_rs::init::{InitializationRegistry, register_rosella_debug, register_rosella_headless};
use rosella_rs::objects::buffer::BufferCreateDesc;
use rosella_rs::objects::SynchronizationGroup;
use rosella_rs::rosella::Rosella;
use rosella_rs::shader::{GraphicsContext, GraphicsShader};
use rosella_rs::shader::vertex::{data_type, VertexFormatBuilder};
use rosella_rs::window::RosellaWindow;

fn main() {
    let position_format = VertexFormatBuilder::new()
        .element(data_type::FLOAT, 3)
        .build();

    let window = RosellaWindow::new("Pain", 800.0, 600.0);
    let rosella = setup_rosella(&window);

    GraphicsShader::new(rosella.device.clone(), include_str!("resources/triangle.vert").to_string(), include_str!("resources/triangle.frag").to_string(), GraphicsContext {
        mutable_uniforms: Default::default(),
        push_uniforms: Default::default(),
        vertex_format: position_format,
    });
    println!("Successfully created shaders.");

    // Vertex Buffer stuff
    let group = rosella.object_manager.create_synchronization_group();
    let mut builder = rosella.object_manager.create_object_set(group);
    let vertex_buffer_id = builder.add_default_gpu_only_buffer(BufferCreateDesc::new_simple(4 * 3 * 3, BufferUsageFlags::VERTEX_BUFFER)); // FLOAT_SIZE * float count per vertex (3) * vertex count (3)
    let object_set = builder.build();
    let vertex_buffer = object_set.get_buffer_handle(vertex_buffer_id).unwrap();

    loop {
        rosella.window_update()
    }
}

fn setup_rosella(window: &RosellaWindow) -> Rosella {
    let mut registry = InitializationRegistry::new();

    register_rosella_headless(&mut registry);
    register_rosella_debug(&mut registry, false);

    match Rosella::new(registry, window, "new_new_rosella_example_scene_1") {
        Ok(rosella) => rosella,
        Err(err) => panic!("Failed to create Rosella {:?}", err)
    }
}