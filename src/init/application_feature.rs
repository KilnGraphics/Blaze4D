use std::any::Any;
use crate::init::{device, instance};
use crate::init::utils::FeatureAccess;
use crate::NamedUUID;


/// Common functions requires by all features
pub trait FeatureBase {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Represents the result of a init operation of a feature
pub enum InitResult {
    /// Indicates that the feature is supported and can be enabled
    Ok,

    /// Indicates that the feature is not supported and must be disabled
    Disable,
}

/// A feature that controls instance initialization
///
/// See [`crate::init::instance`] for more information.
pub trait ApplicationInstanceFeature : FeatureBase {

    /// Tests if the feature is supported
    fn init(&mut self, features: &mut dyn FeatureAccess, info: &instance::InstanceInfo) -> InitResult;

    /// Configures the instance
    fn enable(&mut self, features: &mut dyn FeatureAccess, info: &instance::InstanceInfo, config: &mut instance::InstanceConfigurator);

    /// Performs any necessary post creation steps and generates the data that is sent back to the application
    fn finish(self, _: &ash::Instance) -> Option<Box<dyn Any>> where Self: Sized {
        None
    }
}

pub trait ApplicationDeviceFeature {
    fn make_instance(&self) -> Box<dyn ApplicationDeviceFeatureInstance>;
}

pub trait ApplicationDeviceFeatureInstance : Send + FeatureBase {
    fn init(&mut self, features: &mut dyn FeatureAccess, info: &device::DeviceInfo) -> InitResult;

    fn enable(&mut self, features: &mut dyn FeatureAccess, info: &device::DeviceInfo, config: &mut device::DeviceConfigurator);
}