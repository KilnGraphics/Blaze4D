use std::ffi::{CString};
use std::sync::Arc;

use ash::vk;

use crate::{B4D_CORE_VERSION_MAJOR, B4D_CORE_VERSION_MINOR, B4D_CORE_VERSION_PATCH};

use crate::device::init::{create_device, DeviceCreateConfig};
use crate::instance::init::{create_instance, InstanceCreateConfig};

use crate::prelude::*;

pub fn make_headless_instance() -> Arc<InstanceContext> {
    let mut config = InstanceCreateConfig::new(
        CString::new("B4D Tests").unwrap(),
        vk::make_api_version(0, B4D_CORE_VERSION_MAJOR, B4D_CORE_VERSION_MINOR, B4D_CORE_VERSION_PATCH)
    );
    config.enable_validation();

    // The LunarG desktop profile requires the swapchain extension which in turn requires the surface extensions
    config.require_surface_khr();

    create_instance(config).unwrap()
}

pub fn make_headless_instance_device() -> (Arc<InstanceContext>, Arc<DeviceContext>) {
    let instance = make_headless_instance();

    let mut config = DeviceCreateConfig::new();
    config.disable_robustness(); // We do this in b4d so we should use it for our tests as well
    let device = create_device(config, instance.clone()).unwrap();

    (instance, device)
}