use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::iter::Map;
use ash::{Entry, Instance};
use ash::vk::{PhysicalDevice, PhysicalDeviceFeatures2, PhysicalDeviceProperties, PhysicalDeviceVulkan11Features, PhysicalDeviceVulkan12Features, API_VERSION_1_1, API_VERSION_1_2, ExtensionProperties, QueueFamilyProperties};

/// Utility class to quickly identify and compare entities while retaining a human readable name.
///
/// comparing existing ID's is very fast so it is highly
/// recommended to avoid creating new instances when not necessary. (Also reduces typing mistakes)
#[derive(Clone, Debug)]
pub struct NamedID {
    name: String,
    id: u32,
}

struct QueueRequest {
    requested_family: i128
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
    fn enable(&self); //TODO: DeviceBuildConfigurator
}

struct VulkanInstance {
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
    queue_family_properties: QueueFamilyProperties, // TODO LOW_PRIORITY: look at QueueFamilyProperties2

    building: bool,
    queue_requests: Vec<QueueRequest>,
    enabled_extensions: Vec<String>,
}

struct Device {
    application_features: Vec<Box<dyn ApplicationFeature>>,
    required_features: Vec<Box<dyn ApplicationFeature>>,
    instance: VulkanInstance,
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

        let feature_builder = DeviceFeatureBuilder::new(device_properties.api_version);

        DeviceMeta {
            unsatisfied_requirements: vec![],
            features,
            feature_builder,
            physical_device,
            properties: device_properties,
            extension_properties: HashMap::new(),
            queue_family_properties: Default::default(),
            building: false,
            queue_requests: vec![],
            enabled_extensions: vec![],
        }
    }
}

impl Device {}

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