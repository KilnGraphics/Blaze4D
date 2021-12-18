use std::collections::{HashMap, HashSet};
use std::ffi::CString;

use crate::{ALLOCATION_CALLBACKS, NamedUUID};
use ash::extensions::ext::DebugUtils;
use ash::vk::{make_api_version, ApplicationInfo, InstanceCreateInfo};
use ash::{Entry, Instance, InstanceError};
use crate::init::application_feature::ApplicationInstanceFeature;

use crate::init::initialization_registry::InitializationRegistry;
use crate::init::utils::{ExtensionProperties, FeatureSet, LayerProperties};
use crate::window::RosellaWindow;

use ash::vk;
use crate::rosella::VulkanVersion;
use crate::util::id::UUID;

pub fn create_instance(
    registry: &InitializationRegistry,
    application_name: &str,
    application_version: u32,
    window: &RosellaWindow,
    entry: &ash::Entry,
) -> Instance {
    let supported_version = entry
        .try_enumerate_instance_version()
        .ok()
        .flatten()
        .expect("Failed to enumerate over instance versions");
    assert!(
        supported_version >= registry.min_required_version,
        "minimum vulkan version is not supported"
    );

    let app_name = CString::new(application_name).unwrap();
    let app_info = ApplicationInfo::builder()
        .application_name(app_name.as_c_str())
        .application_version(application_version)
        .engine_name(CString::new("Rosella").unwrap().as_c_str())
        .engine_version(make_api_version(2, 0, 0, 0))
        .api_version(registry.max_supported_version)
        .build();

    // FIXME: do this properly. Just get it to compile for now
    let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
    let layers_names_raw: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();

    let surface_extensions = ash_window::enumerate_required_extensions(&window.handle).unwrap();
    let mut extension_names_raw = surface_extensions.iter().map(|ext| ext.as_ptr()).collect::<Vec<_>>();
    extension_names_raw.push(DebugUtils::name().as_ptr());

    let create_info = InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&extension_names_raw)
        .build();
    // .push_next(createDebugUtilsCallback(VK10.VK_NULL_HANDLE));

    unsafe { entry.create_instance(&create_info, ALLOCATION_CALLBACKS) }.expect("Failed to create a Vulkan Instance.")
}


pub enum InstanceCreateError {
    VulkanError(vk::Result),
    AshInstanceError(ash::InstanceError),
    Utf8Error(std::str::Utf8Error),
    LayerNotSupported,
    ExtensionNotSupported,
}

impl From<vk::Result> for InstanceCreateError {
    fn from(err: vk::Result) -> Self {
        InstanceCreateError::VulkanError(err)
    }
}

impl From<ash::InstanceError> for InstanceCreateError {
    fn from(err: ash::InstanceError) -> Self {
        InstanceCreateError::AshInstanceError(err)
    }
}

impl From<std::str::Utf8Error> for InstanceCreateError {
    fn from(err: std::str::Utf8Error) -> Self {
        InstanceCreateError::Utf8Error(err)
    }
}

pub type InstanceFeatureSet = FeatureSet<dyn ApplicationInstanceFeature>;

pub struct InstanceInfo {
    entry: ash::Entry,
    version: VulkanVersion,
    layers: HashMap<UUID, LayerProperties>,
    extensions: HashMap<UUID, ExtensionProperties>,
}

impl InstanceInfo {
    fn new(entry: ash::Entry) -> Result<Self, InstanceCreateError> {
        let version = match entry.try_enumerate_instance_version()? {
            None => VulkanVersion::VK_1_0,
            Some(version) => VulkanVersion::from_raw(version),
        };

        let layers_raw = entry.enumerate_instance_layer_properties()?;
        let mut layers = HashMap::new();
        for layer in layers_raw {
            let layer = LayerProperties::new(&layer)?;
            let uuid = NamedUUID::uuid_for(layer.get_name());

            layers.insert(uuid, layer);
        }

        let extensions_raw = entry.enumerate_instance_extension_properties()?;
        let mut extensions = HashMap::new();
        for extension in extensions_raw {
            let extension = ExtensionProperties::new(&extension)?;
            let uuid = NamedUUID::uuid_for(extension.get_name());

            extensions.insert(uuid, extension);
        }

        Ok(Self{
            entry,
            version,
            layers,
            extensions,
        })
    }

    pub fn get_entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn get_vulkan_version(&self) -> VulkanVersion {
        self.version
    }

    pub fn is_layer_supported(&self, name: &String) -> bool {
        let uuid = NamedUUID::uuid_for(name);
        self.layers.contains_key(&uuid)
    }

    pub fn is_layer_supported_uuid(&self, uuid: &UUID) -> bool {
        self.layers.contains_key(uuid)
    }

    pub fn get_layer_properties(&self, name: &String) -> Option<&LayerProperties> {
        let uuid = NamedUUID::uuid_for(name);
        self.layers.get(&uuid)
    }

    pub fn get_layer_properties_uuid(&self, uuid: &UUID) -> Option<&LayerProperties> {
        self.layers.get(uuid)
    }

    pub fn is_extension_supported(&self, name: &String) -> bool {
        let uuid = NamedUUID::uuid_for(name);
        self.extensions.contains_key(&uuid)
    }

    pub fn is_extension_supported_uuid(&self, uuid: &UUID) -> bool {
        self.extensions.contains_key(uuid)
    }

    pub fn get_extension_properties(&self, name: &String) -> Option<&ExtensionProperties> {
        let uuid = NamedUUID::uuid_for(name);
        self.extensions.get(&uuid)
    }

    pub fn get_extension_properties_uuid(&self, uuid: &UUID) -> Option<&ExtensionProperties> {
        self.extensions.get(uuid)
    }
}

pub struct InstanceConfigurator {
    enabled_layers: HashSet<UUID>,
    enabled_extensions: HashSet<UUID>,
}

impl InstanceConfigurator {
    fn new() -> Self {
        Self{
            enabled_layers: HashSet::new(),
            enabled_extensions: HashSet::new(),
        }
    }

    pub fn enable_layer(&mut self, name: &String) {
        let uuid = NamedUUID::uuid_for(name);
        self.enabled_layers.insert(uuid);
    }

    pub fn enable_layer_uuid(&mut self, uuid: UUID) {
        self.enabled_layers.insert(uuid);
    }

    pub fn enable_extension(&mut self, name: &String) {
        let uuid = NamedUUID::uuid_for(name);
        self.enabled_extensions.insert(uuid);
    }

    pub fn enable_extension_uuid(&mut self, uuid: UUID) {
        self.enabled_extensions.insert(uuid);
    }

    fn build_instance(self, info: &InstanceInfo, application_info: &vk::ApplicationInfo) -> Result<ash::Instance, InstanceCreateError> {
        let mut layers = Vec::with_capacity(self.enabled_layers.len());
        for layer in &self.enabled_layers {
            layers.push(
                info.get_layer_properties_uuid(layer)
                    .ok_or(InstanceCreateError::LayerNotSupported)?
                    .get_c_name().as_ptr()
            );
        }

        let mut extensions = Vec::with_capacity(self.enabled_extensions.len());
        for extension in &self.enabled_extensions {
            extensions.push(
                info.get_extension_properties_uuid(extension)
                    .ok_or(InstanceCreateError::ExtensionNotSupported)?
                    .get_c_name().as_ptr()
            )
        }

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(application_info)
            .enabled_layer_names(layers.as_slice())
            .enabled_extension_names(extensions.as_slice());

        unsafe {
            info.get_entry().create_instance(&create_info.build(), None)
        }.map_err(|err| InstanceCreateError::AshInstanceError(err))
    }
}