use std::any::Any;
use crate::init::{device, instance};
use crate::init::utils::FeatureAccess;
use crate::NamedUUID;

pub enum FeatureDependency {
    /// A strong dependency prevents a feature from being processed if it is not met.
    Strong(NamedUUID),

    /// A weak dependency only guarantees that the dependency is processed before this feature.
    /// But this feature will be processed even if the dependency is not met.
    Weak(NamedUUID),
}

pub trait FeatureBase {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_data(&self) -> Box<dyn Any>;
}

pub enum InitResult {
    Ok,
    Disable,
}

pub trait ApplicationInstanceFeature : FeatureBase {
    fn init(&mut self, features: &mut dyn FeatureAccess, info: &instance::InstanceInfo) -> InitResult;

    fn enable(&mut self, features: &mut dyn FeatureAccess, info: &instance::InstanceInfo, config: &mut instance::InstanceConfigurator);

    fn finish(self, instance: &ash::Instance) -> Option<Box<dyn Any>>;
}

pub trait ApplicationDeviceFeature {
    type Instance: ApplicationDeviceFeatureInstance;

    fn get_name(&self) -> NamedUUID;

    fn get_dependencies(&self) -> &[FeatureDependency];

    fn make_instance(&self) -> Box<Self::Instance>;
}

pub trait ApplicationDeviceFeatureInstance : Send + FeatureBase {
    fn init(&mut self, features: &mut dyn FeatureAccess, info: &device::DeviceInfo) -> InitResult;

    fn enable(&mut self, features: &mut dyn FeatureAccess, info: &device::DeviceInfo, config: &device::DeviceConfigurator);
}