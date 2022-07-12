use core::panic::{UnwindSafe, RefUnwindSafe};

use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use ash::vk;
use vk_profiles_rs::vp;

use crate::instance::init::DebugUtilsMessengerWrapper;

use crate::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct VulkanVersion(u32);

impl VulkanVersion {
    pub const VK_1_0: VulkanVersion = VulkanVersion(vk::API_VERSION_1_0);
    pub const VK_1_1: VulkanVersion = VulkanVersion(vk::API_VERSION_1_1);
    pub const VK_1_2: VulkanVersion = VulkanVersion(vk::API_VERSION_1_2);
    pub const VK_1_3: VulkanVersion = VulkanVersion(vk::API_VERSION_1_3);

    pub const fn from_raw(value: u32) -> Self {
        Self(value)
    }

    pub fn new(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
        Self(vk::make_api_version(variant, major, minor, patch))
    }

    pub const fn get_major(&self) -> u32 {
        vk::api_version_major(self.0)
    }

    pub const fn get_minor(&self) -> u32 {
        vk::api_version_minor(self.0)
    }

    pub const fn get_patch(&self) -> u32 {
        vk::api_version_patch(self.0)
    }

    pub const fn get_raw(&self) -> u32 {
        self.0
    }
}

impl From<VulkanVersion> for u32 {
    fn from(version: VulkanVersion) -> Self {
        version.0
    }
}

impl Debug for VulkanVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("VulkanVersion([{}] {}.{}.{})", vk::api_version_variant(self.0), vk::api_version_major(self.0), vk::api_version_minor(self.0), vk::api_version_patch(self.0)))
    }
}

/// Implementation of the instance context.
///
/// Since we need to control drop order most of the fields are ManuallyDrop
pub struct InstanceContext {
    id: NamedUUID,
    version: VulkanVersion,
    profile: vp::ProfileProperties,
    entry: ash::Entry,
    instance: ash::Instance,
    surface_khr: Option<ash::extensions::khr::Surface>,
    _debug_messengers: Box<[DebugUtilsMessengerWrapper]>,
}

impl InstanceContext {
    pub fn new(
        version: VulkanVersion,
        profile: vp::ProfileProperties,
        entry: ash::Entry,
        instance: ash::Instance,
        surface_khr: Option<ash::extensions::khr::Surface>,
        debug_messengers: Box<[DebugUtilsMessengerWrapper]>
    ) -> Arc<Self> {
        Arc::new(Self {
            id: NamedUUID::with_str("Instance"),
            version,
            profile,
            entry,
            instance,
            surface_khr,
            _debug_messengers: debug_messengers,
        })
    }

    pub fn get_uuid(&self) -> &NamedUUID {
        &self.id
    }

    pub fn get_entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn vk(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn surface_khr(&self) -> Option<&ash::extensions::khr::Surface> {
        self.surface_khr.as_ref()
    }

    pub fn get_version(&self) -> VulkanVersion {
        self.version
    }

    pub fn get_profile(&self) -> &vp::ProfileProperties {
        &self.profile
    }
}

impl Drop for InstanceContext {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

impl PartialEq for InstanceContext {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for InstanceContext {
}

impl PartialOrd for InstanceContext {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for InstanceContext {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Debug for InstanceContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

assert_impl_all!(InstanceContext: Send, Sync, UnwindSafe, RefUnwindSafe);