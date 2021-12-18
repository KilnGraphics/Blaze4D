use std::collections::{HashMap, HashSet};
use std::ffi::CString;

use crate::{ALLOCATION_CALLBACKS, NamedUUID};
use ash::extensions::ext::DebugUtils;
use ash::vk::{make_api_version, InstanceCreateInfo};
use ash::{Entry, Instance, InstanceError};
use crate::init::application_feature::{ApplicationInstanceFeature, InitResult};

use crate::init::initialization_registry::InitializationRegistry;
use crate::init::utils::{ConditionResult, ExtensionProperties, FeatureProcessor, FeatureSet, FeatureSet2, LayerProperties};
use crate::window::RosellaWindow;

use ash::vk;
use crate::init::device::DeviceInfo;
use crate::init::extensions::{ExtensionFunctionSet, InstanceExtensionLoader, InstanceExtensionLoaderFn, VkExtensionInfo};
use crate::rosella::{InstanceContext, VulkanVersion};
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
    let app_info = vk::ApplicationInfo::builder()
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
    AshLoadingError(ash::LoadingError),
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

impl From<ash::LoadingError> for InstanceCreateError {
    fn from(err: ash::LoadingError) -> Self {
        InstanceCreateError::AshLoadingError(err)
    }
}

impl From<std::str::Utf8Error> for InstanceCreateError {
    fn from(err: std::str::Utf8Error) -> Self {
        InstanceCreateError::Utf8Error(err)
    }
}

struct ApplicationInfo {
    application_name: CString,
    application_version: u32,
    engine_name: CString,
    engine_version: u32,
    api_version: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum InstanceFeatureState {
    Uninitialized,
    Initialized,
    Enabled,
    Disabled,
}

pub type InstanceFeatureSet = FeatureSet2<InstanceFeatureState, Box<dyn ApplicationInstanceFeature>>;

struct InstanceBuilder {
    processor: FeatureProcessor<InstanceFeatureState, Box<dyn ApplicationInstanceFeature>>,
    info: Option<InstanceInfo>,
    config: Option<InstanceConfigurator>,
    application_info: ApplicationInfo,
}

impl InstanceBuilder {
    fn init_condition(state: &InstanceFeatureState) -> Result<ConditionResult, InstanceCreateError> {
        if state != &InstanceFeatureState::Uninitialized {
            panic!("State should be uninitialized during init pass");
        }

        Ok(ConditionResult::Take)
    }

    fn init_get_test(state: &InstanceFeatureState) -> bool {
        if state == &InstanceFeatureState::Disabled {
            return false;
        } else if state != &InstanceFeatureState::Initialized {
            panic!("Invalid state");
        }
        return true;
    }

    fn enable_condition(state: &InstanceFeatureState) -> Result<ConditionResult, InstanceCreateError> {
        if state == &InstanceFeatureState::Disabled {
            return Ok(ConditionResult::Skip);
        }

        if state != &InstanceFeatureState::Initialized {
            panic!("State should be initialized during enable pass");
        }

        Ok(ConditionResult::Take)
    }

    fn enable_get_test(state: &InstanceFeatureState) -> bool {
        if state == &InstanceFeatureState::Disabled {
            return false;
        } else if state != &InstanceFeatureState::Enabled {
            panic!("Invalid state");
        }
        return true;
    }

    fn new(application_info: ApplicationInfo, features: Vec<(NamedUUID, Box<[NamedUUID]>, Box<dyn ApplicationInstanceFeature>)>) -> Self {
        let processor = FeatureProcessor::from_graph(features.into_iter().map(
            |(name, deps, feature)| (name, deps, feature, InstanceFeatureState::Uninitialized)));

        Self {
            processor,
            info: None,
            config: None,
            application_info,
        }
    }

    fn run_init_pass(&mut self) -> Result<(), InstanceCreateError> {
        if self.info.is_some() {
            panic!("Called run init pass but info is already some");
        }
        self.info = Some(InstanceInfo::new(unsafe{ ash::Entry::new() }?)?);
        let info = self.info.as_ref().unwrap();

        self.processor.run_pass(
            Self::init_condition, &Self::init_get_test,
            |feature, set| {
                match feature.init(set, info) {
                    InitResult::Ok => Ok(InstanceFeatureState::Initialized),
                    InitResult::Disable => Ok(InstanceFeatureState::Disabled),
                }
            }
        )?;

        Ok(())
    }

    fn run_enable_pass(&mut self) -> Result<(), InstanceCreateError> {
        if self.config.is_some() {
            panic!("Called run enable pass but config is already some");
        }
        self.config = Some(InstanceConfigurator::new());
        let config = self.config.as_mut().unwrap();

        let info = self.info.as_ref().expect("Called run enable pass but info is none");

        self.processor.run_pass(
            Self::enable_condition, &Self::enable_get_test,
            |feature, set| {
                feature.enable(set, info, config);
                Ok(InstanceFeatureState::Enabled)
            }
        )?;

        Ok(())
    }

    fn build(self) -> Result<InstanceContext, InstanceCreateError> {
        let app_info = vk::ApplicationInfo::builder()
            .application_name(self.application_info.application_name.as_c_str())
            .application_version(self.application_info.application_version)
            .engine_name(self.application_info.engine_name.as_c_str())
            .engine_version(self.application_info.engine_version)
            .api_version(self.application_info.api_version);

        let info = self.info.expect("Called build but info is none");
        let (instance, function_set) = self.config.expect("Called build but config is none")
            .build_instance(&info, &app_info.build())?;

        Ok(InstanceContext::new(info.get_vulkan_version(), info.entry, instance, function_set))
    }
}

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
    enabled_extensions: HashMap<UUID, &'static InstanceExtensionLoaderFn>,
}

impl InstanceConfigurator {
    fn new() -> Self {
        Self{
            enabled_layers: HashSet::new(),
            enabled_extensions: HashMap::new(),
        }
    }

    pub fn enable_layer(&mut self, name: &String) {
        let uuid = NamedUUID::uuid_for(name);
        self.enabled_layers.insert(uuid);
    }

    pub fn enable_layer_uuid(&mut self, uuid: UUID) {
        self.enabled_layers.insert(uuid);
    }

    pub fn enable_extension<EXT: VkExtensionInfo + InstanceExtensionLoader + 'static>(&mut self) {
        let uuid = EXT::UUID.get_uuid();
        self.enabled_extensions.insert(uuid, &EXT::load_extension);
    }

    fn build_instance(self, info: &InstanceInfo, application_info: &vk::ApplicationInfo) -> Result<(ash::Instance, ExtensionFunctionSet), InstanceCreateError> {
        let mut layers = Vec::with_capacity(self.enabled_layers.len());
        for layer in &self.enabled_layers {
            layers.push(
                info.get_layer_properties_uuid(layer)
                    .ok_or(InstanceCreateError::LayerNotSupported)?
                    .get_c_name().as_ptr()
            );
        }

        let mut extensions = Vec::with_capacity(self.enabled_extensions.len());
        for (uuid, _) in &self.enabled_extensions {
            extensions.push(
                info.get_extension_properties_uuid(uuid)
                    .ok_or(InstanceCreateError::ExtensionNotSupported)?
                    .get_c_name().as_ptr()
            )
        }

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(application_info)
            .enabled_layer_names(layers.as_slice())
            .enabled_extension_names(extensions.as_slice());

        let instance = unsafe {
            info.get_entry().create_instance(&create_info.build(), None)
        }?;

        let mut function_set = ExtensionFunctionSet::new();
        for (_, extension) in &self.enabled_extensions {
            extension(&mut function_set, info.get_entry(), &instance);
        }

        Ok((instance, function_set))
    }
}