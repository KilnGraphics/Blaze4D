use std::any::Any;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use ash::extensions::khr::Swapchain;
use ash::prelude::VkResult;

use ash::vk;
use crate::init::application_feature::{ApplicationDeviceFeatureInstance, InitResult};

use crate::init::initialization_registry::InitializationRegistry;
use crate::init::utils::{ExtensionProperties, Feature, FeatureProcessor};
use crate::{NamedUUID, UUID};
use crate::util::extensions::{DeviceExtensionLoader, DeviceExtensionLoaderFn, ExtensionFunctionSet, VkExtensionInfo};
use crate::rosella::{DeviceContext, InstanceContext, VulkanVersion};

struct VulkanQueueImpl {
    queue: Mutex<vk::Queue>,
    family: u32,
}

#[derive(Clone)]
pub struct VulkanQueue(Arc<VulkanQueueImpl>);

impl VulkanQueue {
    fn new(queue: vk::Queue, family: u32) -> Self {
        Self(Arc::new(VulkanQueueImpl{ queue: Mutex::new(queue), family }))
    }

    pub fn get_family(&self) -> u32 {
        self.0.family
    }

    pub fn access_queue(&self) -> &Mutex<vk::Queue> {
        &self.0.queue
    }

    pub fn queue_submit(&self, device: ash::Device, submits: &[vk::SubmitInfo], fence: vk::Fence) -> VkResult<()> {
        let guard = self.0.queue.lock().unwrap();
        unsafe { device.queue_submit(*guard, submits, fence) }
    }

    pub fn queue_bind_sparse(&self, device: ash::Device, submits: &[vk::BindSparseInfo], fence: vk::Fence) -> VkResult<()> {
        let guard = self.0.queue.lock().unwrap();
        unsafe { device.queue_bind_sparse(*guard, submits, fence) }
    }

    pub fn queue_present_khr(&self, swapchain: Swapchain, present_info: &vk::PresentInfoKHR) -> VkResult<bool> {
        let guard = self.0.queue.lock().unwrap();
        unsafe { swapchain.queue_present(*guard, present_info) }
    }
}

#[derive(Debug)]
pub enum DeviceCreateError {
    VulkanError(vk::Result),
    RequiredFeatureNotSupported(NamedUUID),
    Utf8Error(std::str::Utf8Error),
    NulError(std::ffi::NulError),
    ExtensionNotSupported,
    NoSuitableDeviceFound,
}

pub fn create_device(registry: &mut InitializationRegistry, instance: InstanceContext) -> Result<DeviceContext, DeviceCreateError> {
    let (graph, features) : (Vec<_>, Vec<_>) = registry.take_device_features().into_iter().map(
        |(name, dependencies, feature, required)| {
            ((name.clone(), dependencies), (name, feature, required))
        }).unzip();

    let mut topo_sort = topological_sort::TopologicalSort::new();
    for (node, dependencies) in graph {
        for dependency in dependencies.iter() {
            topo_sort.add_dependency(dependency.clone(), node.clone());
        }
        topo_sort.insert(node);
    }
    let ordering : Vec<_> = topo_sort.collect();

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
                    InitResult::Ok => feature.state = DeviceFeatureState::Initialized,
                    InitResult::Disable => {
                        feature.state = DeviceFeatureState::Disabled;
                        log::debug!("Disabled feature {:?}", feature.name);
                        if feature.required {
                            log::warn!("Failed to initialize required feature {:?}", feature.name);
                            return Err(DeviceCreateError::RequiredFeatureNotSupported(feature.name.clone()))
                        }
                    }
                }
                log::debug!("Initialized feature {:?}", feature.name);
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

    fn build(self) -> Result<DeviceContext, DeviceCreateError> {
        let info = self.info.expect("Called build but info is none");
        let (device, function_set) = self.config.expect("Called build but config is none")
            .build_device(&info)?;

        Ok(DeviceContext::new(self.instance, device, self.physical_device, function_set))
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

struct QueueRequestImpl {
    result: Option<VulkanQueue>,
}

impl QueueRequestImpl {
    fn new(family: u32) -> (QueueRequest2, QueueRequestResolver) {
        let cell = Rc::new(RefCell::new(QueueRequestImpl{ result: None }));
        (QueueRequest2(cell.clone()), QueueRequestResolver{ request: cell, family, index: None })
    }
}

pub struct QueueRequest2(Rc<RefCell<QueueRequestImpl>>);

impl QueueRequest2 {
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
}

impl DeviceConfigurator {
    fn new() -> Self {
        Self{
            enabled_extensions: HashMap::new(),
            queue_requests: Vec::new(),
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

    pub fn add_queue_request(&mut self, family: u32) -> QueueRequest2 {
        let (request, resolver) = QueueRequestImpl::new(family);
        self.queue_requests.push(resolver);
        request
    }

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