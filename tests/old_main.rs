mod test_common;

extern crate ash_window;
extern crate winit;

use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use rosella_rs::init::initialization_registry::InitializationRegistry;
use rosella_rs::init::rosella_features::{register_rosella_debug, register_rosella_headless};
use rosella_rs::rosella::Rosella;
use rosella_rs::window::RosellaWindow;
use rosella_rs::shader::{GraphicsContext, GraphicsShader};
use rosella_rs::shader::vertex::VertexFormatBuilder;
use rosella_rs::shader::vertex::data_type;

fn setup_rosella(window: &RosellaWindow) -> Rosella {
    let mut registry = InitializationRegistry::new();

    register_rosella_headless(&mut registry);
    register_rosella_debug(&mut registry, false);

    match Rosella::new(registry, window, "new_new_rosella_example_scene_1") {
        Ok(rosella) => rosella,
        Err(err) => panic!("Failed to create Rosella {:?}", err)
    }
}

fn main() {
    env_logger::init();

    let window = RosellaWindow::new("New New Rosella in Rust tm", 1396.0, 752.0);
    let rosella = setup_rosella(&window);

    // Application Setup usually goes here. Anything in the window loop is either for closing or for looping.
    let basic_vertex_format = VertexFormatBuilder::new()
        .element(data_type::FLOAT, 3)
        .build();

    GraphicsShader::new(rosella.device.clone(), include_str!("test_resources/triangle.vert").to_string(), include_str!("test_resources/triangle.frag").to_string(), GraphicsContext {
        mutable_uniforms: Default::default(),
        push_uniforms: Default::default(),
        vertex_format: basic_vertex_format,
    });
    println!("Successfully created shaders.");

    /*window.event_loop.run(move |event, _, control_flow| {
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
    });*/
}
