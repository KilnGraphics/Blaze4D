//! Common vulkan and rosella instance and device

use std::any::Any;
use std::ffi::{c_void, CStr};
use ash::vk;
use paste::paste;
use crate::init::application_feature::{ApplicationDeviceFeatureGenerator, ApplicationDeviceFeature, ApplicationInstanceFeature, InitResult};
use crate::init::instance::{InstanceConfigurator, InstanceInfo};
use crate::init::application_feature::FeatureBase;
use crate::init::device::{DeviceConfigurator, DeviceInfo};
use crate::init::initialization_registry::InitializationRegistry;
use crate::init::application_feature::FeatureAccess;
use crate::NamedUUID;
use crate::rosella::VulkanVersion;

/// Registers all instance and device features required for rosella to work in headless mode
pub fn register_rosella_headless(registry: &mut InitializationRegistry) {
    KHRGetPhysicalDeviceProperties2::register_into(registry, false);
    KHRTimelineSemaphoreInstance::register_into(registry, false);
    RosellaInstanceBase::register_into(registry, true);

    KHRTimelineSemaphoreDevice::register_into(registry, false);
    RosellaDeviceBase::register_into(registry, true);
}

/// Registers all instance and device features required for rosella to present images
pub fn register_rosella_present(registry: &mut InitializationRegistry) {
    KHRSurface::register_into(registry, true);
    KHRSwapchain::register_into(registry, true);
}

/// Registers instance and device features that provide debugging capabilities
pub fn register_rosella_debug(registry: &mut InitializationRegistry, required: bool) {
    RosellaDebug::register_into(registry, required);
}

/// Utility macro that generates common implementations for instance features which can be default
/// created.
#[macro_export]
macro_rules! const_instance_feature{
    ($struct_name:ty, $name:literal, [$($dependency:expr),*]) => {
        impl $struct_name {
            const NAME: NamedUUID = NamedUUID::new_const($name);
            const DEPENDENCIES: &'static [NamedUUID] = &[$($dependency,)*];

            fn register_into(registry: &mut InitializationRegistry, required: bool) {
                registry.register_instance_feature(
                    Self::NAME,
                    Self::DEPENDENCIES.to_vec().into_boxed_slice(),
                    Box::new(Self::default()),
                    required
                )
            }
        }

        impl FeatureBase for $struct_name {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }
    }
}

/// Utility macro that generates common implementations for device features which can be default
/// created.
#[macro_export]
macro_rules! const_device_feature{
    ($struct_name:ident, $name:literal, [$($dependency:expr),*]) => {
        paste! {
            #[derive(Default)]
            pub struct [<$struct_name Generator>];

            impl ApplicationDeviceFeatureGenerator for [<$struct_name Generator>] {
                fn make_instance(&self) -> Box<dyn ApplicationDeviceFeature> {
                    Box::new($struct_name::default())
                }
            }
        }

        impl $struct_name {
            const NAME: NamedUUID = NamedUUID::new_const($name);
            const DEPENDENCIES: &'static [NamedUUID] = &[$($dependency,)*];

            fn register_into(registry: &mut InitializationRegistry, required: bool) {
                registry.register_device_feature(
                    Self::NAME,
                    Self::DEPENDENCIES.to_vec().into_boxed_slice(),
                    paste! { Box::new([<$struct_name Generator>]::default()) },
                    required
                )
            }
        }

        impl FeatureBase for $struct_name {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }
    }
}

/// Instance feature which provides all requirements needed for rosella to function in headless
#[derive(Default)]
pub struct RosellaInstanceBase;
const_instance_feature!(RosellaInstanceBase, "rosella:instance_base", [KHRTimelineSemaphoreInstance::NAME]);

impl ApplicationInstanceFeature for RosellaInstanceBase {
    fn init(&mut self, features: &mut dyn FeatureAccess, _: &InstanceInfo) -> InitResult {
        if !features.is_supported(&KHRTimelineSemaphoreInstance::NAME.get_uuid()) {
            log::warn!("KHRTimelineSemaphore is not supported");
            return InitResult::Disable;
        }

        InitResult::Ok
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, _: &InstanceInfo, _: &mut InstanceConfigurator) {
    }
}

/// Instance feature which loads validation layers and provides debug callback logging
#[derive(Default)]
pub struct RosellaDebug;
const_instance_feature!(RosellaDebug, "rosella:instance_debug", []);

impl RosellaDebug {
    extern "system" fn debug_callback(severity: vk::DebugUtilsMessageSeverityFlagsEXT, _: vk::DebugUtilsMessageTypeFlagsEXT, data:*const vk::DebugUtilsMessengerCallbackDataEXT, _:*mut c_void) -> vk::Bool32 {
        let data = unsafe { data.as_ref().unwrap() };

        let id = match unsafe { CStr::from_ptr(data.p_message_id_name) }.to_str() {
            Ok(str) => str,
            Err(err) => {
                log::error!("Failed to process debug callback id: {:?}", err);
                return vk::FALSE;
            }
        };

        let msg = match unsafe { CStr::from_ptr(data.p_message) }.to_str() {
            Ok(str) => str,
            Err(err) => {
                log::error!("Failed to process debug callback message: {:?}", err);
                return vk::FALSE;
            }
        };

        if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
            log::error!(target: "vulkan", "{}: {}", id, msg);
        } else if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) {
            log::warn!(target: "vulkan", "{}: {}", id, msg);
        } else if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::INFO) {
            log::info!(target: "vulkan", "{}: {}", id, msg);
        } else {
            log::debug!(target: "vulkan", "{}: {}", id, msg);
        }

        vk::FALSE
    }
}

impl ApplicationInstanceFeature for RosellaDebug {
    fn init(&mut self, _: &mut dyn FeatureAccess, info: &InstanceInfo) -> InitResult {
        if !info.is_extension_supported::<ash::extensions::ext::DebugUtils>() {
            log::warn!("DebugUtils extension not found! Rosella debug will be disabled.");
            return InitResult::Disable;
        }

        if !info.is_layer_supported_str("VK_LAYER_KHRONOS_validation") {
            log::warn!("VK_LAYER_KHRONOS_validation not found! Rosella debug will be disabled.");
            return InitResult::Disable;
        }

        InitResult::Ok
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, _: &InstanceInfo, config: &mut InstanceConfigurator) {
        config.enable_extension::<ash::extensions::ext::DebugUtils>();
        config.enable_layer("VK_LAYER_KHRONOS_validation");
        config.set_debug_messenger(Some(RosellaDebug::debug_callback));
    }
}

/// Instance feature representing the VK_KHR_get_physical_device_properties2 feature set.
/// If the instance version is below 1.1 it will load the extension.
#[derive(Default)]
pub struct KHRGetPhysicalDeviceProperties2;
const_instance_feature!(KHRGetPhysicalDeviceProperties2, "rosella:instance_khr_get_physical_device_properties_2", []);

impl ApplicationInstanceFeature for KHRGetPhysicalDeviceProperties2 {
    fn init(&mut self, _: &mut dyn FeatureAccess, info: &InstanceInfo) -> InitResult {
        if info.get_vulkan_version().is_supported(VulkanVersion::VK_1_1) {
            InitResult::Ok
        } else {
            if info.is_extension_supported::<ash::extensions::khr::GetPhysicalDeviceProperties2>() {
                InitResult::Ok
            } else {
                InitResult::Disable
            }
        }
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, info: &InstanceInfo, config: &mut InstanceConfigurator) {
        if !info.get_vulkan_version().is_supported(VulkanVersion::VK_1_1) {
            config.enable_extension::<ash::extensions::khr::GetPhysicalDeviceProperties2>();
        }
    }
}

/// Instance feature representing the VK_KHR_surface extension.
#[derive(Default)]
pub struct KHRSurface;
const_instance_feature!(KHRSurface, "rosella:instance_khr_surface", []);

impl ApplicationInstanceFeature for KHRSurface {
    fn init(&mut self, _: &mut dyn FeatureAccess, info: &InstanceInfo) -> InitResult {
        if !info.is_extension_supported::<ash::extensions::khr::Surface>() {
            return InitResult::Disable;
        }

        return InitResult::Ok;
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, _: &InstanceInfo, config: &mut InstanceConfigurator) {
        config.enable_extension::<ash::extensions::khr::Surface>();
    }
}

/// Instance feature representing the VK_KHR_timeline_semaphore feature set.
/// If the instance version is below 1.2 it will load the extension.
#[derive(Default)]
pub struct KHRTimelineSemaphoreInstance;
const_instance_feature!(KHRTimelineSemaphoreInstance, "rosella:instance_khr_timeline_semaphore", [KHRGetPhysicalDeviceProperties2::NAME]);

impl ApplicationInstanceFeature for KHRTimelineSemaphoreInstance {
    fn init(&mut self, features: &mut dyn FeatureAccess, info: &InstanceInfo) -> InitResult {
        if !features.is_supported(&KHRGetPhysicalDeviceProperties2::NAME.get_uuid()) {
            log::warn!("KHRGetPhysicalDeviceProperties2 is not supported");
            return InitResult::Disable;
        }

        let core_present = info.get_vulkan_version().is_supported(VulkanVersion::VK_1_2);
        if !core_present {
            if !info.is_extension_supported::<ash::extensions::khr::TimelineSemaphore>() {
                return InitResult::Disable;
            }
        }

        InitResult::Ok
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, info: &InstanceInfo, config: &mut InstanceConfigurator) {
        if !info.get_vulkan_version().is_supported(VulkanVersion::VK_1_2) {
            config.enable_extension_no_load::<ash::extensions::khr::TimelineSemaphore>();
        }
    }
}

/// Device feature representing the VK_KHR_timeline_semaphore feature set.
#[derive(Default)]
pub struct KHRTimelineSemaphoreDevice;
const_device_feature!(KHRTimelineSemaphoreDevice, "rosella:device_khr_timeline_semaphore", []);

impl ApplicationDeviceFeature for KHRTimelineSemaphoreDevice {
    fn init(&mut self, _: &mut dyn FeatureAccess, info: &DeviceInfo) -> InitResult {
        if info.get_instance().get_version().is_supported(VulkanVersion::VK_1_2) {
            if info.get_device_1_2_features().unwrap().timeline_semaphore == vk::TRUE {
                InitResult::Ok
            } else {
                log::warn!("Vulkan 1.2 is supported but timeline semaphores are not supported");
                InitResult::Disable
            }
        } else {
            // Feature is provided by extension
            match info.get_timeline_semaphore_features() {
                None => InitResult::Disable,
                Some(features) => {
                    if features.timeline_semaphore == vk::TRUE {
                        InitResult::Ok
                    } else {
                        log::warn!("VK_KHR_Timeline_Semaphore is enabled but timeline semaphores are not supported");
                        InitResult::Disable
                    }
                }
            }
        }
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, info: &DeviceInfo, config: &mut DeviceConfigurator) {
        if !info.get_instance().get_version().is_supported(VulkanVersion::VK_1_2) {
            config.enable_extension::<ash::extensions::khr::TimelineSemaphore>();
        }
        config.enable_timeline_semaphore()
    }
}

pub struct WindowSurface {
    name: NamedUUID,
    extensions: Vec<std::ffi::CString>,
}

impl WindowSurface {
    pub fn new(window: &winit::window::Window) -> Self {
        let extensions = ash_window::enumerate_required_extensions(window).unwrap();

        Self {
            name: NamedUUID::new_const("rosella:instance_window_surface"),
            extensions: extensions.into_iter().map(|str| std::ffi::CString::from(str)).collect()
        }
    }

    pub fn register_into(registry: &mut InitializationRegistry, window: &winit::window::Window, required: bool) -> NamedUUID {
        let instance = Box::new(Self::new(window));
        let name = instance.name.clone();

        registry.register_instance_feature(name.clone(), [].to_vec().into_boxed_slice(), instance, required);

        name
    }
}

impl FeatureBase for WindowSurface {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ApplicationInstanceFeature for WindowSurface {
    fn init(&mut self, _: &mut dyn FeatureAccess, info: &InstanceInfo) -> InitResult {
        for extension in &self.extensions {
            if !info.is_extension_supported_str(extension.to_str().unwrap()) {
                return InitResult::Disable
            }
        }
        InitResult::Ok
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, _: &InstanceInfo, config: &mut InstanceConfigurator) {
        for extension in &self.extensions {
            config.enable_extension_str_no_load(extension.to_str().unwrap())
        }
    }
}

/// Device feature which provides all requirements needed for rosella to function in headless
#[derive(Default)]
pub struct RosellaDeviceBase;
const_device_feature!(RosellaDeviceBase, "rosella:device_base", [KHRTimelineSemaphoreDevice::NAME]);

impl ApplicationDeviceFeature for RosellaDeviceBase {
    fn init(&mut self, features: &mut dyn FeatureAccess, _: &DeviceInfo) -> InitResult {
        if !features.is_supported(&KHRTimelineSemaphoreDevice::NAME.get_uuid()) {
            return InitResult::Disable;
        }

        InitResult::Ok
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, _: &DeviceInfo, config: &mut DeviceConfigurator) {
        config.add_queue_request(0); // TODO This is just to prevent validation errors
    }
}

/// Device feature representing the VK_KHR_swapchain extension.
#[derive(Default)]
pub struct KHRSwapchain;
const_device_feature!(KHRSwapchain, "rosella:device_khr_swapchain", []);

impl ApplicationDeviceFeature for KHRSwapchain {
    fn init(&mut self, _: &mut dyn FeatureAccess, info: &DeviceInfo) -> InitResult {
        if !info.is_extension_supported::<ash::extensions::khr::Swapchain>() {
            return InitResult::Disable;
        }

        InitResult::Ok
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, _: &DeviceInfo, config: &mut DeviceConfigurator) {
        config.enable_extension::<ash::extensions::khr::Swapchain>()
    }
}
