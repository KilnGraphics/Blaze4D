use std::any::Any;
use ash::Instance;
use crate::init::application_feature::{ApplicationInstanceFeature, InitResult};
use crate::init::instance::{InstanceConfigurator, InstanceInfo};
use crate::init::application_feature::FeatureBase;
use crate::init::initialization_registry::InitializationRegistry;
use crate::init::utils::FeatureAccess;
use crate::NamedUUID;
use crate::rosella::VulkanVersion;

pub fn register_rosella_headless(registry: &mut InitializationRegistry) {
    RosellaInstanceBase::register_into(registry);
    GetPhysicalDeviceProperties2::register_into(registry);
}

pub trait ConstFeature : ApplicationInstanceFeature + Default + 'static {
    const NAME: NamedUUID;
    const DEPENDENCIES: &'static [NamedUUID];

    fn register_into(registry: &mut InitializationRegistry) {
        registry.register_instance_feature(
            Self::NAME,
            Self::DEPENDENCIES.to_vec().into_boxed_slice(),
            Box::new(Self::default())
        )
    }
}

#[derive(Default)]
pub struct RosellaInstanceBase;

impl FeatureBase for RosellaInstanceBase {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_data(&self) -> Box<dyn Any> {
        todo!()
    }
}

impl ApplicationInstanceFeature for RosellaInstanceBase {
    fn init(&mut self, _: &mut dyn FeatureAccess, _: &InstanceInfo) -> InitResult {
        InitResult::Ok
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, _: &InstanceInfo, _: &mut InstanceConfigurator) {
    }

    fn finish(self, _: &Instance) -> Option<Box<dyn Any>> {
        None
    }
}

impl ConstFeature for RosellaInstanceBase {
    const NAME: NamedUUID = NamedUUID::new_const("rosella:rosella_base");
    const DEPENDENCIES: &'static [NamedUUID] = &[];
}

#[derive(Default)]
pub struct GetPhysicalDeviceProperties2;

impl FeatureBase for GetPhysicalDeviceProperties2 {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_data(&self) -> Box<dyn Any> {
        todo!()
    }
}

impl ApplicationInstanceFeature for GetPhysicalDeviceProperties2 {
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

    fn finish(self, _: &Instance) -> Option<Box<dyn Any>> {
        None
    }
}

impl ConstFeature for GetPhysicalDeviceProperties2 {
    const NAME: NamedUUID = NamedUUID::new_const("rosella_vk:get_physical_device_properties_2");
    const DEPENDENCIES: &'static [NamedUUID] = &[];
}

pub struct WindowSurface {
    name: NamedUUID,
    extensions: Vec<std::ffi::CString>,
}

impl WindowSurface {
    pub fn new(window: &winit::window::Window) -> Self {
        let extensions = ash_window::enumerate_required_extensions(window).unwrap();

        Self {
            name: NamedUUID::new_const("rosella:window_surface"),
            extensions: extensions.into_iter().map(|str| std::ffi::CString::from(str)).collect()
        }
    }

    pub fn register_into(registry: &mut InitializationRegistry, window: &winit::window::Window) -> NamedUUID {
        let instance = Box::new(Self::new(window));
        let name = instance.name.clone();

        registry.register_instance_feature(name.clone(), [].to_vec().into_boxed_slice(), instance);

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

    fn get_data(&self) -> Box<dyn Any> {
        todo!()
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