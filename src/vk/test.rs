use std::ffi::{CString};
use std::sync::Arc;
use ash::{Entry, Instance, vk};
use vk_profiles_rs::vp;
use crate::vk::{DeviceEnvironment, InstanceContext};
use crate::device::init::{create_device, DeviceCreateConfig};
use crate::instance::init::{create_instance, InstanceCreateConfig};
use crate::instance::instance::VulkanVersion;
use crate::vk::objects::surface::{SurfaceInitError, SurfaceProvider};

pub fn make_headless_instance() -> Arc<InstanceContext> {
    let mut config = InstanceCreateConfig::new(
        vp::KhrRoadmap2022::profile_properties(),
        VulkanVersion::VK_1_3,
        CString::new("B4D Tests").unwrap(),
        vk::make_api_version(0, 0, 1, 0)
    );
    config.enable_validation();

    // The LunarG desktop profile requires the swapchain extension which in turn requires the surface extensions
    config.require_surface();

    create_instance(config).unwrap()
}

pub fn make_headless_instance_device() -> (Arc<InstanceContext>, DeviceEnvironment) {
    let instance = make_headless_instance();

    let config = DeviceCreateConfig::new();
    let (device, _) = create_device(config, instance.clone()).unwrap();

    (instance, device)
}

pub struct HeadlessSurfaceProvider {
    surface_khr: Option<ash::extensions::khr::Surface>,
    surface: vk::SurfaceKHR,
}

impl HeadlessSurfaceProvider {
    pub fn new() -> Self {
        Self {
            surface_khr: None,
            surface: vk::SurfaceKHR::null(),
        }
    }
}

impl SurfaceProvider for HeadlessSurfaceProvider {
    fn get_required_instance_extensions(&self) -> Vec<CString> {
        vec![CString::new("VK_EXT_headless_surface").unwrap(), CString::new("VK_KHR_surface").unwrap()]
    }

    fn init(&mut self, entry: &Entry, instance: &Instance) -> Result<vk::SurfaceKHR, SurfaceInitError> {
        self.surface_khr = Some(ash::extensions::khr::Surface::new(entry, instance));

        Err(SurfaceInitError::Generic())
    }

    fn get_handle(&self) -> Option<vk::SurfaceKHR> {
        if self.surface == vk::SurfaceKHR::null() {
            None
        } else {
            Some(self.surface)
        }
    }
}

impl Drop for HeadlessSurfaceProvider {
    fn drop(&mut self) {
        if self.surface != vk::SurfaceKHR::null() {
            unsafe { self.surface_khr.as_ref().unwrap().destroy_surface(self.surface, None) };
            self.surface = vk::SurfaceKHR::null();
        }
    }
}