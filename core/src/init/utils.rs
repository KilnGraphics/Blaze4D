use std::ffi::{CStr, CString};
use crate::rosella::VulkanVersion;

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