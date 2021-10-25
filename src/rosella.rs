use ash::Entry;

use crate::init::device::{DeviceBuilder, RosellaDevice};
use crate::init::initialization_registry::InitializationRegistry;
use crate::init::instance_builder::create_instance;
use crate::window::{RosellaSurface, RosellaWindow};

pub struct Rosella {
    device: RosellaDevice,
}

impl Rosella {
    pub fn new(registry: InitializationRegistry, window: &RosellaWindow, application_name: &str) -> Rosella {
        let now = std::time::Instant::now();
        let instance = create_instance(&registry, application_name, 0, window);

        let surface = RosellaSurface::new(&instance, &Entry::new(), window);

        let device = DeviceBuilder { instance }.build(registry, &surface);

        let elapsed = now.elapsed();
        println!("Instance & Device Initialization took: {:.2?}", elapsed);

        /*        let vk = Entry::new();
        let app_name = CString::new(application_name);
        let surface_extensions = ash_window::enumerate_required_extensions(&window.handle).unwrap();
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

        Rosella { device }
    }

    pub fn window_update(&self) {}
}
