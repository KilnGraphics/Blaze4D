extern crate ash_window;
extern crate winit;

use std::ffi::CString;

use ash::Entry;
use ash::extensions::ext::DebugUtils;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use rosella_rs::init::initialization_registry::InitializationRegistry;
use rosella_rs::rosella::Rosella;

use rosella_rs::window::RosellaWindow;

fn main() {
    let window = RosellaWindow::new("New New Rosella in Rust tm", f64::from(800), f64::from(600));
    let mut registry = InitializationRegistry::new();
    registry.add_required_instance_layer("VK_LAYER_KHRONOS_validation".to_string());
    let rosella = Rosella::new(registry, &window, "new_new_rosella_example_scene_1");

    window.event_loop.run(|_, _, _| {});
}