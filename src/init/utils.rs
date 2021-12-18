use std::any::Any;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::Arc;
use crate::init::application_feature::{FeatureBase, FeatureDependency};
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

enum FeatureStage<T> {
    Uninitialized(T),
    Initialized(T),
    Enabled(T),
    Disabled,
    Processing,
}

struct FeatureInfo<T: ?Sized> {
    stage: FeatureStage<Box<T>>,
    name: NamedUUID,
    dependencies: Arc<[FeatureDependency]>,
}

impl<T: ?Sized> FeatureInfo<T> {
    fn new(feature: Box<T>, name: NamedUUID, dependencies: Arc<[FeatureDependency]>) -> Self {
        Self {
            stage: FeatureStage::Uninitialized(feature),
            name,
            dependencies,
        }
    }

    fn get(&self) -> FeatureStage<&T> {
        match &self.stage {
            FeatureStage::Uninitialized(val) => FeatureStage::Uninitialized(val.as_ref()),
            FeatureStage::Initialized(val) => FeatureStage::Initialized(val.as_ref()),
            FeatureStage::Enabled(val) => FeatureStage::Enabled(val.as_ref()),
            FeatureStage::Disabled => FeatureStage::Disabled,
            FeatureStage::Processing => FeatureStage::Processing,
        }
    }

    fn get_mut(&mut self) -> FeatureStage<&mut T> {
        match &mut self.stage {
            FeatureStage::Uninitialized(val) => FeatureStage::Uninitialized(val.as_mut()),
            FeatureStage::Initialized(val) => FeatureStage::Initialized(val.as_mut()),
            FeatureStage::Enabled(val) => FeatureStage::Enabled(val.as_mut()),
            FeatureStage::Disabled => FeatureStage::Disabled,
            FeatureStage::Processing => FeatureStage::Processing,
        }
    }

    fn take_uninitialized(&mut self) -> Option<Box<T>> {
        let feature = std::mem::replace(&mut self.stage, FeatureStage::Processing);

        match feature {
            FeatureStage::Uninitialized(feature) => Some(feature),
            _ => None,
        }
    }

    fn take_initialized(&mut self) -> Option<Box<T>> {
        let feature = std::mem::replace(&mut self.stage, FeatureStage::Processing);

        match feature {
            FeatureStage::Initialized(feature) => Some(feature),
            _ => None,
        }
    }

    fn return_initialized(&mut self, feature: Box<T>) {
        if let FeatureStage::Processing = &self.stage {
            self.stage = FeatureStage::Initialized(feature);
        } else {
            panic!("Expected feature to be in processing stage but was not");
        }
    }

    fn return_enabled(&mut self, feature: Box<T>) {
        if let FeatureStage::Processing = &self.stage {
            self.stage = FeatureStage::Initialized(feature);
        } else {
            panic!("Expected feature to be in processing stage but was not");
        }
    }

    fn return_disabled(&mut self) {
        if let FeatureStage::Processing = &self.stage {
            self.stage = FeatureStage::Disabled;
        } else {
            panic!("Expected feature to be in processing stage but was not");
        }
    }

    fn is_initialized(&self) -> bool {
        match &self.stage {
            FeatureStage::Initialized(_) => true,
            _ => false,
        }
    }

    fn is_enabled(&self) -> bool {
        match &self.stage {
            FeatureStage::Enabled(_) => true,
            _ => false,
        }
    }

    fn get_dependencies(&self) -> &[FeatureDependency] {
        self.dependencies.as_ref()
    }
}

pub struct FeatureSet<T: ?Sized> {
    features: HashMap<UUID, FeatureInfo<T>>,
}

impl<T: FeatureBase + ?Sized> FeatureSet<T> {
    pub(super) fn new(features: Vec<(Box<T>, NamedUUID, Arc<[FeatureDependency]>)>) -> Self {
        Self {
            features: features.into_iter()
                .map(|(feature, uuid, deps)| (uuid.get_uuid(), FeatureInfo::<T>::new(feature, uuid, deps)))
                .collect(),
        }
    }

    pub fn get_feature<R: FeatureBase + 'static>(&self, name: &UUID) -> Option<&R> {
        self.features.get(name).map(
            |feature| match feature.get() {
                FeatureStage::Uninitialized(_) => panic!("Tried to access feature that was uninitialized"),
                FeatureStage::Initialized(feature) => Some(feature),
                FeatureStage::Enabled(feature) => Some(feature),
                FeatureStage::Disabled => None,
                FeatureStage::Processing => panic!("Tried to access feature that was processing"),
            }.map(|feature| feature.as_any().downcast_ref()).flatten()
        ).flatten()
    }

    pub fn get_feature_mut<R: FeatureBase + 'static>(&mut self, name: &UUID) -> Option<&mut R> {
        self.features.get_mut(name).map(
            |feature| match feature.get_mut() {
                FeatureStage::Uninitialized(_) => panic!("Tried to access feature that was uninitialized"),
                FeatureStage::Initialized(feature) => Some(feature),
                FeatureStage::Enabled(feature) => Some(feature),
                FeatureStage::Disabled => None,
                FeatureStage::Processing => panic!("Tried to access feature that was processing"),
            }.map(|feature| feature.as_any_mut().downcast_mut()).flatten()
        ).flatten()
    }

    pub(super) fn validate_dependencies_initialized(&self, name: &UUID) -> bool {
        for dependency in self.features.get(name).unwrap().get_dependencies() {
            match dependency {
                FeatureDependency::Strong(dep) => {
                    if !self.features.get(&dep.get_uuid()).map_or(false, |f| f.is_initialized()) {
                        return false
                    }
                }
                FeatureDependency::Weak(_) => {}
            }
        }
        true
    }

    pub(super) fn validate_dependencies_enabled(&self, name: &UUID) -> bool {
        for dependency in self.features.get(name).unwrap().get_dependencies() {
            match dependency {
                FeatureDependency::Strong(dep) => {
                    if !self.features.get(&dep.get_uuid()).map_or(false, |f| f.is_enabled()) {
                        return false
                    }
                }
                FeatureDependency::Weak(_) => {}
            }
        }
        true
    }

    pub(super) fn take_uninitialized_feature(&mut self, name: &UUID) -> Option<Box<T>> {
        self.features.get_mut(name).map(|v| v.take_uninitialized()).flatten()
    }

    pub(super) fn take_initialized_feature(&mut self, name: &UUID) -> Option<Box<T>> {
        self.features.get_mut(name).map(|v| v.take_initialized()).flatten()
    }

    pub(super) fn return_feature_initialized(&mut self, name: &UUID, feature: Box<T>) {
        self.features.get_mut(name).unwrap().return_initialized(feature)
    }

    pub(super) fn return_feature_enabled(&mut self, name: &UUID, feature: Box<T>) {
        self.features.get_mut(name).unwrap().return_enabled(feature)
    }

    pub(super) fn return_feature_disabled(&mut self, name: &UUID) {
        self.features.get_mut(name).unwrap().return_disabled()
    }

    pub(super) fn collect_data(&mut self) -> HashMap<UUID, Box<dyn Any>> {
        let mut result = HashMap::new();
        for (uuid, feature) in &self.features {
            match feature.get() {
                FeatureStage::Uninitialized(_) => panic!("Found uninitialized feature while collecting data"),
                FeatureStage::Initialized(_) => panic!("Found initialized feature while collecting data"),
                FeatureStage::Enabled(feature) => { result.insert(uuid.clone(), feature.get_data()); },
                FeatureStage::Disabled => {}
                FeatureStage::Processing => panic!("Found processing feature while collecting data"),
            }
        };
        result
    }
}

pub trait Feature {
    type State;

    fn get_payload(&self, pass_state: &Self::State) -> Option<&dyn Any>;

    fn get_payload_mut(&mut self, pass_state: &Self::State) -> Option<&mut dyn Any>;
}

pub trait FeatureAccess {
    fn get(&self, feature: &UUID) -> Option<&dyn Any>;

    fn get_mut(&mut self, feature: &UUID) -> Option<&mut dyn Any>;
}

struct FeatureInfo2<F: Feature>(Option<F>);

impl<F: Feature> FeatureInfo2<F> {
    fn new(feature: F) -> Self {
        Self(Some(feature))
    }

    fn is_processing(&self) -> bool {
        self.0.is_none()
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

pub struct FeatureSet2<F: Feature> {
    features: HashMap<UUID, FeatureInfo2<F>>,
    current_stage: Option<F::State>
}

impl<F: Feature> FeatureSet2<F> {
    fn new(features: HashMap<UUID, FeatureInfo2<F>>) -> Self {
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

impl<F: Feature> FeatureAccess for FeatureSet2<F> {
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

impl<F: Feature> IntoIterator for FeatureSet2<F> {
    type Item = F;
    type IntoIter = <Vec<F> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let vec: Vec<_>= self.features.into_iter().map(|(_, mut info)| (info.take_feature())).collect();
        vec.into_iter()
    }
}

pub struct FeatureProcessor<F: Feature> {
    order: Box<[NamedUUID]>,
    features: FeatureSet2<F>
}

impl<F: Feature> FeatureProcessor<F> {
    pub fn new<I: Iterator<Item = (UUID, F)>>(features: I, order: Box<[NamedUUID]>) -> Self {
        Self {
            order,
            features: FeatureSet2::new(
                features.map(|(uuid, feature)|
                    (uuid, FeatureInfo2::new(feature))
                ).collect()),
        }
    }

    pub fn from_graph<I: Iterator<Item = (NamedUUID, Box<[NamedUUID]>, F)>>(features: I) -> Self {
        let (graph, features): (Vec<_>, HashMap<_, _>) =
            features.map(
                |(name, dependencies, feature)| {
                    let uuid = name.get_uuid();
                    ((name, dependencies), (uuid, FeatureInfo2::new(feature)))
                }
            ).unzip();

        let mut topo_sort = topological_sort::TopologicalSort::new();
        for node in graph {
            for dependency in node.1.as_ref() {
                topo_sort.add_dependency(dependency.clone(), node.0.clone());
            }
            topo_sort.insert(node.0);
        };

        let order: Vec<NamedUUID> = topo_sort.collect();

        Self {
            order: order.into_boxed_slice(),
            features: FeatureSet2::new(features),
        }
    }

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
    type IntoIter = <FeatureSet2<F> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.features.into_iter()
    }
}