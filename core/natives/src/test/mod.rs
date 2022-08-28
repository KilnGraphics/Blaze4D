//! Contains utilities useful for tests
use std::ffi::CString;
use std::sync::Arc;

use ash::vk;

use crate::BUILD_INFO;
use crate::device::init::{create_device, DeviceCreateConfig, DeviceCreateError};
use crate::instance::debug_messenger::RustLogDebugMessenger;

use crate::instance::init::{create_instance, InstanceCreateConfig, InstanceCreateError};
use crate::vk::objects::surface::{SurfaceInitError, SurfaceProvider};

use crate::prelude::*;

pub mod headless_surface;
pub mod emulator;

pub fn create_test_instance(surface: Option<&dyn SurfaceProvider>) -> Result<Arc<InstanceContext>, InstanceCreateError> {
    let mut config = InstanceCreateConfig::new(
        CString::new("Blaze4D Test").unwrap(),
        vk::make_api_version(0, BUILD_INFO.version_major, BUILD_INFO.version_minor, BUILD_INFO.version_patch)
    );
    config.enable_validation();
    config.add_debug_messenger(Box::new(RustLogDebugMessenger::new()));
    if let Some(surface) = surface {
        for ext in surface.get_required_instance_extensions() {
            config.add_required_extension(&ext);
        }
    }

    create_instance(config)
}

pub fn create_test_device(instance: Arc<InstanceContext>, surface: Option<vk::SurfaceKHR>) -> Result<Arc<DeviceContext>, DeviceCreateError> {
    let mut config = DeviceCreateConfig::new();
    if let Some(surface) = surface {
        config.require_swapchain();
        config.add_surface(surface);
    }
    config.disable_robustness();

    create_device(config, instance)
}

pub enum InstanceDeviceCreateError {
    InstanceCreateError(InstanceCreateError),
    DeviceCreateError(DeviceCreateError),
    SurfaceInitError(SurfaceInitError),
}

impl From<InstanceCreateError> for InstanceDeviceCreateError {
    fn from(err: InstanceCreateError) -> Self {
        Self::InstanceCreateError(err)
    }
}

impl From<DeviceCreateError> for InstanceDeviceCreateError {
    fn from(err: DeviceCreateError) -> Self {
        Self::DeviceCreateError(err)
    }
}

impl From<SurfaceInitError> for InstanceDeviceCreateError {
    fn from(err: SurfaceInitError) -> Self {
        Self::SurfaceInitError(err)
    }
}

pub fn create_test_instance_device(surface: Option<&mut dyn SurfaceProvider>) -> Result<(Arc<InstanceContext>, Arc<DeviceContext>), InstanceDeviceCreateError> {
    let instance = create_test_instance(surface.into())?;

    let surface_instance = if let Some(surface) = surface {
        Some(unsafe { surface.init(instance.get_entry(), instance.vk())? })
    } else {
        None
    };

    let device = create_test_device(instance.clone(), surface_instance).map_err(|err| {
        if let Some(surface) = surface {
            unsafe { surface.destroy() };
        }
        err
    })?;

    Ok((instance, device))
}