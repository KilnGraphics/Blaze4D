use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Pointer};
use std::sync::{Arc, Mutex};

use ash::vk;

use crate::init::EnabledFeatures;
use crate::instance::InstanceContext;
use crate::objects::id::SurfaceId;
use crate::objects::surface::{Surface, SurfaceCapabilities};
use crate::util::extensions::{AsRefOption, ExtensionFunctionSet, VkExtensionInfo, VkExtensionFunctions};
use crate::{NamedUUID, UUID};

pub enum SurfaceAttachError {
    SurfaceAlreadyPresent,
    DeviceUnsupported,
}

struct DeviceContextImpl {
    id: NamedUUID,
    instance: InstanceContext,
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
    extensions: ExtensionFunctionSet,
    features: EnabledFeatures,
    surfaces: Mutex<HashMap<SurfaceId, (Surface, SurfaceCapabilities)>>,
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
    pub fn new(instance: InstanceContext, device: ash::Device, physical_device: vk::PhysicalDevice, extensions: ExtensionFunctionSet, features: EnabledFeatures) -> Self {
        Self(Arc::new(DeviceContextImpl{
            id: NamedUUID::with_str("Device"),
            instance,
            device,
            physical_device,
            extensions,
            features,
            surfaces: Mutex::new(HashMap::new()),
        }))
    }

    pub fn get_uuid(&self) -> &NamedUUID {
        &self.0.id
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

    pub fn get_enabled_features(&self) -> &EnabledFeatures {
        &self.0.features
    }

    pub fn attach_surface(&self, surface: Surface) -> Result<SurfaceId, SurfaceAttachError> {
        let id = surface.get_id();


        let capabilities = SurfaceCapabilities::new(self.get_instance(), *self.get_physical_device(), surface.get_handle());
        if capabilities.is_none() {
            return Err(SurfaceAttachError::DeviceUnsupported);
        }
        let capabilities = capabilities.unwrap();

        let mut map = self.0.surfaces.lock().unwrap();

        if map.contains_key(&id) {
            return Err(SurfaceAttachError::SurfaceAlreadyPresent);
        }

        map.insert(id, (surface, capabilities));

        Ok(id)
    }

    pub fn get_surface(&self, id: SurfaceId) -> Option<Surface> {
        self.0.surfaces.lock().unwrap().get(&id).map(|data| data.0.clone())
    }

    pub fn get_surface_capabilities(&self, id: SurfaceId) -> Option<&SurfaceCapabilities> {
        None
    }
}

impl PartialEq for DeviceContext {
    fn eq(&self, other: &Self) -> bool {
        self.0.id.eq(&other.0.id)
    }
}

impl Eq for DeviceContext {
}

impl PartialOrd for DeviceContext {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.id.partial_cmp(&other.0.id)
    }
}

impl Ord for DeviceContext {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.id.cmp(&other.0.id)
    }
}

impl Debug for DeviceContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}