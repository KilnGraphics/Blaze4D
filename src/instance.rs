use std::sync::Arc;

use ash::vk;

use crate::init::EnabledFeatures;
use crate::util::extensions::{AsRefOption, ExtensionFunctionSet, VkExtensionInfo, VkExtensionFunctions};
use crate::UUID;

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
    features: EnabledFeatures,
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
    pub fn new(version: VulkanVersion, entry: ash::Entry, instance: ash::Instance, extensions: ExtensionFunctionSet, features: EnabledFeatures) -> Self {
        Self(Arc::new(InstanceContextImpl{
            version,
            entry,
            instance,
            extensions,
            features,
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

    pub fn get_enabled_features(&self) -> &EnabledFeatures {
        &self.0.features
    }
}