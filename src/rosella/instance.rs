use ash::{Entry, Instance};

/// Represents a version of Vulkan with extra data such as the major, minor, and patch.
struct VulkanVersion {
    major: i64,
    minor: i64,
    patch: i64,
}

struct ApplicationFeature {}

struct VulkanInstance {
    instance: Instance,
    version: VulkanVersion,
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

struct Device {
    application_features: Vec<ApplicationFeature>,
    required_features: Vec<ApplicationFeature>,
}