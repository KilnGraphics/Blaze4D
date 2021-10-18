mod rosella;

use std::ffi::CString;
use ash::Entry;
use ash::extensions::ext::DebugUtils;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use rosella::window::RosellaWindow;

fn main() {
    let window = RosellaWindow::new("Rosella Rust TM", f64::from(800), f64::from(600));

    let vk = Entry::new();
    let application_name = CString::new("Rosella In Rust Test. Keep Malding");

/*    let surface_extensions = ash_window::enumerate_required_extensions(&window.handle).unwrap();
    let mut extension_names_raw = surface_extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect::<Vec<_>>();
    extension_names_raw.push(DebugUtils::name().as_ptr());

    let debug_utils_loader = DebugUtils::new(&vk, &instance);

    unsafe {
        let debug_call_back = debug_utils_loader
            .create_debug_utils_messenger(&debug_info, None)
            .unwrap();
    }*/
}