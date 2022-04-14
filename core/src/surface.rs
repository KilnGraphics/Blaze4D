use std::ffi::CString;
use ash::vk;

pub enum SurfaceInitError {
    /// A vulkan error
    Vulkan(vk::Result),
    /// A generic error with attached message
    Message(String),
    /// A generic error
    Generic(),
}

///
pub trait SurfaceProvider {
    fn get_required_instance_extensions(&self) -> Vec<CString>;

    fn init(&mut self, entry: &ash::Entry, instance: &ash::Instance) -> Result<vk::SurfaceKHR, SurfaceInitError>;
}