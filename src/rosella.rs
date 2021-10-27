use ash::{Entry, Instance};
use crate::ALLOCATION_CALLBACKS;

use crate::init::device::{create_device, RosellaDevice};
use crate::init::initialization_registry::InitializationRegistry;
use crate::init::instance_builder::create_instance;
use crate::window::{RosellaSurface, RosellaWindow};

pub struct Rosella {
    pub device: RosellaDevice,
    pub instance: Instance,
}

impl Rosella {
    pub fn new(registry: InitializationRegistry, window: &RosellaWindow, application_name: &str) -> Rosella {
        let now = std::time::Instant::now();
        let instance = create_instance(&registry, application_name, 0, window);

        let surface = RosellaSurface::new(&instance, &Entry::new(), window);

        let device = create_device(&instance, registry, &surface);

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
                .create_debug_utils_messenger(&debug_info, ALLOCATION_CALLBACKS)
                .unwrap();
        }*/

        Rosella { device, instance }
    }

    pub fn window_update(&self) {}

    pub fn recreate_swapchain(&self, width: u32, height: u32) {
        println!("resize to {}x{}", width, height);
    }
}

impl Drop for Rosella {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(ALLOCATION_CALLBACKS); }
    }
}
