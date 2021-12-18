use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use ash::vk::{API_VERSION_1_0, API_VERSION_1_2};
use topological_sort::TopologicalSort;
use crate::init::application_feature::{ApplicationDeviceFeature, ApplicationInstanceFeature};

use crate::init::device::ApplicationFeature;
use crate::NamedUUID;
use crate::util::id::UUID;

///
/// A class used to collect any callbacks and settings that are used for device and instance initialization.
///
pub struct InitializationRegistry {
    instance_features: HashMap<UUID, (NamedUUID, Box<[NamedUUID]>, Box<dyn ApplicationInstanceFeature>)>,

    pub features: HashMap<NamedUUID, MarkedFeature>,
    pub required_features: HashSet<NamedUUID>,
}

pub struct MarkedFeature {
    pub feature: Rc<dyn ApplicationFeature>,
}

impl MarkedFeature {
    fn new(feature: Rc<dyn ApplicationFeature>) -> Self {
        MarkedFeature { feature }
    }
}

impl InitializationRegistry {
    pub fn new() -> Self {
        InitializationRegistry {
            instance_features: HashMap::new(),
            features: HashMap::new(),
            required_features: HashSet::new(),
        }
    }

    ///
    /// Marks a feature as required. This means that during device selection no device will be used
    /// that does not support all required features.
    ///
    pub fn add_required_application_feature(&mut self, name: NamedUUID) {
        self.required_features.insert(name);
    }

    ///
    /// Registers a application feature into this registry.
    ///
    pub fn register_application_feature(&mut self, feature: Rc<dyn ApplicationFeature>) -> Result<(), String> {
        if self.features.contains_key(&feature.get_feature_name()) {
            return Err(format!("Feature {} is already registered", feature.get_feature_name().get_name()));
        }

        self.features.insert(feature.get_feature_name(), MarkedFeature::new(feature));
        Ok(())
    }

    ///
    /// Topologically sorts all features and returns them as a list.
    /// The list can be iterated from beginning to end to ensure all dependencies are always met.
    ///
    pub fn get_ordered_features(&self) -> Vec<Rc<dyn ApplicationFeature>> {
        let mut sort = TopologicalSort::<NamedUUID>::new();
        self.features.keys().for_each(|feature| self.add_feature(feature, &mut sort));

        let mut sorted = Vec::new();

        while let Some(id) = sort.pop() {
            sorted.push(self.features[&id].feature.clone());
        }

        sorted
    }

    pub fn add_feature(&self, id: &NamedUUID, sort: &mut TopologicalSort<NamedUUID>) {
        for dependency in self.features[id].feature.get_dependencies() {
            sort.add_dependency(id.clone(), dependency);
        }

        sort.insert(id.clone());
    }

    pub fn register_instance_feature(&mut self, name: NamedUUID, dependencies: Box<[NamedUUID]>, feature: Box<dyn ApplicationInstanceFeature>) {
        if self.instance_features.insert(name.get_uuid(), (name, dependencies, feature)).is_some() {
            panic!("Feature is already present in registry");
        }
    }

    pub(super) fn take_instance_features(&mut self) -> Vec<(NamedUUID, Box<[NamedUUID]>, Box<dyn ApplicationInstanceFeature>)> {
        let features = std::mem::replace(&mut self.instance_features, HashMap::new());
        features.into_values().collect()
    }
}

impl Default for InitializationRegistry {
    fn default() -> Self {
        Self::new()
    }
}