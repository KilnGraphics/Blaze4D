use std::sync::Arc;

use crate::init::device::{create_device, DeviceCreateError};
use crate::init::initialization_registry::InitializationRegistry;
use crate::init::instance::{create_instance, InstanceCreateError};
use crate::window::{RosellaSurface, RosellaWindow};

use ash::vk;
use crate::util::extensions::{AsRefOption, ExtensionFunctionSet, VkExtensionInfo, VkExtensionFunctions};
use crate::init::rosella_features::WindowSurface;
use crate::util::id::UUID;

pub struct Rosella {
    pub instance: InstanceContext,
    pub surface: RosellaSurface,
    pub device: DeviceContext,
}

#[derive(Debug)]
pub enum RosellaCreateError {
    InstanceCreateError(InstanceCreateError),
    DeviceCreateError(DeviceCreateError),
}

impl From<InstanceCreateError> for RosellaCreateError {
    fn from(err: InstanceCreateError) -> Self {
        RosellaCreateError::InstanceCreateError(err)
    }
}

impl From<DeviceCreateError> for RosellaCreateError {
    fn from(err: DeviceCreateError) -> Self {
        RosellaCreateError::DeviceCreateError(err)
    }
}

impl Rosella {
    pub fn new(mut registry: InitializationRegistry, window: &RosellaWindow, application_name: &str) -> Result<Rosella, RosellaCreateError> {
        log::info!("Starting Rosella");

        WindowSurface::register_into(&mut registry, &window.handle, true);

        let now = std::time::Instant::now();

        let ash_entry = unsafe{ ash::Entry::new() }.unwrap();
        let instance = create_instance(&mut registry, application_name, 0)?;

        let surface = RosellaSurface::new(instance.vk(), &ash_entry, window);

        let device = create_device(&mut registry, instance.clone())?;

        let elapsed = now.elapsed();
        println!("Instance & Device Initialization took: {:.2?}", elapsed);

        Ok(Rosella {
            instance: instance.clone(),
            surface,
            device: device.clone()
        })
    }

    pub fn window_update(&self) {}

    pub fn recreate_swapchain(&self, width: u32, height: u32) {
        println!("resize to {}x{}", width, height);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VulkanVersion(u32);

impl VulkanVersion {
    pub const VK_1_0: VulkanVersion = VulkanVersion(vk::API_VERSION_1_0);
    pub const VK_1_1: VulkanVersion = VulkanVersion(vk::API_VERSION_1_1);
    pub const VK_1_2: VulkanVersion = VulkanVersion(vk::API_VERSION_1_2);

    pub const fn from_raw(value: u32) -> Self {
        Self(value)
    }

    pub fn new(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
        Self(vk::make_api_version(variant, major, minor, patch))
    }

    pub fn is_supported(&self, version: VulkanVersion) -> bool {
        vk::api_version_major(self.0) >= vk::api_version_major(version.0)
    }
}

struct InstanceContextImpl {
    version: VulkanVersion,
    entry: ash::Entry,
    instance: ash::Instance,
    extensions: ExtensionFunctionSet,
}

impl Drop for InstanceContextImpl {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

#[derive(Clone)]
pub struct InstanceContext(Arc<InstanceContextImpl>);

impl InstanceContext {
    pub fn new(version: VulkanVersion, entry: ash::Entry, instance: ash::Instance, extensions: ExtensionFunctionSet) -> Self {
        Self(Arc::new(InstanceContextImpl{
            version,
            entry,
            instance,
            extensions,
        }))
    }

    pub fn get_entry(&self) -> &ash::Entry {
        &self.0.entry
    }

    pub fn vk(&self) -> &ash::Instance {
        &self.0.instance
    }

    pub fn get_version(&self) -> VulkanVersion {
        self.0.version
    }

    pub fn get_extension<T: VkExtensionInfo>(&self) -> Option<&T> where VkExtensionFunctions: AsRefOption<T> {
        self.0.extensions.get()
    }

    pub fn is_extension_enabled(&self, uuid: UUID) -> bool {
        self.0.extensions.contains(uuid)
    }
}

pub struct DeviceContextImpl {
    instance: InstanceContext,
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
    extensions: ExtensionFunctionSet,
}

impl Drop for DeviceContextImpl {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}

#[derive(Clone)]
pub struct DeviceContext(Arc<DeviceContextImpl>);

impl DeviceContext {
    pub fn new(instance: InstanceContext, device: ash::Device, physical_device: vk::PhysicalDevice, extensions: ExtensionFunctionSet) -> Self {
        Self(Arc::new(DeviceContextImpl{
            instance,
            device,
            physical_device,
            extensions,
        }))
    }

    pub fn get_entry(&self) -> &ash::Entry {
        self.0.instance.get_entry()
    }

    pub fn get_instance(&self) -> &InstanceContext {
        &self.0.instance
    }

    pub fn vk(&self) -> &ash::Device {
        &self.0.device
    }

    pub fn get_physical_device(&self) -> &vk::PhysicalDevice {
        &self.0.physical_device
    }

    pub fn get_extension<T: VkExtensionInfo>(&self) -> Option<&T> where VkExtensionFunctions: AsRefOption<T> {
        self.0.extensions.get()
    }

    pub fn is_extension_enabled(&self, uuid: UUID) -> bool {
        self.0.extensions.contains(uuid)
    }
}
