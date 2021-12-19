use std::any::Any;
use std::ffi::{c_void, CStr};
use ash::{Instance, vk};
use paste::paste;
use crate::init::application_feature::{ApplicationDeviceFeatureGenerator, ApplicationDeviceFeature, ApplicationInstanceFeature, InitResult};
use crate::init::instance::{InstanceConfigurator, InstanceInfo};
use crate::init::application_feature::FeatureBase;
use crate::init::device::{DeviceConfigurator, DeviceInfo};
use crate::init::initialization_registry::InitializationRegistry;
use crate::init::utils::FeatureAccess;
use crate::NamedUUID;
use crate::rosella::VulkanVersion;

pub fn register_rosella_headless(registry: &mut InitializationRegistry) {
    KHRGetPhysicalDeviceProperties2::register_into(registry, false);
    KHRTimelineSemaphore::register_into(registry, false);
    RosellaInstanceBase::register_into(registry, true);

    RosellaDeviceBase::register_into(registry, true);
}

pub fn register_rosella_debug(registry: &mut InitializationRegistry, required: bool) {
    RosellaDebug::register_into(registry, required);
}

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

#[derive(Default)]
pub struct RosellaInstanceBase;
const_instance_feature!(RosellaInstanceBase, "rosella:instance_base", [KHRTimelineSemaphore::NAME]);

impl ApplicationInstanceFeature for RosellaInstanceBase {
    fn init(&mut self, features: &mut dyn FeatureAccess, _: &InstanceInfo) -> InitResult {
        if !features.is_supported(&KHRTimelineSemaphore::NAME.get_uuid()) {
            log::warn!("KHRTimelineSemaphore is not supported");
            return InitResult::Disable;
        }

        InitResult::Ok
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, _: &InstanceInfo, _: &mut InstanceConfigurator) {
    }
}

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

#[derive(Default)]
pub struct KHRTimelineSemaphore;
const_instance_feature!(KHRTimelineSemaphore, "rosella:instance_khr_timeline_semaphore", [KHRGetPhysicalDeviceProperties2::NAME]);

impl ApplicationInstanceFeature for KHRTimelineSemaphore {
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
            config.enable_extension::<ash::extensions::khr::TimelineSemaphore>();
        }
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

    fn finish(self, _: &Instance) -> Option<Box<dyn Any>> {
        None
    }
}

#[derive(Default)]
struct RosellaDeviceBase;
const_device_feature!(RosellaDeviceBase, "rosella:device_base", []);

impl ApplicationDeviceFeature for RosellaDeviceBase {
    fn init(&mut self, _: &mut dyn FeatureAccess, _: &DeviceInfo) -> InitResult {
        InitResult::Ok
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, _: &DeviceInfo, config: &mut DeviceConfigurator) {
        config.add_queue_request(0); // TODO This is just to prevent validation errors
    }
}