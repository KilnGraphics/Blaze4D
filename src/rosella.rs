use std::rc::Rc;
use std::sync::Arc;
use crate::ALLOCATION_CALLBACKS;
use ash::{Entry, Instance};
use ash::vk;

use crate::init::device::{create_device};
use crate::init::initialization_registry::InitializationRegistry;
use crate::init::instance_builder::create_instance;
use crate::window::{RosellaSurface, RosellaWindow};

pub struct Rosella {
    pub instance: Arc<InstanceContext>,
    pub surface: RosellaSurface,
    pub device: Arc<DeviceContext>,
}

impl Rosella {
    pub fn new(registry: InitializationRegistry, window: &RosellaWindow, application_name: &str) -> Rosella {
        let now = std::time::Instant::now();

        let vk_instance = create_instance(&registry, application_name, 0, window);
        let surface = RosellaSurface::new(&vk_instance, &Entry::new(), window);
        let vk_device = create_device(&vk_instance, registry, &surface);

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

        let instance = Arc::new(InstanceContext::new(vk_instance));
        let device = Arc::new(DeviceContext::new(instance.clone(), vk_device));

        Rosella { instance, surface, device }
    }

    pub fn window_update(&self) {}

    pub fn recreate_swapchain(&self, width: u32, height: u32) {
        println!("resize to {}x{}", width, height);
    }
}

pub struct InstanceContext {
    instance: ash::Instance,
}

impl InstanceContext {
    fn new(instance: ash::Instance) -> Self {
        Self{ instance }
    }

    pub fn vk(&self) -> &ash::Instance {
        &self.instance
    }
}

impl Drop for InstanceContext {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(ALLOCATION_CALLBACKS);
        }
    }
}

pub struct DeviceContext {
    instance: Arc<InstanceContext>,
    device: ash::Device,
}

impl DeviceContext {
    fn new(instance: Arc<InstanceContext>, device: ash::Device) -> Self {
        Self{ instance, device }
    }

    pub fn vk(&self) -> &ash::Device {
        &self.device
    }
}

impl Drop for DeviceContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(ALLOCATION_CALLBACKS);
        }
    }
}