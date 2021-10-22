use core::mem;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::iter::{FromIterator, Map};
use std::os::raw::c_char;
use ash::{Device, Entry, Instance};
use ash::extensions::khr::Swapchain;
use ash::prelude::VkResult;
use ash::vk::{PhysicalDevice, PhysicalDeviceFeatures2, PhysicalDeviceProperties, PhysicalDeviceVulkan11Features, PhysicalDeviceVulkan12Features, API_VERSION_1_1, API_VERSION_1_2, ExtensionProperties, QueueFamilyProperties, Queue, SubmitInfo, Fence, BindSparseInfo, PresentInfoKHR, PhysicalDeviceType, DeviceCreateInfo, DeviceQueueCreateInfo};
use crate::rosella::utils::string_from_array;

/// Utility class to quickly identify and compare entities while retaining a human readable name.
///
/// comparing existing ID's is very fast so it is highly
/// recommended to avoid creating new instances when not necessary. (Also reduces typing mistakes)
#[derive(Clone, Debug)]
pub struct NamedID {
    name: String,
    id: u32,
}

pub struct VulkanQueue {
    queue: Queue,
    family: i32,
}

struct QueueRequest {
    requested_family: i32,
    assigned_index: i32,

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
    fn get_feature_name(&self) -> NamedID;
    fn is_supported(&self, meta: &DeviceMeta) -> bool;
    fn enable(&self, meta: &DeviceMeta);
}

pub struct VulkanInstance {
    instance: Instance,
    version: u32,
}

/// Builds all information about features on the device and what is enabled.
/// TODO LOW_PRIORITY: add support for VK1.0 by not doing any of this on vk1.0. instead just create a simple VkPhysicalDeviceFeatures field
pub struct DeviceFeatureBuilder {
    vulkan_features: PhysicalDeviceFeatures2,
    vulkan_11_features: Option<PhysicalDeviceVulkan11Features>,
    vulkan_12_features: Option<PhysicalDeviceVulkan12Features>,
}

pub struct DeviceMeta {
    unsatisfied_requirements: Vec<NamedID>,
    features: HashMap<NamedID, Box<dyn ApplicationFeature>>,
    feature_builder: DeviceFeatureBuilder,

    physical_device: PhysicalDevice,
    properties: PhysicalDeviceProperties,
    extension_properties: HashMap<String, ExtensionProperties>,
    queue_family_properties: Vec<QueueFamilyProperties>, // TODO LOW_PRIORITY: look at QueueFamilyProperties2

    building: bool,
    queue_requests: Vec<QueueRequest>,
    enabled_extensions: Vec<String>,
}

pub struct RosellaDevice {
    device: Device,
}

impl DeviceMeta {
    pub fn new(instance: Instance, physical_device: PhysicalDevice, required_features: &mut Vec<NamedID>, application_features: Vec<Box<dyn ApplicationFeature>>) -> DeviceMeta {
        let mut unsatisfied_requirements: Vec<NamedID> = vec![];
        unsatisfied_requirements.append(required_features);

        let mut features = HashMap::new();
        for feature in application_features {
            features.insert(feature.get_feature_name(), feature);
        }

        let device_properties = unsafe {
            instance.get_physical_device_properties(physical_device)
        };

        let mut feature_builder = DeviceFeatureBuilder::new(device_properties.api_version);
        unsafe { feature_builder.vulkan_features.features = instance.get_physical_device_features(physical_device); }
        let mut queue_family_properties = vec![];
        unsafe { queue_family_properties = instance.get_physical_device_queue_family_properties(physical_device); }
        let mut extension_properties = HashMap::new();
        unsafe {
            for extension_property in instance.enumerate_device_extension_properties(physical_device).unwrap() {
                extension_properties.insert(string_from_array(&extension_property.extension_name), extension_property);
            }
        }

        DeviceMeta {
            unsatisfied_requirements: vec![],
            features,
            feature_builder,
            physical_device,
            properties: device_properties,
            extension_properties,
            queue_family_properties,
            building: false,
            queue_requests: vec![],
            enabled_extensions: vec![],
        }
    }

    fn process_support(&mut self) {
        self.unsatisfied_requirements.clear();
        for feature in self.features.values() {
            if !feature.is_supported(&self) {
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
            _ => 0
        }
    }

    pub fn create_device(&mut self, instance: Instance) -> RosellaDevice {
        assert!(!self.building);
        self.building = true;

        for feature in self.features.values() {
            if feature.is_supported(&self) {
                feature.enable(&self);
            }
        }

        let device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(self.generate_queue_mappings())
            .enabled_extension_names(self.generate_enabled_extension_names())
            .push_next(&mut self.feature_builder.vulkan_features)
            .build();

        unsafe {
            RosellaDevice {
                device: instance.create_device(self.physical_device, &device_create_info, None).expect("Failed to create the VkDevice!")
            }
        }
    }

    fn generate_queue_mappings(&self) -> &[DeviceQueueCreateInfo] {
        todo!("Generate Queue Mappings")
    }

    fn generate_enabled_extension_names(&self) -> &[*const c_char] {
        todo!("Generate Enabled Extension Names")
    }
}

impl Drop for RosellaDevice {
    fn drop(&mut self) {
        todo!("DROP")
    }
}

impl RosellaDevice {}

/// Builds all information about features on the device and what is enabled.
impl DeviceFeatureBuilder {
    pub fn new(vk_api_version: u32) -> DeviceFeatureBuilder {
        DeviceFeatureBuilder {
            vulkan_features: PhysicalDeviceFeatures2::default(),
            vulkan_11_features: match vk_api_version {
                API_VERSION_1_1 => Some(PhysicalDeviceVulkan11Features::default()),
                _ => None
            },
            vulkan_12_features: match vk_api_version {
                API_VERSION_1_2 => Some(PhysicalDeviceVulkan12Features::default()),
                _ => None
            },
        }
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanQueue {
    pub fn queue_submit(&self, device: ash::Device, submits: &[SubmitInfo], fence: Fence) -> VkResult<()> {
        unsafe { return device.queue_submit(self.queue, submits, fence); }
    }

    pub fn queue_bind_sparse(&self, device: ash::Device, submits: &[BindSparseInfo], fence: Fence) -> VkResult<()> {
        unsafe { return device.queue_bind_sparse(self.queue, submits, fence); }
    }

    pub fn queue_present_khr(&self, swapchain: Swapchain, present_info: &PresentInfoKHR) -> VkResult<bool> {
        unsafe { return swapchain.queue_present(self.queue, present_info); }
    }
}

impl PartialEq<Self> for NamedID {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for NamedID {}

impl PartialOrd<Self> for NamedID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for NamedID {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Hash for NamedID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}