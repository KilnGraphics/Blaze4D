use std::ffi::CString;
use ash::vk;
use vk_profiles_rs::vp;
use crate::{DeviceContext, InstanceContext};
use crate::init::device::{create_device, DeviceCreateConfig};
use crate::init::instance::{create_instance, InstanceCreateConfig};
use crate::instance::VulkanVersion;

pub fn make_headless_instance() -> InstanceContext {
    let mut config = InstanceCreateConfig::new(
        vp::LunargDesktopPortability2021::profile_properties(),
        VulkanVersion::VK_1_1,
        CString::new("B4D Tests").unwrap(),
        vk::make_api_version(0, 0, 1, 0)
    );
    config.request_min_api_version(VulkanVersion::VK_1_3);
    config.enable_validation();

    create_instance(config).unwrap()
}

pub fn make_headless_instance_device() -> (InstanceContext, DeviceContext) {
    let instance = make_headless_instance();

    let mut config = DeviceCreateConfig::new();
    let device = create_device(config, instance.clone()).unwrap();

    (instance, device)
}