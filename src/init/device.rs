use std::any::Any;
use std::borrow::BorrowMut;
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use ash::extensions::khr::Swapchain;
use ash::prelude::VkResult;
use ash::vk::{BindSparseInfo, DeviceCreateInfo, DeviceQueueCreateInfo, Fence, PhysicalDevice, PhysicalDeviceFeatures2, PhysicalDeviceProperties, PhysicalDeviceType, PhysicalDeviceVulkan11Features, PhysicalDeviceVulkan12Features, PresentInfoKHR, Queue, QueueFamilyProperties, SubmitInfo, API_VERSION_1_1, API_VERSION_1_2, StructureType};
use ash::{Device, Instance};

use ash::vk;
use topological_sort::TopologicalSort;
use crate::init::application_feature::{ApplicationDeviceFeatureInstance, FeatureDependency, InitResult};

use crate::init::initialization_registry::InitializationRegistry;
use crate::init::instance::InstanceFeatureState;
use crate::init::utils::{ExtensionProperties, Feature, FeatureProcessor};
use crate::util::utils::string_from_array;
use crate::window::RosellaSurface;
use crate::{NamedUUID, UUID};
use crate::init::extensions::{DeviceExtensionLoader, DeviceExtensionLoaderFn, ExtensionFunctionSet, VkExtensionInfo};
use crate::rosella::{InstanceContext, VulkanVersion};

#[derive(Debug)]
pub struct VulkanQueue {
    queue: Mutex<Queue>,
    family: i32,
}

struct QueueRequest {
    requested_family: i32,
    assigned_index: i32,
    queue: Option<Arc<VulkanQueue>>,
}

/// A class that represents some collection of device features or capabilities.
///
/// Instances of this class can be registered into a FIXME {@link graphics.kiln.rosella.init.InitializationRegistry} which will then be
/// used to select and initialize a device.
///
/// This happens in 2 stages.
/// <ol>
///     <li>The feature is queried if the device supports the feature.</li>
///     <li>If support is detected and desired the feature will be called to configure the device.</li>
/// </ol>
/// For these interactions a instance of DeviceMeta is provided which manages
/// information for a single physical device.
///
/// Since multiple devices may be tested concurrently the createInstance function will be called for each device which
/// should return a object that can keep track of all necessary metadata it may need for one device. The ApplicationFeature
/// class as well as separate created instances may be called concurrently, however created instances individually will
/// never be called concurrently.
///
/// If the feature wants to return information to the application it can provide a metadata object which will be stored
/// in the created device for the application to access.
///
/// A feature can access the instances of other features, however it must make sure to declare dependencies as otherwise
/// those features may not have run yet.
///
/// The default implementation of this class only validates that all dependencies are met and does not create any metadata.
pub trait ApplicationFeature {
    fn get_feature_name(&self) -> NamedUUID;
    fn is_supported(&self, meta: &DeviceMeta) -> bool;
    fn enable(&self, meta: &mut DeviceMeta, instance: &Instance, surface: &RosellaSurface);
    fn get_dependencies(&self) -> HashSet<NamedUUID>;
}

/// Builds all information about features on the device and what is enabled.
/// TODO LOW_PRIORITY: add support for VK1.0 by not doing any of this on vk1.0. instead just create a simple VkPhysicalDeviceFeatures field
pub struct DeviceFeatureBuilder {
    pub vulkan_features: PhysicalDeviceFeatures2,
    pub vulkan_11_features: Option<PhysicalDeviceVulkan11Features>,
    pub vulkan_12_features: Option<PhysicalDeviceVulkan12Features>,
}

pub struct DeviceMeta {
    unsatisfied_requirements: Vec<NamedUUID>,
    features: HashMap<NamedUUID, Rc<dyn ApplicationFeature>>,
    pub feature_builder: DeviceFeatureBuilder,

    pub physical_device: PhysicalDevice,
    properties: PhysicalDeviceProperties,
    extension_properties: HashMap<String, vk::ExtensionProperties>,
    queue_family_properties: Vec<QueueFamilyProperties>, // TODO LOW_PRIORITY: look at QueueFamilyProperties2

    queue_requests: Vec<QueueRequest>,
    enabled_extensions: Vec<*const c_char>,
}

pub fn create_device(instance: &Instance, registry: InitializationRegistry, surface: &RosellaSurface) -> Device {
    let mut devices: Vec<DeviceMeta> = vec![];
    let raw_devices = unsafe { instance.enumerate_physical_devices() }.expect("Failed to find devices.");
    let application_features = registry.get_ordered_features();

    for physical_device in raw_devices {
        let mut meta = DeviceMeta::new(instance, physical_device, application_features.clone());
        meta.process_support();
        devices.push(meta);
    }

    //TODO: Sorting
    devices.remove(0).create_device(instance, surface)
}

impl DeviceMeta {
    pub fn new(instance: &Instance, physical_device: PhysicalDevice, application_features: Vec<Rc<dyn ApplicationFeature>>) -> DeviceMeta {
        let mut features = HashMap::new();

        for feature in application_features {
            features.insert(feature.get_feature_name(), feature);
        }

        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };

        let mut feature_builder = DeviceFeatureBuilder::new(device_properties.api_version);
        feature_builder.vulkan_features.features = unsafe { instance.get_physical_device_features(physical_device) };

        let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let mut extension_properties = HashMap::new();

        for extension_property in unsafe { instance.enumerate_device_extension_properties(physical_device) }.unwrap() {
            extension_properties.insert(string_from_array(&extension_property.extension_name), extension_property);
        }

        DeviceMeta {
            unsatisfied_requirements: vec![],
            features,
            feature_builder,
            physical_device,
            properties: device_properties,
            extension_properties,
            queue_family_properties,
            queue_requests: vec![],
            enabled_extensions: vec![],
        }
    }

    fn process_support(&mut self) {
        self.unsatisfied_requirements.clear();

        for feature in self.features.values() {
            if !feature.is_supported(self) {
                self.unsatisfied_requirements.push(feature.get_feature_name())
            }
        }
    }

    /// return true if all required features are met by this device.
    pub fn is_valid(&self) -> bool {
        self.unsatisfied_requirements.is_empty()
    }

    pub fn get_performance_ranking(&self) -> i32 {
        match self.properties.device_type {
            PhysicalDeviceType::VIRTUAL_GPU => 1,
            PhysicalDeviceType::INTEGRATED_GPU => 2,
            PhysicalDeviceType::DISCRETE_GPU => 3,
            _ => 0,
        }
    }

    pub fn add_queue_request(&mut self, family: i32) {
        self.queue_requests.push(QueueRequest::new(family));
    }

    pub fn enable_extension(&mut self, extension: *const c_char) {
        self.enabled_extensions.push(extension)
    }

    pub fn create_device(mut self, instance: &Instance, surface: &RosellaSurface) -> Device {
        for feature in std::mem::take(&mut self.features).values() {
            if feature.is_supported(&self) {
                feature.enable(&mut self, instance, surface);
            }
        }

        let queue_mappings = self.generate_queue_mappings();
        let mappings: Vec<DeviceQueueCreateInfo> = queue_mappings.iter().map(|x| { x.0 }).collect();
        let mut device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&mappings)
            .enabled_extension_names(&self.enabled_extensions)
            .push_next(&mut self.feature_builder.vulkan_features);

        if let Some(v11) = self.feature_builder.vulkan_11_features.as_mut() {
            device_create_info = device_create_info.push_next(v11);
        }

        if let Some(v12) = self.feature_builder.vulkan_12_features.as_mut() {
            device_create_info = device_create_info.push_next(v12);
        }

        let vk_device =
            unsafe { instance.create_device(self.physical_device, &device_create_info, None) }.expect("Failed to create the VkDevice!");

        self.fulfill_queue_requests(&vk_device);
        drop(queue_mappings);

        vk_device
    }

    fn generate_queue_mappings(&mut self) -> Vec<(DeviceQueueCreateInfo, Option<Vec<f32>>)> {
        let mut next_queue_indices = vec![0; self.queue_family_properties.len()];

        for mut request in self.queue_requests.iter_mut() {
            let requested_family = request.requested_family as usize;
            let index_requests = next_queue_indices[requested_family];
            let index: u32 = index_requests as u32;
            next_queue_indices[requested_family] += 1;

            request.assigned_index = (index % self.queue_family_properties[requested_family].queue_count) as i32;
        }

        let family_count = next_queue_indices.iter().filter(|&&x| x > 0).count();

        let mut queue_create_infos = vec![(DeviceQueueCreateInfo::default(), Option::<Vec<f32>>::None); family_count];

        for family in 0..next_queue_indices.len() {
            if next_queue_indices[family] == 0 {
                continue;
            }

            let length = min(
                next_queue_indices[family],
                self.queue_family_properties[family].queue_count as usize,
            );
            let priorities = vec![1.0; length];

            let (info, vec) = &mut queue_create_infos[family];
            info.queue_family_index = family as u32;
            info.p_queue_priorities = priorities.as_ptr();
            info.queue_count = length as u32;
            *vec = Some(priorities); // Hacks to make sure that Rust doesn't make this memory crab when vulkan still needs it.
        }

        queue_create_infos
    }

    fn fulfill_queue_requests(&mut self, device: &Device) {
        let queue_family_count: usize = self.queue_family_properties.len();
        let max_queue_count: usize = self
            .queue_family_properties
            .iter()
            .map(|queue_family_properties| queue_family_properties.queue_count)
            .max()
            .unwrap_or(0) as usize;

        let mut requests: Vec<Vec<Option<Arc<VulkanQueue>>>> = vec![vec![None; max_queue_count as usize]; queue_family_count];

        for request in self.queue_requests.iter_mut() {
            let family = request.requested_family as usize;
            let index = request.assigned_index as usize;

            if requests[family][index].is_none() {
                requests[family][index] = Some(Arc::new(VulkanQueue {
                    queue: Mutex::new(unsafe { device.get_device_queue(family as u32, index as u32) }),
                    family: family as i32,
                }));
            }

            request.queue = requests[family][index].as_ref().cloned();
        }
    }
}

/// Builds all information about features on the device and what is enabled.
impl DeviceFeatureBuilder {
    pub fn new(vk_api_version: u32) -> DeviceFeatureBuilder {
        DeviceFeatureBuilder {
            vulkan_features: PhysicalDeviceFeatures2::default(),
            vulkan_11_features: if vk_api_version >= API_VERSION_1_1 {
                Some(PhysicalDeviceVulkan11Features::default())
            } else {
                None
            },
            vulkan_12_features: if vk_api_version >= API_VERSION_1_2 {
                Some(PhysicalDeviceVulkan12Features::default())
            } else {
                None
            },
        }
    }
}

impl QueueRequest {
    pub fn new(family: i32) -> QueueRequest {
        QueueRequest {
            requested_family: family,
            assigned_index: 0,
            queue: None,
        }
    }
}

impl VulkanQueue {
    pub fn access_queue(&self) -> &Mutex<Queue> {
        &self.queue
    }

    pub fn queue_submit(&self, device: ash::Device, submits: &[SubmitInfo], fence: Fence) -> VkResult<()> {
        let guard = self.queue.lock().unwrap();
        unsafe { device.queue_submit(*guard, submits, fence) }
    }

    pub fn queue_bind_sparse(&self, device: ash::Device, submits: &[BindSparseInfo], fence: Fence) -> VkResult<()> {
        let guard = self.queue.lock().unwrap();
        unsafe { device.queue_bind_sparse(*guard, submits, fence) }
    }

    pub fn queue_present_khr(&self, swapchain: Swapchain, present_info: &PresentInfoKHR) -> VkResult<bool> {
        let guard = self.queue.lock().unwrap();
        unsafe { swapchain.queue_present(*guard, present_info) }
    }
}


pub enum DeviceCreateError {
    VulkanError(vk::Result),
    RequiredFeatureNotSupported(NamedUUID),
    Utf8Error(std::str::Utf8Error),
    NulError(std::ffi::NulError),
    ExtensionNotSupported,
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DeviceFeatureState {
    Uninitialized,
    Initialized,
    Enabled,
    Disabled
}

struct FeatureInfo {
    feature: Box<dyn ApplicationDeviceFeatureInstance>,
    state: DeviceFeatureState,
    name: NamedUUID,
    required: bool,
}

impl Feature for FeatureInfo {
    type State = DeviceFeatureState;

    fn get_payload(&self, pass_state: &Self::State) -> Option<&dyn Any> {
        todo!()
    }

    fn get_payload_mut(&mut self, pass_state: &Self::State) -> Option<&mut dyn Any> {
        todo!()
    }
}

struct DeviceBuilder {
    processor: FeatureProcessor<FeatureInfo>,
    instance: InstanceContext,
    physical_device: vk::PhysicalDevice,
    info: Option<DeviceInfo>,
    config: Option<DeviceConfigurator>,
}

impl DeviceBuilder {
    fn new(instance: InstanceContext, physical_device: vk::PhysicalDevice, order: Box<[NamedUUID]>, features: Vec<(NamedUUID, Box<dyn ApplicationDeviceFeatureInstance>, bool)>) -> Self {
        let processor = FeatureProcessor::new(features.into_iter().map(
            |(name, feature, required)|
                (name.get_uuid(),
                 FeatureInfo {
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

    fn run_init_pass(&mut self) -> Result<(), DeviceCreateError> {
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
                    InitResult::Ok => feature.state = DeviceFeatureState::Initialized,
                    InitResult::Disable => {
                        feature.state = DeviceFeatureState::Disabled;
                        if feature.required {
                            return Err(DeviceCreateError::RequiredFeatureNotSupported(feature.name.clone()))
                        }
                    }
                }
                Ok(())
            }
        )?;

        Ok(())
    }

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
}

enum PNextVariant {
    VkPhysicalDeviceVulkan1_1Features(&'static vk::PhysicalDeviceVulkan11Features),
    VkPhysicalDeviceVulkan1_2Features(&'static vk::PhysicalDeviceVulkan12Features),
    VkPhysicalDeviceVulkan1_1Properties(&'static vk::PhysicalDeviceVulkan11Properties),
    VkPhysicalDeviceVulkan1_2Properties(&'static vk::PhysicalDeviceVulkan12Properties),
}

struct PNextIterator {
    current: *const c_void,
}

impl PNextIterator {
    unsafe fn new(initial: *const c_void) -> Self {
        Self { current: initial }
    }
}

impl Iterator for PNextIterator {
    type Item = PNextVariant;

    fn next(&mut self) -> Option<Self::Item> {
        #[repr(C)]
        struct RawStruct {
            pub s_type: vk::StructureType,
            pub p_next: *const c_void,
        }

        // Iterate until we find a struct that we know
        while !self.current.is_null() {
            let current = self.current;
            let raw = unsafe { current.cast::<RawStruct>().read() };
            self.current = raw.p_next;

            match raw.s_type {
                vk::StructureType::PHYSICAL_DEVICE_VULKAN_1_1_FEATURES => {
                    return Some(PNextVariant::VkPhysicalDeviceVulkan1_1Features(unsafe {
                        current.cast::<vk::PhysicalDeviceVulkan11Features>().as_ref().unwrap()
                    }));
                }

                vk::StructureType::PHYSICAL_DEVICE_VULKAN_1_2_FEATURES => {
                    return Some(PNextVariant::VkPhysicalDeviceVulkan1_2Features(unsafe {
                        current.cast::<vk::PhysicalDeviceVulkan12Features>().as_ref().unwrap()
                    }));
                }

                vk::StructureType::PHYSICAL_DEVICE_VULKAN_1_1_PROPERTIES => {
                    return Some(PNextVariant::VkPhysicalDeviceVulkan1_1Properties(unsafe {
                        current.cast::<vk::PhysicalDeviceVulkan11Properties>().as_ref().unwrap()
                    }));
                }

                vk::StructureType::PHYSICAL_DEVICE_VULKAN_1_2_PROPERTIES => {
                    return Some(PNextVariant::VkPhysicalDeviceVulkan1_2Properties(unsafe {
                        current.cast::<vk::PhysicalDeviceVulkan12Properties>().as_ref().unwrap()
                    }));
                }

                _ => {}
            }
        }

        // No more structs to process
        None
    }
}

pub struct QueueFamilyInfo {
    index: u32,
    properties: vk::QueueFamilyProperties,
}

impl QueueFamilyInfo {
    fn new(index: u32, properties: vk::QueueFamilyProperties) -> Self {
        Self {
            index,
            properties,
        }
    }

    fn new2(index: u32, properties2: vk::QueueFamilyProperties2) -> Self {
        let properties = properties2.queue_family_properties;

        for variant in unsafe { PNextIterator::new(properties2.p_next) } {
            match variant {
                _ => {}
            }
        }

        Self {
            index,
            properties,
        }
    }

    pub fn get_index(&self) -> u32 {
        self.index
    }

    pub fn get_properties(&self) -> &vk::QueueFamilyProperties {
        &self.properties
    }
}

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

        let queue_families;

        let vk_1_1 = instance.get_version().is_supported(VulkanVersion::VK_1_1);
        let get_physical_device_properties_2 = instance.get_extension::<ash::extensions::khr::GetPhysicalDeviceProperties2>();

        if vk_1_1 || get_physical_device_properties_2.is_some() {
            // Use the newer VK_KHR_get_physical_device_properties2 functions

            let mut features2 = vk::PhysicalDeviceFeatures2::default();
            if vk_1_1 {
                unsafe { instance.vk().get_physical_device_features2(physical_device, &mut features2) };
            } else {
                unsafe { get_physical_device_properties_2.unwrap().get_physical_device_features2(physical_device, features2.borrow_mut()) };
            }
            let features2 = features2;

            features_1_0 = Some(features2.features);

            for variant in unsafe{ PNextIterator::new(features2.p_next) } {
                match variant {
                    PNextVariant::VkPhysicalDeviceVulkan1_1Features(features) => {
                        let mut tmp = features.clone();
                        tmp.p_next = null_mut();
                        features_1_1 = Some(tmp);
                    }
                    PNextVariant::VkPhysicalDeviceVulkan1_2Features(features) => {
                        let mut tmp = features.clone();
                        tmp.p_next = null_mut();
                        features_1_2 = Some(tmp);
                    }
                    _ => {}
                }
            }

            let mut properties2 = vk::PhysicalDeviceProperties2::default();
            if vk_1_1 {
                unsafe { instance.vk().get_physical_device_properties2(physical_device, &mut properties2) };
            } else {
                unsafe { get_physical_device_properties_2.unwrap().get_physical_device_properties2(physical_device, properties2.borrow_mut()) };
            }
            let properties2 = properties2;

            properties_1_0 = Some(properties2.properties);

            for variant in unsafe{ PNextIterator::new(properties2.p_next) } {
                match variant {
                    PNextVariant::VkPhysicalDeviceVulkan1_1Properties(properties) => {
                        let mut tmp = properties.clone();
                        tmp.p_next = null_mut();
                        properties_1_1 = Some(tmp);
                    }
                    PNextVariant::VkPhysicalDeviceVulkan1_2Properties(properties) => {
                        let mut tmp = properties.clone();
                        tmp.p_next = null_mut();
                        properties_1_2 = Some(tmp);
                    }
                    _ => {}
                }
            }

            let mut memory_properties2 = vk::PhysicalDeviceMemoryProperties2::default();
            if vk_1_1 {
                unsafe { instance.vk().get_physical_device_memory_properties2(physical_device, &mut memory_properties2) };
            } else {
                unsafe { get_physical_device_properties_2.unwrap().get_physical_device_memory_properties2(physical_device, memory_properties2.borrow_mut()) };
            }
            let memory_properties2 = memory_properties2;

            memory_properties_1_0 = Some(memory_properties2.memory_properties);

            for variant in unsafe{ PNextIterator::new(memory_properties2.p_next) } {
                match variant {
                    _ => {}
                }
            }


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
            queue_families: queue_families.unwrap(),
            extensions,
        })
    }

    pub fn get_instance(&self) -> &InstanceContext {
        &self.instance
    }

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

    pub fn get_queue_family_infos(&self) -> &[QueueFamilyInfo] {
        self.queue_families.as_ref()
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

pub struct DeviceConfigurator {
    enabled_extensions: HashMap<UUID, Option<&'static DeviceExtensionLoaderFn>>
}

impl DeviceConfigurator {
    fn new() -> Self {
        Self{
            enabled_extensions: HashMap::new(),
        }
    }

    pub fn enable_extension<EXT: VkExtensionInfo + DeviceExtensionLoader + 'static>(&mut self) {
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

    fn build_device(self, info: &DeviceInfo) -> Result<(ash::Device, ExtensionFunctionSet), DeviceCreateError> {
        let mut extensions = Vec::with_capacity(self.enabled_extensions.len());
        for (uuid, _) in &self.enabled_extensions {
            extensions.push(
                info.get_extension_properties_uuid(uuid)
                    .ok_or(DeviceCreateError::ExtensionNotSupported)?
                    .get_c_name().as_ptr()
            )
        }

        let create_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(extensions.as_slice());

        let device = unsafe {
            info.get_instance().vk().create_device(info.physical_device, &create_info.build(), None)
        }?;

        let mut function_set = ExtensionFunctionSet::new();
        for (_, extension) in &self.enabled_extensions {
            if let Some(extension) = extension {
                extension(&mut function_set, info.get_instance().get_entry(), info.get_instance().vk(), &device);
            }
        }

        Ok((device, function_set))
    }
}