use std::collections::HashMap;

use crate::init::application_feature::{ApplicationDeviceFeature, ApplicationInstanceFeature};

use crate::{ NamedUUID, UUID };

///
/// A class used to collect any callbacks and settings that are used for device and instance initialization.
///
pub struct InitializationRegistry {
    instance_features: HashMap<UUID, (NamedUUID, Box<[NamedUUID]>, Box<dyn ApplicationInstanceFeature>, bool)>,
    device_features: HashMap<UUID, (NamedUUID, Box<[NamedUUID]>, Box<dyn ApplicationDeviceFeature>, bool)>,
}

impl InitializationRegistry {
    pub fn new() -> Self {
        InitializationRegistry {
            instance_features: HashMap::new(),
            device_features: HashMap::new(),
        }
    }

    pub fn register_instance_feature(&mut self, name: NamedUUID, dependencies: Box<[NamedUUID]>, feature: Box<dyn ApplicationInstanceFeature>, required: bool) {
        if self.instance_features.insert(name.get_uuid(), (name, dependencies, feature, required)).is_some() {
            panic!("Feature is already present in registry");
        }
    }

    pub fn register_device_feature(&mut self, name: NamedUUID, dependencies: Box<[NamedUUID]>, feature: Box<dyn ApplicationDeviceFeature>, required: bool) {
        if self.device_features.insert(name.get_uuid(), (name, dependencies, feature, required)).is_some() {
            panic!("Feature is already present in registry");
        }
    }

    pub(super) fn take_instance_features(&mut self) -> Vec<(NamedUUID, Box<[NamedUUID]>, Box<dyn ApplicationInstanceFeature>, bool)> {
        let features = std::mem::replace(&mut self.instance_features, HashMap::new());
        features.into_values().collect()
    }

    pub(super) fn take_device_features(&mut self) -> Vec<(NamedUUID, Box<[NamedUUID]>, Box<dyn ApplicationDeviceFeature>, bool)> {
        let features = std::mem::replace(&mut self.device_features, HashMap::new());
        features.into_values().collect()
    }
}

impl Default for InitializationRegistry {
    fn default() -> Self {
        Self::new()
    }
}