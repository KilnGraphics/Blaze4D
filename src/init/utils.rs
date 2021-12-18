use std::any::Any;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::iter::Map;
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

pub struct FeatureInfo2<S: Eq, F> {
    state: S,
    name: NamedUUID,
    feature: Option<F>,
}

impl<S: Eq, F> FeatureInfo2<S, F> {
    fn new(state: S, feature: F, name: NamedUUID) -> Self {
        Self {
            state,
            name,
            feature: Some(feature),
        }
    }

    fn get_state(&self) -> &S {
        &self.state
    }

    fn is_in_state(&self, state: &S) -> bool {
        &self.state == state
    }

    fn set_state(&mut self, state: S) {
        self.state = state;
    }

    fn is_processing(&self) -> bool {
        self.feature.is_none()
    }

    fn get(&self) -> Option<&F> {
        self.feature.as_ref()
    }

    fn get_mut(&mut self) -> Option<&mut F> {
        self.feature.as_mut()
    }

    fn take_feature(&mut self) -> F {
        self.feature.take().expect("Attempted to take feature that is already processing")
    }

    fn return_feature(&mut self, feature: F) {
        if self.feature.is_some() {
            panic!("Attempted to return to feature that is not processing")
        }
        self.feature = Some(feature)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ConditionResult {
    Take,
    Skip,
}

pub struct FeatureSet2<S: Eq + 'static, F> {
    features: HashMap<UUID, FeatureInfo2<S, F>>,
    get_test: Option<&'static dyn Fn(&S) -> bool>,
}

impl<S: Eq + 'static, F> FeatureSet2<S, F> {
    fn new<I: Iterator>(features: I) -> Self
        where <I as Iterator>::Item: Into<(NamedUUID, F, S)> {

        let features = features.map(|feature| {
            let (uuid, feature, state) = feature.into();
            (uuid.get_uuid(), FeatureInfo2::new(state, feature, uuid))
        }).collect();

        Self{
            features,
            get_test: None,
        }
    }

    fn take_conditional<R, C>(&mut self, uuid: &UUID, cnd: C) -> Result<Option<F>, R>
        where C: Fn(&S) -> Result<ConditionResult, R> {

        match self.features.get_mut(uuid) {
            Some(info) => {
                if info.is_processing() {
                    panic!("Attempted to take a feature which is already processing");
                }

                cnd(info.get_state()).map(|result| {
                    if result == ConditionResult::Take {
                        Some(info.take_feature())
                    } else {
                        None
                    }
                })
            }
            None => {
                panic!("Attempted to take a feature which does not exist");
            }
        }
    }

    fn transition_return(&mut self, uuid: &UUID, feature: F, new_state: S) {
        let info = self.features.get_mut(uuid).expect("Attempted to return non existing feature");
        info.return_feature(feature);
        info.set_state(new_state);
    }

    fn clear_get_test(&mut self) {
        self.get_test = None
    }

    fn set_get_test(&mut self, test: &'static dyn Fn(&S) -> bool) {
        self.get_test = Some(test)
    }
}

impl<S: Eq + 'static, F: FeatureBase> FeatureSet2<S, F> {
    pub fn get<T: 'static>(&self, name: &NamedUUID) -> Option<&T> {
        match self.get_test {
            Some(test) => {
                match self.features.get(&name.get_uuid()) {
                    Some(info) => {
                        if test(info.get_state()) {
                            Some(info.get().expect("Attempted to get processing feature")
                                .as_any().downcast_ref().expect("Invalid get type"))
                        } else {
                            None
                        }
                    }
                    None => None
                }
            }
            None => panic!("Called get but get_test is not set"),
        }
    }

    pub fn get_mut<T: 'static>(&mut self, name: &NamedUUID) -> Option<&T> {
        match self.get_test {
            Some(test) => {
                match self.features.get_mut(&name.get_uuid()) {
                    Some(info) => {
                        if test(info.get_state()) {
                            Some(info.get_mut().expect("Attempted to get processing feature")
                                .as_any_mut().downcast_mut().expect("Invalid get type"))
                        } else {
                            None
                        }
                    }
                    None => None
                }
            }
            None => panic!("Called get but get_test is not set"),
        }
    }
}

impl<S: Eq + 'static, F> IntoIterator for FeatureSet2<S, F> {
    type Item = (S, F);
    type IntoIter = Map<<HashMap::<UUID, FeatureInfo2<S, F>> as IntoIterator>::IntoIter, fn((UUID, FeatureInfo2<S, F>)) -> (S, F)>;

    fn into_iter(self) -> Self::IntoIter {
        self.features.into_iter().map(|(_, mut info)|
            (info.state, info.feature.take().expect("Attempted to convert processing feature set into iterator")))
    }
}

pub struct FeatureProcessor<S: Eq + 'static, F> {
    order: Box<[NamedUUID]>,
    features: FeatureSet2<S, F>
}

impl<S: Eq + 'static, F> FeatureProcessor<S, F> {
    pub fn new<I: Iterator>(features: I, order: Box<[NamedUUID]>) -> Self
        where <I as Iterator>::Item: Into<(NamedUUID, F, S)> {

        Self {
            order,
            features: FeatureSet2::new(features),
        }
    }

    pub fn from_graph<I: Iterator<Item = (NamedUUID, Box<[NamedUUID]>, F, S)>>(features: I) -> Self {
        let (graph, features): (Vec<_>, Vec<_>) =
            features.map(
                |(name, dependencies, feature, state)|
                    ((name.clone(), dependencies), (name, feature, state))
            ).unzip();

        let mut topo_sort = topological_sort::TopologicalSort::new();
        for node in graph {
            for dependency in node.1.as_ref() {
                topo_sort.add_dependency(dependency.clone(), node.0.clone());
            }
        };

        let order: Vec<NamedUUID> = topo_sort.collect();

        Self {
            order: order.into_boxed_slice(),
            features: FeatureSet2::new(features.into_iter()),
        }
    }

    pub fn run_pass<C, P, R>(&mut self, condition: C, get_test: &'static dyn Fn(&S) -> bool, mut processor: P) -> Result<(), R>
        where C: Fn(&S) -> Result<ConditionResult, R>, P: FnMut(&mut F, &mut FeatureSet2<S, F>) -> Result<S, R> {

        self.features.set_get_test(get_test);

        for uuid in self.order.as_ref() {
            let uuid = uuid.get_uuid();

            let feature = self.features.take_conditional(&uuid, &condition)?;

            if let Some(mut feature) = feature {
                let new_state = processor(&mut feature, &mut self.features)?;

                self.features.transition_return(&uuid, feature, new_state);
            }
        }

        Ok(())
    }
}

impl<S: Eq + 'static, F> IntoIterator for FeatureProcessor<S, F> {
    type Item = (S, F);
    type IntoIter = <FeatureSet2<S, F> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.features.into_iter()
    }
}