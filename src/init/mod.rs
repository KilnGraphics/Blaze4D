use crate::init::device::{DeviceInfo, DeviceConfiguration};

pub mod device;
pub mod initialization_registry;
pub mod instance_builder;
mod capability;


pub trait ApplicationFeatureInstance {
    type ResultData;

    fn init(&self, info: &DeviceInfo) -> Result<(), ()>;

    fn configure(&self, info: &DeviceInfo, config: &mut DeviceConfiguration) -> Result<(), ()>;

    fn collapse(self) -> Box<Self::ResultData>;
}