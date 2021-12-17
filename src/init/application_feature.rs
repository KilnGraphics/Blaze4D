use std::any::Any;
use crate::init::device;
use crate::NamedUUID;

pub enum FeatureDependency {
    /// A strong dependency prevents a feature from being processed if it is not met.
    Strong(NamedUUID),

    /// A weak dependency only guarantees that the dependency is processed before this feature.
    /// But this feature will be processed even if the dependency is not met.
    Weak(NamedUUID),
}

pub enum InitResult {
    Ok,
    Disable,
}

pub trait ApplicationDeviceFeature {
    type Instance: ApplicationDeviceFeatureInstance;

    fn get_name(&self) -> NamedUUID;

    fn get_dependencies(&self) -> &[FeatureDependency];

    fn make_instance(&self) -> Box<Self::Instance>;
}

pub trait ApplicationDeviceFeatureInstance : Send {
    fn init(&mut self, features: &device::DeviceFeatureSet, info: &device::DeviceInfo) -> InitResult;

    fn enable(&mut self, features: &device::DeviceFeatureSet, info: &device::DeviceInfo, config: &device::DeviceConfigurator);

    fn get_data(&self) -> Box<dyn Any>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}