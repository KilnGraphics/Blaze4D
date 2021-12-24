//! Instance initialization utilities
//!
//! An application can control how a vulkan instance is created by using
//! [`ApplicationInstanceFeature`]s. Each feature represents some capability or set of capabilities
//! that a vulkan instance may or may not support. The initialization code will call each feature
//! and enable it if it is supported. An application can mark features as required in which case
//! the init process will fail with [`InstanceCreateError::RequiredFeatureNotSupported`]  if any
//! required feature is not supported.
//!
//! Features can return data to the application if they are enabled. (This is not implemented yet)
//!
//! Features are processed in multiple stages. First [`ApplicationInstanceFeature::init`] is called
//! to query if a feature is supported. On any supported feature
//! [`ApplicationInstanceFeature::enable`] will then be called to enable it and configure the
//! instance. Finally after the vulkan instance has been created
//! [`ApplicationInstanceFeature::finish`] is called to generate the data that can be returned to
//! the application.
//!
//! Features can access other features during any of these stages. The ensure that dependencies have
//! already completed processing the respective stage these dependencies must be declared when
//! registering the feature into the [`InitializationRegistry`].

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::ffi::CString;

use crate::{ UUID, NamedUUID };
use crate::init::application_feature::{ApplicationInstanceFeature, InitResult};

use crate::init::initialization_registry::{InitializationRegistry};
use crate::init::utils::{ExtensionProperties, Feature, FeatureProcessor, LayerProperties};

use ash::vk;
use ash::vk::{DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT};
use crate::util::extensions::{ExtensionFunctionSet, InstanceExtensionLoader, InstanceExtensionLoaderFn, VkExtensionInfo};
use crate::rosella::{InstanceContext, VulkanVersion};

/// An error that may occur during the instance initialization process.
#[derive(Debug)]
pub enum InstanceCreateError {
    VulkanError(vk::Result),
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

/// Creates a new instance based on the features declared in the provided registry.
///
/// This function will consume the instance features stored in the registry.
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

/// Represents the current state of some feature in the instance initialization process
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum InstanceFeatureState {
    Uninitialized,
    Initialized,
    Enabled,
    Disabled,
}

/// Meta information of a feature needed during the initialization process
struct InstanceFeatureInfo {
    feature: Box<dyn ApplicationInstanceFeature>,
    state: InstanceFeatureState,
    name: NamedUUID,
    required: bool,
}

impl Feature for InstanceFeatureInfo {
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

/// High level implementation of the instance init process.
struct InstanceBuilder {
    processor: FeatureProcessor<InstanceFeatureInfo>,
    info: Option<InstanceInfo>,
    config: Option<InstanceConfigurator>,
    application_info: ApplicationInfo,
}

impl InstanceBuilder {
    /// Generates a new builder for some feature set.
    ///
    /// No vulkan functions will be called here.
    fn new(application_info: ApplicationInfo, features: Vec<(NamedUUID, Box<[NamedUUID]>, Box<dyn ApplicationInstanceFeature>, bool)>) -> Self {
        let processor = FeatureProcessor::from_graph(features.into_iter().map(
            |(name, deps, feature, required)| {
                log::debug!("Instance feature {:?}", name);
                let info = InstanceFeatureInfo {
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

    /// Runs the init pass.
    ///
    /// First collects information about the capabilities of the vulkan environment and then calls
    /// [`ApplicationInstanceFeature::init`] on all registered features in topological order.
    fn run_init_pass(&mut self) -> Result<(), InstanceCreateError> {
        log::debug!("Starting init pass");

        if self.info.is_some() {
            panic!("Called run init pass but info is already some");
        }
        self.info = Some(InstanceInfo::new(ash::Entry::new() )?);
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

    /// Runs the enable pass
    ///
    /// Creates a [`InstanceConfigurator`] instance and calls [`ApplicationInstanceFeature::enable`]
    /// on all supported features to configure the instance. This function does not create the
    /// vulkan instance.
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

    /// Creates the vulkan instance
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


/// Contains information about the vulkan environment.
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

    /// Returns an [`ash::Entry`] instance that can be used to access entry functions.
    pub fn get_entry(&self) -> &ash::Entry {
        &self.entry
    }

    /// Returns the version advertised by the vulkan environment
    pub fn get_vulkan_version(&self) -> VulkanVersion {
        self.version
    }

    /// Queries if a instance layer is supported
    pub fn is_layer_supported_str(&self, name: &str) -> bool {
        let uuid = NamedUUID::uuid_for(name);
        self.layers.contains_key(&uuid)
    }

    /// Queries if a instance layer is supported
    pub fn is_layer_supported_uuid(&self, uuid: &UUID) -> bool {
        self.layers.contains_key(uuid)
    }

    /// Returns the properties of a instance layer
    pub fn get_layer_properties_str(&self, name: &str) -> Option<&LayerProperties> {
        let uuid = NamedUUID::uuid_for(name);
        self.layers.get(&uuid)
    }

    /// Returns the properties of a instance layer
    pub fn get_layer_properties_uuid(&self, uuid: &UUID) -> Option<&LayerProperties> {
        self.layers.get(uuid)
    }

    /// Queries if a instance extension is supported
    pub fn is_extension_supported<T: VkExtensionInfo>(&self) -> bool {
        self.extensions.contains_key(&T::UUID.get_uuid())
    }

    /// Queries if a instance extension is supported
    pub fn is_extension_supported_str(&self, name: &str) -> bool {
        let uuid = NamedUUID::uuid_for(name);
        self.extensions.contains_key(&uuid)
    }

    /// Queries if a instance extension is supported
    pub fn is_extension_supported_uuid(&self, uuid: &UUID) -> bool {
        self.extensions.contains_key(uuid)
    }

    /// Returns the properties of a instance extension
    ///
    /// If the extension is not supported returns [`None`]
    pub fn get_extension_properties<T: VkExtensionInfo>(&self) -> Option<&ExtensionProperties> {
        self.extensions.get(&T::UUID.get_uuid())
    }

    /// Returns the properties of a instance extension
    ///
    /// If the extension is not supported returns [`None`]
    pub fn get_extension_properties_str(&self, name: &str) -> Option<&ExtensionProperties> {
        let uuid = NamedUUID::uuid_for(name);
        self.extensions.get(&uuid)
    }

    /// Returns the properties of a instance extension
    ///
    /// If the extension is not supported returns [`None`]
    pub fn get_extension_properties_uuid(&self, uuid: &UUID) -> Option<&ExtensionProperties> {
        self.extensions.get(uuid)
    }
}

/// Used by features to configure the created vulkan instance.
pub struct InstanceConfigurator {
    enabled_layers: HashSet<UUID>,
    enabled_extensions: HashMap<UUID, Option<&'static InstanceExtensionLoaderFn>>,

    /// Temporary hack until extensions can be properly handled
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

    /// Enables a instance layer
    pub fn enable_layer(&mut self, name: &str) {
        let uuid = NamedUUID::uuid_for(name);
        self.enabled_layers.insert(uuid);
    }

    /// Enables a instance layer
    pub fn enable_layer_uuid(&mut self, uuid: UUID) {
        self.enabled_layers.insert(uuid);
    }

    /// Enables a instance extension and registers the extension for automatic function loading
    pub fn enable_extension<EXT: VkExtensionInfo + InstanceExtensionLoader + 'static>(&mut self) {
        let uuid = EXT::UUID.get_uuid();
        self.enabled_extensions.insert(uuid, Some(&EXT::load_extension));
    }

    /// Enables a instance extension without automatic function loading
    pub fn enable_extension_no_load<EXT: VkExtensionInfo>(&mut self) {
        let uuid = EXT::UUID.get_uuid();
        self.enabled_extensions.insert(uuid, None);
    }

    /// Enables a instance extension without automatic function loading
    pub fn enable_extension_str_no_load(&mut self, str: &str) {
        let uuid = NamedUUID::uuid_for(str);

        // Do not override a variant where the loader is potentially set
        if !self.enabled_extensions.contains_key(&uuid) {
            self.enabled_extensions.insert(uuid, None);
        }
    }

    /// Sets the debug messenger for VK_EXT_debug_utils
    ///
    /// This is a temporary hack until extension configuration can be properly handled.
    pub fn set_debug_messenger(&mut self, messenger: vk::PFN_vkDebugUtilsMessengerCallbackEXT) {
        self.debug_util_messenger = messenger;
    }

    /// Creates a vulkan instance based on the configuration stored in this InstanceConfigurator
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
                .message_severity(DebugUtilsMessageSeverityFlagsEXT::VERBOSE | DebugUtilsMessageSeverityFlagsEXT::INFO | DebugUtilsMessageSeverityFlagsEXT::WARNING | DebugUtilsMessageSeverityFlagsEXT::ERROR)
                .message_type(DebugUtilsMessageTypeFlagsEXT::GENERAL | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE | DebugUtilsMessageTypeFlagsEXT::VALIDATION)
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