use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::ffi::CString;

use crate::{ UUID, NamedUUID };
use crate::init::application_feature::{ApplicationInstanceFeature, InitResult};

use crate::init::initialization_registry::{InitializationRegistry};
use crate::init::utils::{ExtensionProperties, Feature, FeatureProcessor, LayerProperties};

use ash::vk;
use ash::vk::{DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT};
use log::debug;
use crate::util::extensions::{ExtensionFunctionSet, InstanceExtensionLoader, InstanceExtensionLoaderFn, VkExtensionInfo};
use crate::rosella::{InstanceContext, VulkanVersion};

#[derive(Debug)]
pub enum InstanceCreateError {
    VulkanError(vk::Result),
    AshInstanceError(ash::InstanceError),
    AshLoadingError(ash::LoadingError),
    Utf8Error(std::str::Utf8Error),
    NulError(std::ffi::NulError),
    RequiredFeatureNotSupported(NamedUUID),
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

impl From<std::ffi::NulError> for InstanceCreateError {
    fn from(err: std::ffi::NulError) -> Self {
        InstanceCreateError::NulError(err)
    }
}

pub fn create_instance(registry: &mut InitializationRegistry, application_name: &str, application_version: u32) -> Result<InstanceContext, InstanceCreateError> {
    let application_info = ApplicationInfo{
        application_name: CString::new(application_name)?,
        application_version,
        engine_name: CString::new("Rosella")?,
        engine_version: 0, // TODO
        api_version: vk::API_VERSION_1_2
    };

    log::info!("Creating instance for \"{}\" {}", application_name, application_version);

    let mut builder = InstanceBuilder::new(application_info, registry.take_instance_features());
    builder.run_init_pass()?;
    builder.run_enable_pass()?;
    builder.build()
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

pub struct FeatureInfo {
    feature: Box<dyn ApplicationInstanceFeature>,
    state: InstanceFeatureState,
    name: NamedUUID,
    required: bool,
}

impl Feature for FeatureInfo {
    type State = InstanceFeatureState;

    fn get_payload(&self, pass_state: &Self::State) -> Option<&dyn Any> {
        if self.state == InstanceFeatureState::Disabled {
            return None;
        }
        if &self.state != pass_state {
            panic!("Attempted to access feature in invalid state");
        }

        Some(self.feature.as_ref().as_any())
    }

    fn get_payload_mut(&mut self, pass_state: &Self::State) -> Option<&mut dyn Any> {
        if self.state == InstanceFeatureState::Disabled {
            return None;
        }
        if &self.state != pass_state {
            panic!("Attempted to access feature in invalid state");
        }

        Some(self.feature.as_mut().as_any_mut())
    }
}

struct InstanceBuilder {
    processor: FeatureProcessor<FeatureInfo>,
    info: Option<InstanceInfo>,
    config: Option<InstanceConfigurator>,
    application_info: ApplicationInfo,
}

impl InstanceBuilder {
    fn new(application_info: ApplicationInfo, features: Vec<(NamedUUID, Box<[NamedUUID]>, Box<dyn ApplicationInstanceFeature>, bool)>) -> Self {
        let processor = FeatureProcessor::from_graph(features.into_iter().map(
            |(name, deps, feature, required)| {
                log::debug!("Instance feature {:?}", name);
                let info = FeatureInfo {
                    feature,
                    state: InstanceFeatureState::Uninitialized,
                    name: name.clone(),
                    required
                };
                (name, deps, info)
            }));

        Self {
            processor,
            info: None,
            config: None,
            application_info,
        }
    }

    fn run_init_pass(&mut self) -> Result<(), InstanceCreateError> {
        log::debug!("Starting init pass");

        if self.info.is_some() {
            panic!("Called run init pass but info is already some");
        }
        self.info = Some(InstanceInfo::new(unsafe{ ash::Entry::new() }?)?);
        let info = self.info.as_ref().unwrap();

        self.processor.run_pass::<InstanceCreateError, _>(
            InstanceFeatureState::Initialized,
            |feature, access| {
                if feature.state != InstanceFeatureState::Uninitialized {
                    panic!("Feature is not in uninitialized state in init pass");
                }
                match feature.feature.init(access, info) {
                    InitResult::Ok => feature.state = InstanceFeatureState::Initialized,
                    InitResult::Disable => {
                        feature.state = InstanceFeatureState::Disabled;
                        log::debug!("Disabled feature {:?}", feature.name);
                        if feature.required {
                            log::warn!("Failed to initialize required feature {:?}", feature.name);
                            return Err(InstanceCreateError::RequiredFeatureNotSupported(feature.name.clone()))
                        }
                    },
                }
                log::debug!("Initialized feature {:?}", feature.name);
                Ok(())
            }
        )?;

        Ok(())
    }

    fn run_enable_pass(&mut self) -> Result<(), InstanceCreateError> {
        log::debug!("Starting enable pass");

        if self.config.is_some() {
            panic!("Called run enable pass but config is already some");
        }
        self.config = Some(InstanceConfigurator::new());
        let config = self.config.as_mut().unwrap();

        let info = self.info.as_ref().expect("Called run enable pass but info is none");

        self.processor.run_pass::<InstanceCreateError, _>(
            InstanceFeatureState::Enabled,
            |feature, access| {
                if feature.state == InstanceFeatureState::Disabled {
                    return Ok(())
                }
                if feature.state != InstanceFeatureState::Initialized {
                    panic!("Feature is not in initialized state in enable pass");
                }
                feature.feature.enable(access, info, config);
                feature.state = InstanceFeatureState::Enabled;
                Ok(())
            }
        )?;

        Ok(())
    }

    fn build(self) -> Result<InstanceContext, InstanceCreateError> {
        log::debug!("Building instance");

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
            let uuid = NamedUUID::uuid_for(layer.get_name().as_str());

            layers.insert(uuid, layer);
        }

        let extensions_raw = entry.enumerate_instance_extension_properties()?;
        let mut extensions = HashMap::new();
        for extension in extensions_raw {
            let extension = ExtensionProperties::new(&extension)?;
            let uuid = NamedUUID::uuid_for(extension.get_name().as_str());

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

    pub fn is_layer_supported_str(&self, name: &str) -> bool {
        let uuid = NamedUUID::uuid_for(name);
        self.layers.contains_key(&uuid)
    }

    pub fn is_layer_supported_uuid(&self, uuid: &UUID) -> bool {
        self.layers.contains_key(uuid)
    }

    pub fn get_layer_properties_str(&self, name: &str) -> Option<&LayerProperties> {
        let uuid = NamedUUID::uuid_for(name);
        self.layers.get(&uuid)
    }

    pub fn get_layer_properties_uuid(&self, uuid: &UUID) -> Option<&LayerProperties> {
        self.layers.get(uuid)
    }

    pub fn is_extension_supported<T: VkExtensionInfo>(&self) -> bool {
        self.extensions.contains_key(&T::UUID.get_uuid())
    }

    pub fn is_extension_supported_str(&self, name: &str) -> bool {
        let uuid = NamedUUID::uuid_for(name);
        self.extensions.contains_key(&uuid)
    }

    pub fn is_extension_supported_uuid(&self, uuid: &UUID) -> bool {
        self.extensions.contains_key(uuid)
    }

    pub fn get_extension_properties<T: VkExtensionInfo>(&self) -> Option<&ExtensionProperties> {
        self.extensions.get(&T::UUID.get_uuid())
    }

    pub fn get_extension_properties_str(&self, name: &str) -> Option<&ExtensionProperties> {
        let uuid = NamedUUID::uuid_for(name);
        self.extensions.get(&uuid)
    }

    pub fn get_extension_properties_uuid(&self, uuid: &UUID) -> Option<&ExtensionProperties> {
        self.extensions.get(uuid)
    }
}

pub struct InstanceConfigurator {
    enabled_layers: HashSet<UUID>,
    enabled_extensions: HashMap<UUID, Option<&'static InstanceExtensionLoaderFn>>,
    debug_util_messenger: vk::PFN_vkDebugUtilsMessengerCallbackEXT, // TODO Make this flexible somehow, probably requires general overhaul of p_next pushing
}

impl InstanceConfigurator {
    fn new() -> Self {
        Self{
            enabled_layers: HashSet::new(),
            enabled_extensions: HashMap::new(),
            debug_util_messenger: None,
        }
    }

    pub fn enable_layer(&mut self, name: &str) {
        let uuid = NamedUUID::uuid_for(name);
        self.enabled_layers.insert(uuid);
    }

    pub fn enable_layer_uuid(&mut self, uuid: UUID) {
        self.enabled_layers.insert(uuid);
    }

    pub fn enable_extension<EXT: VkExtensionInfo + InstanceExtensionLoader + 'static>(&mut self) {
        let uuid = EXT::UUID.get_uuid();
        self.enabled_extensions.insert(uuid, Some(&EXT::load_extension));
    }

    pub fn enable_extension_str_no_load(&mut self, str: &str) {
        let uuid = NamedUUID::uuid_for(str);

        // Do not override a variant where the loader is potentially set
        if !self.enabled_extensions.contains_key(&uuid) {
            self.enabled_extensions.insert(uuid, None);
        }
    }

    pub fn set_debug_messenger(&mut self, messenger: vk::PFN_vkDebugUtilsMessengerCallbackEXT) {
        self.debug_util_messenger = messenger;
    }

    fn build_instance(self, info: &InstanceInfo, application_info: &vk::ApplicationInfo) -> Result<(ash::Instance, ExtensionFunctionSet), InstanceCreateError> {
        let mut layers = Vec::with_capacity(self.enabled_layers.len());
        for layer in &self.enabled_layers {
            let layer = info.get_layer_properties_uuid(layer)
                .ok_or(InstanceCreateError::LayerNotSupported)?;

            log::debug!("Enabling layer \"{}\"", layer.get_name());

            layers.push(layer.get_c_name().as_ptr());
        }

        let mut extensions = Vec::with_capacity(self.enabled_extensions.len());
        for (uuid, loader) in &self.enabled_extensions {
            let extension = info.get_extension_properties_uuid(uuid)
                .ok_or(InstanceCreateError::ExtensionNotSupported)?;

            if loader.is_some() {
                log::debug!("Enabling extension \"{}\"", extension.get_name());
            } else {
                log::debug!("Enabling no load extension \"{}\"", extension.get_name());
            }

            extensions.push(extension.get_c_name().as_ptr());
        }

        let mut create_info = vk::InstanceCreateInfo::builder()
            .application_info(application_info)
            .enabled_layer_names(layers.as_slice())
            .enabled_extension_names(extensions.as_slice());

        let mut messenger;
        if self.debug_util_messenger.is_some() {
            messenger = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(DebugUtilsMessageSeverityFlagsEXT::all())
                .message_type(DebugUtilsMessageTypeFlagsEXT::all())
                .pfn_user_callback(self.debug_util_messenger);

            create_info = create_info.push_next(&mut messenger);
        }

        let instance = unsafe {
            info.get_entry().create_instance(&create_info, None)
        }?;

        let mut function_set = ExtensionFunctionSet::new();
        for (_, extension) in &self.enabled_extensions {
            if let Some(extension) = extension {
                extension(&mut function_set, info.get_entry(), &instance);
            }
        }

        log::debug!("Instance creation successful");

        Ok((instance, function_set))
    }
}