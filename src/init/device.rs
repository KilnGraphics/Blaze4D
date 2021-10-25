use std::cmp::{min, Ordering};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::os::raw::c_char;
use std::sync::Arc;

use ash::extensions::khr::Swapchain;
use ash::prelude::VkResult;
use ash::vk::{
    BindSparseInfo, DeviceCreateInfo, DeviceQueueCreateInfo, ExtensionProperties, Fence, PhysicalDevice, PhysicalDeviceFeatures2,
    PhysicalDeviceProperties, PhysicalDeviceType, PhysicalDeviceVulkan11Features, PhysicalDeviceVulkan12Features, PresentInfoKHR, Queue,
    QueueFamilyProperties, StructureType, SubmitInfo, API_VERSION_1_1, API_VERSION_1_2,
};
use ash::{Device, Instance};

use crate::utils::string_from_array;

/// Utility class to quickly identify and compare entities while retaining a human readable name.
///
/// comparing existing ID's is very fast so it is highly
/// recommended to avoid creating new instances when not necessary. (Also reduces typing mistakes)
#[derive(Clone, Debug)]
pub struct NamedID {
    pub name: String,
    id: u32,
}

#[derive(Clone, Debug)]
pub struct VulkanQueue {
    queue: Queue,
    family: i32,
}

struct QueueRequest {
    requested_family: i32,
    assigned_index: i32,
    queue: Arc<VulkanQueue>,
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
    fn get_dependencies(&self) -> HashSet<NamedID>;
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

pub struct DeviceBuilder {
    pub(crate) instance: Instance,
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

impl DeviceBuilder {
    pub fn build(&mut self, required_features: &mut HashSet<NamedID>) -> RosellaDevice {
        let mut devices: Vec<DeviceMeta> = vec![];
        let raw_devices = unsafe { self.instance.enumerate_physical_devices() }.expect("Failed to find devices.");

        for physical_device in raw_devices.iter() {
            let mut meta = DeviceMeta::new(&self.instance, *physical_device, required_features, vec![]);
            meta.process_support();
            devices.push(meta)
        }

        //TODO: Sorting
        devices
            .get_mut(0)
            .expect("No suitable devices where found.")
            .create_device(&self.instance)
    }
}

impl DeviceMeta {
    pub fn new(
        instance: &Instance,
        physical_device: PhysicalDevice,
        required_features: &mut HashSet<NamedID>,
        application_features: Vec<Box<dyn ApplicationFeature>>,
    ) -> DeviceMeta {
        let mut features = HashMap::new();
        for feature in application_features {
            features.insert(feature.get_feature_name(), feature);
        }

        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };

        let mut feature_builder = DeviceFeatureBuilder::new(device_properties.api_version);
        unsafe {
            feature_builder.vulkan_features.features = instance.get_physical_device_features(physical_device);
        }
        let mut queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
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

    pub fn create_device(&mut self, instance: &Instance) -> RosellaDevice {
        assert!(!self.building);
        self.building = true;

        for feature in self.features.values() {
            if feature.is_supported(self) {
                feature.enable(self);
            }
        }

        let device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&self.generate_queue_mappings())
            .enabled_extension_names(&self.generate_enabled_extension_names())
            .push_next(&mut self.feature_builder.vulkan_features)
            .build();

        let vk_device = unsafe {
            instance
                .create_device(self.physical_device, &device_create_info, None)
                .expect("Failed to create the VkDevice!")
        };

        self.fulfill_queue_requests(&vk_device);

        RosellaDevice { device: vk_device }
    }

    fn generate_queue_mappings(&mut self) -> Vec<DeviceQueueCreateInfo> {
        let mut next_queue_indices = vec![0; self.queue_family_properties.len()];

        for mut request in self.queue_requests.iter_mut() {
            let requested_family = request.requested_family as usize;
            let index_requests = next_queue_indices[requested_family];
            let index: u32 = index_requests as u32;
            next_queue_indices[requested_family] += 1;

            request.assigned_index = (index % self.queue_family_properties[requested_family].queue_count) as i32;
        }

        let family_count = next_queue_indices.iter().filter(|&&x| x > 0).count();

        let mut queue_create_infos = vec![DeviceQueueCreateInfo::default(); family_count];

        for family in 0..next_queue_indices.len() {
            if next_queue_indices[family] == 0 {
                continue;
            }

            let priorities = vec![
                1.0;
                min(
                    next_queue_indices[family],
                    self.queue_family_properties[family].queue_count as usize,
                )
            ];

            let info = &mut queue_create_infos[family];
            info.s_type = StructureType::DEVICE_QUEUE_CREATE_INFO;
            info.queue_family_index = family as u32;
            info.p_queue_priorities = priorities.as_ptr();
        }

        queue_create_infos
    }

    fn generate_enabled_extension_names(&self) -> Vec<*const c_char> {
        if self.enabled_extensions.is_empty() {
            return Vec::new();
        }

        let mut names = Vec::with_capacity(self.enabled_extensions.capacity());

        for name in self.enabled_extensions.iter() {
            names.push(name.as_ptr() as *const c_char);
        }

        names
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
                    queue: unsafe { device.get_device_queue(family as u32, index as u32) },
                    family: family as i32,
                }));
            }

            request.queue = requests[family][index]
                .as_ref()
                .unwrap_or_else(|| panic!("Queue exists for family: {} and index: {}", family, index))
                .clone();
        }
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
                _ => None,
            },
            vulkan_12_features: match vk_api_version {
                API_VERSION_1_2 => Some(PhysicalDeviceVulkan12Features::default()),
                _ => None,
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
        unsafe { device.queue_submit(self.queue, submits, fence) }
    }

    pub fn queue_bind_sparse(&self, device: ash::Device, submits: &[BindSparseInfo], fence: Fence) -> VkResult<()> {
        unsafe { device.queue_bind_sparse(self.queue, submits, fence) }
    }

    pub fn queue_present_khr(&self, swapchain: Swapchain, present_info: &PresentInfoKHR) -> VkResult<bool> {
        unsafe { swapchain.queue_present(self.queue, present_info) }
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
