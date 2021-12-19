use std::any::Any;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use crate::init::application_feature::FeatureAccess;
use crate::NamedUUID;
use crate::rosella::VulkanVersion;
use crate::util::id::UUID;

#[derive(Clone, Debug)]
pub struct LayerProperties {
    c_name: CString,
    name: String,
    description: String,
    spec_version: VulkanVersion,
    implementation_version: u32,
}

impl LayerProperties {
    pub fn new(src: &ash::vk::LayerProperties) -> Result<Self, std::str::Utf8Error> {
        let c_name = CString::from(
            unsafe{ CStr::from_ptr(src.layer_name.as_ptr()) }
        );
        let name = String::from(c_name.to_str()?);

        let description = String::from(
            unsafe{ CStr::from_ptr(src.description.as_ptr()) }.to_str()?
        );

        Ok(Self{
            c_name,
            name,
            description,
            spec_version: VulkanVersion::from_raw(src.spec_version),
            implementation_version: src.implementation_version,
        })
    }

    pub fn get_c_name(&self) -> &CString {
        &self.c_name
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_description(&self) -> &String {
        &self.description
    }

    pub fn get_spec_version(&self) -> VulkanVersion {
        self.spec_version
    }

    pub fn get_implementation_version(&self) -> u32 {
        self.implementation_version
    }
}

#[derive(Clone, Debug)]
pub struct ExtensionProperties {
    c_name: CString,
    name: String,
    version: u32,
}

impl ExtensionProperties {
    pub fn new(src: &ash::vk::ExtensionProperties) -> Result<Self, std::str::Utf8Error> {
        let c_name = CString::from(
            unsafe{ CStr::from_ptr(src.extension_name.as_ptr()) }
        );
        let name = String::from(c_name.to_str()?);

        Ok(Self{
            c_name,
            name,
            version: src.spec_version,
        })
    }

    pub fn get_c_name(&self) -> &CString {
        &self.c_name
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_version(&self) -> u32 {
        self.version
    }
}

pub(super) trait Feature {
    type State;

    fn get_payload(&self, pass_state: &Self::State) -> Option<&dyn Any>;

    fn get_payload_mut(&mut self, pass_state: &Self::State) -> Option<&mut dyn Any>;
}

struct FeatureInfo<F: Feature>(Option<F>);

impl<F: Feature> FeatureInfo<F> {
    fn new(feature: F) -> Self {
        Self(Some(feature))
    }

    fn get(&self) -> Option<&F> {
        self.0.as_ref()
    }

    fn get_mut(&mut self) -> Option<&mut F> {
        self.0.as_mut()
    }

    fn take_feature(&mut self) -> F {
        self.0.take().expect("Attempted to take feature that is already processing")
    }

    fn return_feature(&mut self, feature: F) {
        if self.0.is_some() {
            panic!("Attempted to return to feature that is not processing")
        }
        self.0 = Some(feature)
    }
}

/// Internal utility to enable processing of features while allowing access to others
/// TODO could this be simplified using RefCell?
pub(super) struct FeatureSet<F: Feature> {
    features: HashMap<UUID, FeatureInfo<F>>,
    current_stage: Option<F::State>
}

impl<F: Feature> FeatureSet<F> {
    fn new(features: HashMap<UUID, FeatureInfo<F>>) -> Self {
        Self{
            features,
            current_stage: None
        }
    }

    fn take_feature(&mut self, uuid: &UUID) -> F {
        self.features.get_mut(uuid).unwrap().take_feature()
    }

    fn return_feature(&mut self, uuid: &UUID, feature: F) {
        self.features.get_mut(uuid).unwrap().return_feature(feature);
    }
}

impl<F: Feature> FeatureAccess for FeatureSet<F> {
    fn get(&self, feature: &UUID) -> Option<&dyn Any> {
        let stage = self.current_stage.as_ref().expect("Attempted to access feature outside of pass");
        let feature = self.features.get(feature)?;

        feature.get().unwrap().get_payload(stage)
    }

    fn get_mut(&mut self, feature: &UUID) -> Option<&mut dyn Any> {
        let stage = self.current_stage.as_ref().expect("Attempted to access feature outside of pass");
        let feature = self.features.get_mut(feature)?;

        feature.get_mut().unwrap().get_payload_mut(stage)
    }
}

impl<F: Feature> IntoIterator for FeatureSet<F> {
    type Item = F;
    type IntoIter = <Vec<F> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let vec: Vec<_>= self.features.into_iter().map(|(_, mut info)| (info.take_feature())).collect();
        vec.into_iter()
    }
}

/// Internal utility that abstracts the process passes
pub(super) struct FeatureProcessor<F: Feature> {
    order: Box<[NamedUUID]>,
    features: FeatureSet<F>
}

impl<F: Feature> FeatureProcessor<F> {
    /// Creates a new processor using a predefined order
    pub fn new<I: Iterator<Item = (UUID, F)>>(features: I, order: Box<[NamedUUID]>) -> Self {
        Self {
            order,
            features: FeatureSet::new(
                features.map(|(uuid, feature)|
                    (uuid, FeatureInfo::new(feature))
                ).collect()),
        }
    }

    /// Creates a new processor which generates the order based on a dependency graph
    pub fn from_graph<I: Iterator<Item = (NamedUUID, Box<[NamedUUID]>, F)>>(features: I) -> Self {
        let (graph, features): (Vec<_>, HashMap<_, _>) =
            features.map(
                |(name, dependencies, feature)| {
                    let uuid = name.get_uuid();
                    ((name, dependencies), (uuid, FeatureInfo::new(feature)))
                }
            ).unzip();

        let mut topo_sort = topological_sort::TopologicalSort::new();
        for node in graph {
            for dependency in node.1.as_ref() {
                topo_sort.add_dependency(dependency.clone(), node.0.clone());
            }
            topo_sort.insert(node.0);
        };

        // Remove features that dont exist
        let order: Vec<NamedUUID> = topo_sort
            .filter(|uuid: &NamedUUID| features.contains_key(&uuid.get_uuid()))
            .collect();

        Self {
            order: order.into_boxed_slice(),
            features: FeatureSet::new(features),
        }
    }

    /// Runs a pass over all features in order
    pub fn run_pass<R, P>(&mut self, pass_stage: F::State, mut processor: P) -> Result<(), R>
        where P: FnMut(&mut F, &mut dyn FeatureAccess) -> Result<(), R> {

        self.features.current_stage = Some(pass_stage);

        for name in self.order.as_ref() {
            let uuid = name.get_uuid();
            let mut feature = self.features.take_feature(&uuid);

            processor(&mut feature, &mut self.features)?;

            self.features.return_feature(&uuid, feature);
        }

        Ok(())
    }
}

impl<F: Feature> IntoIterator for FeatureProcessor<F> {
    type Item = F;
    type IntoIter = <FeatureSet<F> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.features.into_iter()
    }
}