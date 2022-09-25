use std::ffi::CString;
use std::sync::Arc;

use ash::vk;
use crate::BUILD_INFO;

use crate::instance::debug_messenger::RustLogDebugMessenger;
use crate::device::init::{create_device, DeviceCreateConfig};
use crate::device::surface::DeviceSurface;
use crate::instance::init::{create_instance, InstanceCreateConfig};
use crate::vk::objects::surface::SurfaceProvider;

use crate::prelude::*;

pub struct Blaze4D {
    instance: Arc<InstanceContext>,
    device: Arc<DeviceContext>,
}

impl Blaze4D {
    /// Creates a new Blaze4D instance and starts all engine modules.
    ///
    /// The supported vertex formats for the [`EmulatorRenderer`] must be provided here.
    pub fn new(mut main_window: Box<dyn SurfaceProvider>, enable_validation: bool) -> Self {
        log::info!("Creating Blaze4D instance {:?}", BUILD_INFO);

        let mut instance_config = InstanceCreateConfig::new(
            CString::new("Minecraft").unwrap(),
            vk::make_api_version(0, 0, 1, 0)
        );
        if enable_validation {
            instance_config.enable_validation();
        }
        instance_config.add_debug_messenger(Box::new(RustLogDebugMessenger::new()));
        for ext in main_window.get_required_instance_extensions() {
            instance_config.add_required_extension(&ext);
        }

        let instance = create_instance(instance_config).unwrap();

        let window_surface = unsafe { main_window.init(instance.get_entry(), instance.vk()) }.unwrap();

        let mut device_config = DeviceCreateConfig::new();
        device_config.require_swapchain();
        device_config.add_surface(window_surface);
        device_config.disable_robustness();

        let device = create_device(device_config, instance.clone()).unwrap_or_else(|err| {
            log::error!("Failed to create device in Blaze4D::new(): {:?}", err);
            panic!()
        });
        let main_surface = DeviceSurface::new(device.get_functions().clone(), main_window);
        drop(main_surface);

        Self {
            instance,
            device,
        }
    }
}