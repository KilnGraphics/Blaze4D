use std::cmp::Ordering;
use std::iter::Map;
use ash::{Entry, Instance};
use ash::vk::{PhysicalDevice, PhysicalDeviceProperties};

/// Utility class to quickly identify and compare entities while retaining a human readable name.
///
/// comparing existing ID's is very fast so it is highly
/// recommended to avoid creating new instances when not necessary. (Also reduces typing mistakes)
#[derive(Copy, Debug)]
pub struct NamedID {
    name: String,
    id: u32,
}

///
///<p>A class that represents some collection of device features or capabilities.</p>
///
///<p>Instances of this class can be registered into a FIXME {@link graphics.kiln.rosella.init.InitializationRegistry} which will then be
///used to select and initialize a device.</p>
///
///<p>This happens in 2 stages.</p>
///<ol>
///    <li>The feature is queried if the device supports the feature.</li>
///    <li>If support is detected and desired the feature will be called to configure the device.</li>
///</ol>
///<p>For these interactions a instance of FIXME {@link graphics.kiln.rosella.init.DeviceBuilder.DeviceMeta} is provided which manages
///information for a single physical device.</p>
///
///<p>Since multiple devices may be tested concurrently the createInstance function will be called for each device which
///should return a object that can keep track of all necessary metadata it may need for one device. The ApplicationFeature
///class as well as separate created instances may be called concurrently, however created instances individually will
///never be called concurrently.</p>
///
///<p>If the feature wants to return information to the application it can provide a metadata object which will be stored
///in the created device for the application to access.</p>
///
///<p>A feature can access the instances of other features, however it must make sure to declare dependencies as otherwise
///those features may not have run yet.</p>
///
///<p>The default implementation of this class only validates that all dependencies are met and does not create any metadata.</p>
///
pub trait ApplicationFeature {
    fn get_feature_name(&self) -> &str;
    fn is_supported(&self, meta: &DeviceMeta) -> bool;
    fn enable(&self); //TODO: DeviceBuildConfigurator
}

struct VulkanInstance {
    instance: Instance,
    version: VulkanVersion,
}

/// Builds all information about features on the device and what is enabled.
struct DeviceFeatureBuilder {}

pub struct DeviceMeta {
    unsatisfied_requirements: Vec<NamedID>,
    features: Map<NamedID, dyn ApplicationFeature>,
    sorted_features: Vec<dyn ApplicationFeature>,

    physical_device: PhysicalDevice,
    properties: PhysicalDeviceProperties,

}

struct Device {
    application_features: Vec<dyn ApplicationFeature>,
    required_features: Vec<dyn ApplicationFeature>,
    instance: VulkanInstance,
}

impl Device {}

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