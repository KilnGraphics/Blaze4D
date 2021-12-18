use std::any::Any;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::Arc;
use crate::init::application_feature::FeatureDependency;
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

pub trait FeatureBase {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_data(&self) -> Box<dyn Any>;
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