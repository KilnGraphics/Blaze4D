use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use ash::vk;
use vk_profiles_rs::vp;

use crate::NamedUUID;
use crate::objects::id::{ObjectSetId, SurfaceId};
use crate::objects::surface::SurfaceProvider;

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

    pub fn is_supported(&self, version: VulkanVersion) -> bool {
        vk::api_version_major(self.0) >= vk::api_version_major(version.0)
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
pub struct InstanceContextImpl {
    id: NamedUUID,
    version: VulkanVersion,
    profile: vp::ProfileProperties,
    entry: ash::Entry,
    instance: ash::Instance,
    surface_khr: Option<ash::extensions::khr::Surface>,
    surfaces: ManuallyDrop<Mutex<HashMap<SurfaceId, Box<dyn SurfaceProvider>>>>,
    _debug_messengers: ManuallyDrop<Box<[crate::init::instance::DebugUtilsMessengerWrapper]>>,
}

impl InstanceContextImpl {
    pub fn new(
        version: VulkanVersion,
        profile: vp::ProfileProperties,
        entry: ash::Entry,
        instance: ash::Instance,
        surface_khr: Option<ash::extensions::khr::Surface>,
        surfaces: HashMap<SurfaceId, Box<dyn SurfaceProvider>>,
        debug_messengers: Box<[crate::init::instance::DebugUtilsMessengerWrapper]>
    ) -> Self {
        Self {
            id: NamedUUID::with_str("Instance"),
            version,
            profile,
            entry,
            instance,
            surface_khr,
            surfaces: ManuallyDrop::new(Mutex::new(surfaces)),
            _debug_messengers: ManuallyDrop::new(debug_messengers),
        }
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

    pub(crate) fn take_surface(&self, surface: SurfaceId) -> Option<Box<dyn SurfaceProvider>> {
        let mut surfaces = self.surfaces.lock().unwrap();
        surfaces.remove(&surface)
    }
}

impl Drop for InstanceContextImpl {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.surfaces);

            self.instance.destroy_instance(None);

            ManuallyDrop::drop(&mut self._debug_messengers);
        }
    }
}

impl PartialEq for InstanceContextImpl {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for InstanceContextImpl {
}

impl PartialOrd for InstanceContextImpl {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for InstanceContextImpl {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Debug for InstanceContextImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

pub type InstanceContext = Arc<InstanceContextImpl>;