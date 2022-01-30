extern crate ash_window;
extern crate winit;

use ash::vk;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use rosella_rs::init::initialization_registry::InitializationRegistry;
use rosella_rs::init::rosella_features::{register_rosella_debug, register_rosella_headless, register_rosella_present};
use rosella_rs::objects::Format;
use rosella_rs::objects::swapchain::{SwapchainCreateDesc, SwapchainImageSpec};
use rosella_rs::rosella::Rosella;
use rosella_rs::window::RosellaWindow;
use rosella_rs::shader::{GraphicsContext, GraphicsShader};
use rosella_rs::shader::vertex::VertexFormatBuilder;
use rosella_rs::shader::vertex::data_type;

fn setup_rosella(window: &RosellaWindow) -> Rosella {
    let mut registry = InitializationRegistry::new();

    register_rosella_headless(&mut registry);
    register_rosella_present(&mut registry);
    register_rosella_debug(&mut registry, false);

    match Rosella::new(registry, window, "new_new_rosella_example_scene_1") {
        Ok(rosella) => rosella,
        Err(err) => panic!("Failed to create Rosella {:?}", err)
    }
}

fn main() {
    env_logger::init();

    let window = RosellaWindow::new("New New Rosella in Rust tm", 1000.0, 700.0);
    let rosella = setup_rosella(&window);
    window.handle.set_visible(true);

    // Application Setup usually goes here. Anything in the window loop is either for closing or for looping.
    let basic_vertex_format = VertexFormatBuilder::new()
        .element(data_type::FLOAT, 3)
        .build();

    GraphicsShader::new(rosella.device.clone(), include_str!("resources/triangle.vert").to_string(), include_str!("resources/triangle.frag").to_string(), GraphicsContext {
        mutable_uniforms: Default::default(),
        push_uniforms: Default::default(),
        vertex_format: basic_vertex_format,
    });
    println!("Successfully created shaders.");

    let capabilities = rosella.device.get_surface_capabilities(rosella.surface).unwrap();
    println!("Capabilities: {:?}", capabilities.get_capabilities());

    let surface_format = capabilities.get_surface_formats().get(0).unwrap();

    let desc = SwapchainCreateDesc::make(
        SwapchainImageSpec::make(
            Format::format_for(surface_format.format),
            surface_format.color_space,
            1000,
            700,
        ),
        capabilities.get_capabilities().min_image_count,
        vk::ImageUsageFlags::COLOR_ATTACHMENT,
        *capabilities.get_present_modes().get(0).unwrap()
    );

    let swapchain_set = rosella.object_manager.create_swapchain_object_set(rosella.surface, desc).build();

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

    drop(swapchain_set)
}
