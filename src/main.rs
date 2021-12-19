extern crate ash_window;
extern crate winit;

use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use rosella_rs::init::initialization_registry::InitializationRegistry;
use rosella_rs::init::rosella_features::register_rosella_headless;
use rosella_rs::rosella::{Rosella, RosellaCreateError};
use rosella_rs::window::RosellaWindow;
use rosella_rs::shader::{GraphicsContext, GraphicsShader};
use rosella_rs::shader::vertex::VertexFormatBuilder;
use rosella_rs::shader::vertex::data_type;

struct QueueFamilyIndices {
    graphics_family: i32,
    present_family: i32,
}

struct QueueFeature;

/*
impl QueueFeature {
    fn get_feature_name(&self) -> NamedUUID {
        NamedUUID::new("QueueFeature".to_string())
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

    fn get_dependencies(&self) -> HashSet<NamedUUID> {
        HashSet::new()
    }
}*/

fn setup_rosella(window: &RosellaWindow) -> Rosella {
    let mut registry = InitializationRegistry::new();
    register_rosella_headless(&mut registry);
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
