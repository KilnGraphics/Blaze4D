use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use ash::vk::{API_VERSION_1_0, API_VERSION_1_2};
use topological_sort::TopologicalSort;

use crate::init::device::ApplicationFeature;
use crate::NamedUUID;

///
/// A class used to collect any callbacks and settings that are used for device and instance initialization.
///
pub struct InitializationRegistry {
    pub min_required_version: u32,
    pub max_supported_version: u32,

    pub required_instance_extensions: HashSet<String>,
    pub optional_instance_extensions: HashSet<String>,
    pub required_instance_layers: HashSet<String>,
    pub optional_instance_layers: HashSet<String>,

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
            min_required_version: API_VERSION_1_0,
            max_supported_version: API_VERSION_1_2,
            required_instance_layers: HashSet::new(),
            optional_instance_layers: HashSet::new(),
            required_instance_extensions: HashSet::new(),
            optional_instance_extensions: HashSet::new(),
            features: HashMap::new(),
            required_features: HashSet::new(),
        }
    }

    pub fn add_required_instance_layer(&mut self, layer: String) {
        self.required_instance_layers.insert(layer);
    }

    pub fn add_optional_instance_layer(&mut self, layer: String) {
        self.optional_instance_layers.insert(layer);
    }

    pub fn add_required_instance_extension(&mut self, extension: String) {
        self.required_instance_extensions.insert(extension);
    }

    pub fn add_optional_instance_extension(&mut self, extension: String) {
        self.optional_instance_extensions.insert(extension);
    }

    pub fn set_minimum_vulkan_version(&mut self, version: u32) {
        if version > self.min_required_version {
            self.min_required_version = version;
        }
    }

    pub fn set_maximum_vulkan_version(&mut self, version: u32) {
        if version > self.max_supported_version {
            self.max_supported_version = version;
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
}

impl Default for InitializationRegistry {
    fn default() -> Self {
        Self::new()
    }
}
