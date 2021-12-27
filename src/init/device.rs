//! Device initialization utilities
//!
//! An application can control how a vulkan device is created by using
//! [`ApplicationDeviceFeature`]s. Each feature represents some capability or set of capabilities
//! that a vulkan device may or may not support. The initialization code will call each feature
//! and enable it if it is supported. An application can mark features as required in which case
//! the init process will fail with [`DeviceCreateError::RequiredFeatureNotSupported`]  if any
//! required feature is not supported.
//!
//! Features can return data to the application if they are enabled. (This is not implemented yet)
//!
//! Features are processed in multiple stages. First [`ApplicationDeviceFeature::init`] is called
//! to query if a feature is supported. On any supported feature
//! [`ApplicationDeviceFeature::enable`] will then be called to enable it and configure the
//! instance. Finally after the vulkan instance has been created
//! [`ApplicationDeviceFeature::finish`] is called to generate the data that can be returned to
//! the application.
//!
//! To allow features to maintain internal state and process multiple potential physical devices
//! a [`ApplicationDeviceFeatureGenerator`] is used to generate a [`ApplicationDeviceFeature`]
//! instance for each physical device.
//!
//! Features can access other features during any of these stages. The ensure that dependencies have
//! already completed processing the respective stage these dependencies must be declared when
//! registering the feature into the [`InitializationRegistry`].

use std::any::Any;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use ash::extensions::khr::Swapchain;
use ash::prelude::VkResult;

use ash::vk;
use crate::init::application_feature::{ApplicationDeviceFeature, InitResult};

use crate::init::initialization_registry::InitializationRegistry;
use crate::init::utils::{ExtensionProperties, Feature, FeatureProcessor};
use crate::{NamedUUID, UUID};
use crate::init::EnabledFeatures;
use crate::util::extensions::{DeviceExtensionLoader, DeviceExtensionLoaderFn, ExtensionFunctionSet, VkExtensionInfo};
use crate::rosella::{DeviceContext, InstanceContext, VulkanVersion};

/// Internal implementation of the [`VulkanQueue`] struct
struct VulkanQueueImpl {
    queue: Mutex<vk::Queue>,
    family: u32,
}

/// A wrapper around vulkan queues which provides thread safe access to a queue.
#[derive(Clone)]
pub struct VulkanQueue(Arc<VulkanQueueImpl>);

impl VulkanQueue {
    fn new(queue: vk::Queue, family: u32) -> Self {
        Self(Arc::new(VulkanQueueImpl{ queue: Mutex::new(queue), family }))
    }

    /// Returns the family index of the queue
    pub fn get_family(&self) -> u32 {
        self.0.family
    }

    /// Returns the mutex that protects the queue
    pub fn access_queue(&self) -> &Mutex<vk::Queue> {
        &self.0.queue
    }

    /// Performs a thread safe vkQueueSubmit call
    pub fn queue_submit(&self, device: ash::Device, submits: &[vk::SubmitInfo], fence: vk::Fence) -> VkResult<()> {
        let guard = self.0.queue.lock().unwrap();
        unsafe { device.queue_submit(*guard, submits, fence) }
    }

    /// Performs a thread safe vkQueueBindSparse call
    pub fn queue_bind_sparse(&self, device: ash::Device, submits: &[vk::BindSparseInfo], fence: vk::Fence) -> VkResult<()> {
        let guard = self.0.queue.lock().unwrap();
        unsafe { device.queue_bind_sparse(*guard, submits, fence) }
    }

    /// Performs a thread safe vkQueuePresentKHR call
    pub fn queue_present_khr(&self, swapchain: Swapchain, present_info: &vk::PresentInfoKHR) -> VkResult<bool> {
        let guard = self.0.queue.lock().unwrap();
        unsafe { swapchain.queue_present(*guard, present_info) }
    }
}

/// An error that may occur during the device initialization process.
#[derive(Debug)]
pub enum DeviceCreateError {
    VulkanError(vk::Result),
    RequiredFeatureNotSupported(NamedUUID),
    Utf8Error(std::str::Utf8Error),
    NulError(std::ffi::NulError),
    ExtensionNotSupported,
    NoSuitableDeviceFound,
}

impl From<vk::Result> for DeviceCreateError {
    fn from(err: vk::Result) -> Self {
        DeviceCreateError::VulkanError(err)
    }
}

impl From<std::str::Utf8Error> for DeviceCreateError {
    fn from(err: std::str::Utf8Error) -> Self {
        DeviceCreateError::Utf8Error(err)
    }
}

impl From<std::ffi::NulError> for DeviceCreateError {
    fn from(err: std::ffi::NulError) -> Self {
        DeviceCreateError::NulError(err)
    }
}

/// Creates a single new device based on the features declared in the provided registry.
///
/// This function will consume the device features stored in the registry.
///
/// All discovered physical devices will be processed and the most suitable device will be selected.
/// (TODO not implemented yet)
pub fn create_device(registry: &mut InitializationRegistry, instance: InstanceContext) -> Result<DeviceContext, DeviceCreateError> {
    let (graph, features) : (Vec<_>, Vec<_>) = registry.take_device_features().into_iter().map(
        |(name, dependencies, feature, required)| {
            ((name.clone(), dependencies), (name, feature, required))
        }).unzip();

    let feature_lookup : HashSet<_> = features.iter().map(|(uuid, _, _)| uuid.get_uuid()).collect();

    let mut topo_sort = topological_sort::TopologicalSort::new();
    for (node, dependencies) in graph {
        for dependency in dependencies.iter() {
            topo_sort.add_dependency(dependency.clone(), node.clone());
        }
        topo_sort.insert(node);
    }
    let ordering : Vec<NamedUUID> = topo_sort
        .filter(|uuid: &NamedUUID| feature_lookup.contains(&uuid.get_uuid())) // Remove features that dont exist
        .collect();

    let devices = unsafe { instance.vk().enumerate_physical_devices() }?;
    let devices : Vec<_> = devices.into_iter().map(|device| {
        let feature_instances : Vec<_> = features.iter().map(
            |(name, feature, required)| {
                (name.clone(), feature.make_instance(), *required)
            }).collect();

        DeviceBuilder::new(instance.clone(), device, ordering.clone().into_boxed_slice(), feature_instances)
    }).collect();

    let mut devices : Vec<_> = devices.into_iter().filter_map(|mut device| {
        if device.run_init_pass().is_err() {
            return None;
        }
        if device.run_enable_pass().is_err() {
            return None;
        }
        Some(device)
    }).collect();

    if devices.is_empty() {
        return Err(DeviceCreateError::NoSuitableDeviceFound);
    }

    let device = devices.remove(0).build()?;

    Ok(device)
}

/// Represents the current state of some feature in the device initialization process
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum DeviceFeatureState {
    Uninitialized,
    Initialized,
    Enabled,
    Disabled
}

/// Meta information of a feature needed during the initialization process
struct DeviceFeatureInfo {
    feature: Box<dyn ApplicationDeviceFeature>,
    state: DeviceFeatureState,
    name: NamedUUID,
    required: bool,
}

impl Feature for DeviceFeatureInfo {
    type State = DeviceFeatureState;

    fn get_payload(&self, pass_state: &Self::State) -> Option<&dyn Any> {
        if self.state == DeviceFeatureState::Disabled {
            return None;
        }
        if &self.state != pass_state {
            panic!("Attempted to access feature in invalid state");
        }

        Some(self.feature.as_ref().as_any())
    }

    fn get_payload_mut(&mut self, pass_state: &Self::State) -> Option<&mut dyn Any> {
        if self.state == DeviceFeatureState::Disabled {
            return None;
        }
        if &self.state != pass_state {
            panic!("Attempted to access feature in invalid state");
        }

        Some(self.feature.as_mut().as_any_mut())
    }
}

/// High level implementation of the device init process.
struct DeviceBuilder {
    processor: FeatureProcessor<DeviceFeatureInfo>,
    instance: InstanceContext,
    physical_device: vk::PhysicalDevice,
    info: Option<DeviceInfo>,
    config: Option<DeviceConfigurator>,
}

impl DeviceBuilder {
    /// Generates a new builder for some feature set and physical device.
    ///
    /// No vulkan functions will be called here.
    fn new(instance: InstanceContext, physical_device: vk::PhysicalDevice, order: Box<[NamedUUID]>, features: Vec<(NamedUUID, Box<dyn ApplicationDeviceFeature>, bool)>) -> Self {
        let processor = FeatureProcessor::new(features.into_iter().map(
            |(name, feature, required)|
                (name.get_uuid(),
                 DeviceFeatureInfo {
                    feature,
                    state: DeviceFeatureState::Uninitialized,
                    name,
                    required,
                })
        ), order);

        Self {
            processor,
            instance,
            physical_device,
            info: None,
            config: None,
        }
    }

    /// Runs the init pass.
    ///
    /// First collects information about the capabilities of the physical device and then calls
    /// [`ApplicationDeviceFeature::init`] on all registered features in topological order.
    fn run_init_pass(&mut self) -> Result<(), DeviceCreateError> {
        log::debug!("Starting init pass");

        if self.info.is_some() {
            panic!("Called run init pass but info is already some");
        }
        self.info = Some(DeviceInfo::new(self.instance.clone(), self.physical_device)?);
        let info = self.info.as_ref().unwrap();

        self.processor.run_pass::<DeviceCreateError, _>(
            DeviceFeatureState::Initialized,
            |feature, access| {
                if feature.state != DeviceFeatureState::Uninitialized {
                    panic!("Feature is not in uninitialized state in init pass");
                }
                match feature.feature.init(access, info) {
                    InitResult::Ok => {
                        log::debug!("Initialized feature {:?}", feature.name);
                        feature.state = DeviceFeatureState::Initialized;
                    }
                    InitResult::Disable => {
                        feature.state = DeviceFeatureState::Disabled;
                        log::debug!("Disabled feature {:?}", feature.name);
                        if feature.required {
                            log::warn!("Failed to initialize required feature {:?}", feature.name);
                            return Err(DeviceCreateError::RequiredFeatureNotSupported(feature.name.clone()))
                        }
                    }
                }
                Ok(())
            }
        )?;

        Ok(())
    }

    /// Runs the enable pass
    ///
    /// Creates a [`DeviceConfigurator`] instance and calls [`ApplicationDeviceFeature::enable`]
    /// on all supported features to configure the device. This function does not create the
    /// vulkan device.
    fn run_enable_pass(&mut self) -> Result<(), DeviceCreateError> {
        if self.config.is_some() {
            panic!("Called run enable pass but config is already some");
        }
        self.config = Some(DeviceConfigurator::new());
        let config = self.config.as_mut().unwrap();

        let info = self.info.as_ref().expect("Called run enable pass but info is none");

        self.processor.run_pass::<DeviceCreateError, _>(
            DeviceFeatureState::Enabled,
            |feature, access| {
                if feature.state == DeviceFeatureState::Disabled {
                    return Ok(())
                }
                if feature.state != DeviceFeatureState::Initialized {
                    panic!("Feature is not in initialized state in enable pass");
                }
                feature.feature.enable(access, info, config);
                feature.state = DeviceFeatureState::Enabled;
                Ok(())
            }
        )?;

        Ok(())
    }

    /// Creates the vulkan device
    fn build(self) -> Result<DeviceContext, DeviceCreateError> {
        let instance = self.instance;

        let info = self.info.expect("Called build but info is none");
        let (device, function_set) = self.config.expect("Called build but config is none")
            .build_device(&info)?;

        let features = EnabledFeatures::new(self.processor.into_iter().filter_map(
            |mut info| {
                Some((info.name.get_uuid(), info.feature.as_mut().finish(&instance, &device, &function_set)))
            }));

        Ok(DeviceContext::new(instance, device, self.physical_device, function_set, features))
    }
}

/// Information about a queue family
pub struct QueueFamilyInfo {
    index: u32,
    properties: vk::QueueFamilyProperties,
}

impl QueueFamilyInfo {
    /// Collects information from a VK1.0 vkQueueFamilyProperties struct
    fn new(index: u32, properties: vk::QueueFamilyProperties) -> Self {
        Self {
            index,
            properties,
        }
    }

    /// Collects information form a VK1.1 vkQueueFamilyProperties2 struct
    fn new2(index: u32, properties2: vk::QueueFamilyProperties2) -> Self {
        let properties = properties2.queue_family_properties;

        Self {
            index,
            properties,
        }
    }

    /// Returns the queue family index
    pub fn get_index(&self) -> u32 {
        self.index
    }

    /// Returns the vkQueueFamilyProperties of this queue family
    pub fn get_properties(&self) -> &vk::QueueFamilyProperties {
        &self.properties
    }
}

/// Contains information about the vulkan device.
pub struct DeviceInfo {
    instance: InstanceContext,
    physical_device: vk::PhysicalDevice,
    features_1_0: vk::PhysicalDeviceFeatures,
    features_1_1: Option<vk::PhysicalDeviceVulkan11Features>,
    features_1_2: Option<vk::PhysicalDeviceVulkan12Features>,
    properties_1_0: vk::PhysicalDeviceProperties,
    properties_1_1: Option<vk::PhysicalDeviceVulkan11Properties>,
    properties_1_2: Option<vk::PhysicalDeviceVulkan12Properties>,
    memory_properties_1_0: vk::PhysicalDeviceMemoryProperties,

    /// Temporary hack until extension feature management is implemented
    timeline_semaphore_features: Option<vk::PhysicalDeviceTimelineSemaphoreFeatures>,
    queue_families: Box<[QueueFamilyInfo]>,
    extensions: HashMap<UUID, ExtensionProperties>,
}

impl DeviceInfo {
    fn new(instance: InstanceContext, physical_device: vk::PhysicalDevice) -> Result<Self, DeviceCreateError> {
        let features_1_0;
        let mut features_1_1 = None;
        let mut features_1_2 = None;

        let properties_1_0;
        let mut properties_1_1 = None;
        let mut properties_1_2 = None;

        let memory_properties_1_0;

        let mut timeline_semaphore = None;

        let queue_families;

        let vk_1_1 = instance.get_version().is_supported(VulkanVersion::VK_1_1);
        let vk_1_2 = instance.get_version().is_supported(VulkanVersion::VK_1_2);
        let get_physical_device_properties_2 = instance.get_extension::<ash::extensions::khr::GetPhysicalDeviceProperties2>();

        if vk_1_1 || get_physical_device_properties_2.is_some() {
            // Use the newer VK_KHR_get_physical_device_properties2 functions
            let mut features2 = vk::PhysicalDeviceFeatures2::builder();
            let mut properties2 = vk::PhysicalDeviceProperties2::builder();
            let mut memory_properties2 = vk::PhysicalDeviceMemoryProperties2::builder();

            if vk_1_1 {
                features_1_1 = Some(vk::PhysicalDeviceVulkan11Features::default());
                features2 = features2.push_next(features_1_1.as_mut().unwrap());

                properties_1_1 = Some(vk::PhysicalDeviceVulkan11Properties::default());
                properties2 = properties2.push_next(properties_1_1.as_mut().unwrap());
            }

            if vk_1_2 {
                features_1_2 = Some(vk::PhysicalDeviceVulkan12Features::default());
                features2 = features2.push_next(features_1_2.as_mut().unwrap());

                properties_1_2 = Some(vk::PhysicalDeviceVulkan12Properties::default());
                properties2 = properties2.push_next(properties_1_2.as_mut().unwrap());
            }

            if instance.is_extension_enabled(ash::extensions::khr::TimelineSemaphore::UUID.get_uuid()) {
                timeline_semaphore = Some(vk::PhysicalDeviceTimelineSemaphoreFeatures::default());
                features2 = features2.push_next(timeline_semaphore.as_mut().unwrap());
            }

            if vk_1_1 {
                unsafe { instance.vk().get_physical_device_features2(physical_device, &mut features2) };
            } else {
                unsafe { get_physical_device_properties_2.unwrap().get_physical_device_features2(physical_device, features2.borrow_mut()) };
            }
            features_1_0 = Some(features2.features);
            drop(features2); // Get rid of mut references

            if vk_1_1 {
                unsafe { instance.vk().get_physical_device_properties2(physical_device, &mut properties2) };
            } else {
                unsafe { get_physical_device_properties_2.unwrap().get_physical_device_properties2(physical_device, properties2.borrow_mut()) };
            }
            properties_1_0 = Some(properties2.properties);
            drop(properties2); // Get rid of mut references

            if vk_1_1 {
                unsafe { instance.vk().get_physical_device_memory_properties2(physical_device, &mut memory_properties2) };
            } else {
                unsafe { get_physical_device_properties_2.unwrap().get_physical_device_memory_properties2(physical_device, memory_properties2.borrow_mut()) };
            }
            memory_properties_1_0 = Some(memory_properties2.memory_properties);
            drop(memory_properties2); // Get rid of mut references


            let mut queue_properties2 = Vec::new();
            if vk_1_1 {
                let count = unsafe { instance.vk().get_physical_device_queue_family_properties2_len(physical_device) };

                queue_properties2.resize(count, vk::QueueFamilyProperties2::default());

                unsafe { instance.vk().get_physical_device_queue_family_properties2(physical_device, queue_properties2.as_mut()) };
            } else {
                let count = unsafe { get_physical_device_properties_2.unwrap().get_physical_device_queue_family_properties2_len(physical_device) };

                queue_properties2.resize(count, vk::QueueFamilyProperties2::default());

                unsafe { get_physical_device_properties_2.unwrap().get_physical_device_queue_family_properties2(physical_device, queue_properties2.as_mut()) };
            }

            queue_families = Some(queue_properties2.into_iter()
                .enumerate()
                .map(|(index, properties)| QueueFamilyInfo::new2(index as u32, properties))
                .collect::<Vec<_>>()
                .into_boxed_slice());

        } else {
            // Fallback to base vulkan 1.0 functions
            features_1_0 = Some(unsafe { instance.vk().get_physical_device_features(physical_device) });
            properties_1_0 = Some(unsafe { instance.vk().get_physical_device_properties(physical_device) });
            memory_properties_1_0 = Some(unsafe { instance.vk().get_physical_device_memory_properties(physical_device) });

            queue_families = Some(
                unsafe { instance.vk().get_physical_device_queue_family_properties(physical_device) }
                    .into_iter()
                    .enumerate()
                    .map(|(index, properties)| QueueFamilyInfo::new(index as u32, properties))
                .collect::<Vec<_>>()
                .into_boxed_slice());
        }

        let extensions_raw = unsafe { instance.vk().enumerate_device_extension_properties(physical_device) }?;
        let mut extensions = HashMap::new();
        for extension in extensions_raw {
            let extension = ExtensionProperties::new(&extension)?;
            let uuid = NamedUUID::uuid_for(extension.get_name().as_str());

            extensions.insert(uuid, extension);
        }

        Ok(Self {
            instance,
            physical_device,
            features_1_0: features_1_0.unwrap(),
            features_1_1,
            features_1_2,
            properties_1_0: properties_1_0.unwrap(),
            properties_1_1,
            properties_1_2,
            memory_properties_1_0: memory_properties_1_0.unwrap(),
            timeline_semaphore_features: timeline_semaphore,
            queue_families: queue_families.unwrap(),
            extensions,
        })
    }

    /// Returns the [`InstanceContext`] used
    pub fn get_instance(&self) -> &InstanceContext {
        &self.instance
    }

    /// Returns the physical device that is being processed
    pub fn get_physical_device(&self) -> &vk::PhysicalDevice {
        &self.physical_device
    }

    pub fn get_device_1_0_features(&self) -> &vk::PhysicalDeviceFeatures {
        &self.features_1_0
    }

    pub fn get_device_1_1_features(&self) -> Option<&vk::PhysicalDeviceVulkan11Features> {
        self.features_1_1.as_ref()
    }

    pub fn get_device_1_2_features(&self) -> Option<&vk::PhysicalDeviceVulkan12Features> {
        self.features_1_2.as_ref()
    }

    pub fn get_device_1_0_properties(&self) -> &vk::PhysicalDeviceProperties {
        &self.properties_1_0
    }

    pub fn get_device_1_1_properties(&self) -> Option<&vk::PhysicalDeviceVulkan11Properties> {
        self.properties_1_1.as_ref()
    }

    pub fn get_device_1_2_properties(&self) -> Option<&vk::PhysicalDeviceVulkan12Properties> {
        self.properties_1_2.as_ref()
    }

    pub fn get_memory_1_0_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.memory_properties_1_0
    }

    /// Temporary hack until extension feature management is implemented
    pub fn get_timeline_semaphore_features(&self) -> Option<&vk::PhysicalDeviceTimelineSemaphoreFeatures> {
        self.timeline_semaphore_features.as_ref()
    }

    pub fn get_queue_family_infos(&self) -> &[QueueFamilyInfo] {
        self.queue_families.as_ref()
    }

    /// Queries if a device extension is supported
    pub fn is_extension_supported<T: VkExtensionInfo>(&self) -> bool {
        self.extensions.contains_key(&T::UUID.get_uuid())
    }

    /// Queries if a device extension is supported
    pub fn is_extension_supported_str(&self, name: &str) -> bool {
        let uuid = NamedUUID::uuid_for(name);
        self.extensions.contains_key(&uuid)
    }

    /// Queries if a device extension is supported
    pub fn is_extension_supported_uuid(&self, uuid: &UUID) -> bool {
        self.extensions.contains_key(uuid)
    }

    /// Returns the properties of a device extension
    ///
    /// If the extension is not supported returns [`None`]
    pub fn get_extension_properties<T: VkExtensionInfo>(&self) -> Option<&ExtensionProperties> {
        self.extensions.get(&T::UUID.get_uuid())
    }

    /// Returns the properties of a device extension
    ///
    /// If the extension is not supported returns [`None`]
    pub fn get_extension_properties_str(&self, name: &str) -> Option<&ExtensionProperties> {
        let uuid = NamedUUID::uuid_for(name);
        self.extensions.get(&uuid)
    }

    /// Returns the properties of a device extension
    ///
    /// If the extension is not supported returns [`None`]
    pub fn get_extension_properties_uuid(&self, uuid: &UUID) -> Option<&ExtensionProperties> {
        self.extensions.get(uuid)
    }
}

/// Internal implementation of queue requests.
struct QueueRequestImpl {
    result: Option<VulkanQueue>,
}

impl QueueRequestImpl {
    /// Generates a new queue request for a specific family
    fn new(family: u32) -> (QueueRequest, QueueRequestResolver) {
        let cell = Rc::new(RefCell::new(QueueRequestImpl{ result: None }));
        (QueueRequest(cell.clone()), QueueRequestResolver{ request: cell, family, index: None })
    }
}

/// A queue request
///
/// During the enable pass features may request queues. A [`QueueRequest`] will be returned in such
/// a case. [`QueueRequests`] can be accessed to retrieve a [`VulkanQueue`] during the finish pass.
pub struct QueueRequest(Rc<RefCell<QueueRequestImpl>>);

impl QueueRequest {
    /// Returns the [`VulkanQueue`] to fulfill this request.
    ///
    /// # Panics
    /// Will panic if the request has not yet been resolved. Or in other words if this function is
    /// called before the finish pass.
    pub fn get(&self) -> VulkanQueue {
        self.0.borrow().result.as_ref().unwrap().clone()
    }
}

struct QueueRequestResolver {
    request: Rc<RefCell<QueueRequestImpl>>,
    family: u32,
    index: Option<u32>,
}

impl QueueRequestResolver {
    /// Resolves the queue request
    fn resolve(&mut self, queue: VulkanQueue) {
        (*self.request).borrow_mut().result = Some(queue);
    }

    fn get_family(&self) -> u32 {
        self.family
    }
}

pub struct DeviceConfigurator {
    enabled_extensions: HashMap<UUID, Option<&'static DeviceExtensionLoaderFn>>,
    queue_requests: Vec<QueueRequestResolver>,

    /// Temporary hack until extension feature management is implemented
    enable_timeline_semaphores: bool,
}

impl DeviceConfigurator {
    fn new() -> Self {
        Self{
            enabled_extensions: HashMap::new(),
            queue_requests: Vec::new(),
            enable_timeline_semaphores: false,
        }
    }

    /// Enables a device extension and registers the extension for automatic function loading
    pub fn enable_extension<EXT: VkExtensionInfo + DeviceExtensionLoader + 'static>(&mut self) {
        let uuid = EXT::UUID.get_uuid();
        self.enabled_extensions.insert(uuid, Some(&EXT::load_extension));
    }

    /// Enables a device extension without automatic function loading
    pub fn enable_extension_str_no_load(&mut self, str: &str) {
        let uuid = NamedUUID::uuid_for(str);

        // Do not override a variant where the loader is potentially set
        if !self.enabled_extensions.contains_key(&uuid) {
            self.enabled_extensions.insert(uuid, None);
        }
    }

    /// Creates a queue request
    pub fn add_queue_request(&mut self, family: u32) -> QueueRequest {
        let (request, resolver) = QueueRequestImpl::new(family);
        self.queue_requests.push(resolver);
        request
    }

    /// Temporary hack until extension feature management is implemented
    pub fn enable_timeline_semaphore(&mut self) {
        self.enable_timeline_semaphores = true;
    }

    /// Generates queue assignments to fulfill requests
    ///
    /// Currently only generates 1 queue per needed family.
    /// TODO maybe use multiple queues if supported?
    fn generate_queue_assignments(&mut self, info: &DeviceInfo) -> Box<[(u32, Box<[f32]>)]> {
        let mut families = Vec::new();
        families.resize_with(info.get_queue_family_infos().len(), || 0u32);

        for request in &mut self.queue_requests {
            *families.get_mut(request.get_family() as usize).unwrap() += 1u32;
            request.index = Some(0);
        }

        families.into_iter().enumerate().filter_map(|(i, c)| if c != 0u32 {
            let mut priorities = Vec::new();
            priorities.resize_with(c as usize, || 1.0f32);
            Some((i as u32, priorities.into_boxed_slice()))
        } else { None }).collect()
    }

    /// Creates a vulkan device based on the configuration stored in this DeviceConfigurator
    fn build_device(mut self, info: &DeviceInfo) -> Result<(ash::Device, ExtensionFunctionSet), DeviceCreateError> {
        let mut extensions = Vec::with_capacity(self.enabled_extensions.len());
        for (uuid, _) in &self.enabled_extensions {
            extensions.push(
                info.get_extension_properties_uuid(uuid)
                    .ok_or(DeviceCreateError::ExtensionNotSupported)?
                    .get_c_name().as_ptr()
            )
        }

        let queue_assignments = self.generate_queue_assignments(info);
        let mut queue_create_infos = Vec::with_capacity(queue_assignments.len());
        for (family, priorities) in queue_assignments.iter() {
            let create_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*family)
                .queue_priorities(priorities);
            queue_create_infos.push(*create_info);
        }

        let mut create_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(extensions.as_slice())
            .queue_create_infos(queue_create_infos.as_slice());

        // Temporary hack until extension feature management is implemented
        let mut timeline_semaphore_info;
        if self.enable_timeline_semaphores {
            timeline_semaphore_info = vk::PhysicalDeviceTimelineSemaphoreFeatures::builder()
                .timeline_semaphore(true);
            create_info = create_info.push_next(&mut timeline_semaphore_info);
        }

        let device = unsafe {
            info.get_instance().vk().create_device(info.physical_device, &create_info, None)
        }?;

        let mut queues = Vec::with_capacity(queue_assignments.len());
        for (family, priorities) in queue_assignments.iter() {
            let mut family_queues = Vec::with_capacity(priorities.len());
            for i in 0u32..(priorities.len() as u32) {
                let queue = unsafe { device.get_device_queue(*family, i) };
                family_queues.push(VulkanQueue::new(queue, *family));
            }
            queues.push(family_queues);
        }
        let queues = queues;

        for request in &mut self.queue_requests {
            request.resolve(queues.get(request.family as usize).unwrap().get(request.index.unwrap() as usize).unwrap().clone());
        }

        let mut function_set = ExtensionFunctionSet::new();
        for (_, extension) in &self.enabled_extensions {
            if let Some(extension) = extension {
                extension(&mut function_set, info.get_instance().get_entry(), info.get_instance().vk(), &device);
            }
        }

        Ok((device, function_set))
    }
}