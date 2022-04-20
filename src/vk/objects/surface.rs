use std::ffi::CString;

use ash::vk;
use crate::vk::instance::InstanceContextImpl;

#[derive(Debug)]
pub enum SurfaceInitError {
    /// A vulkan error
    Vulkan(vk::Result),
    /// A generic error with attached message
    Message(String),
    /// A generic error
    Generic(),
}

impl From<vk::Result> for SurfaceInitError {
    fn from(res: vk::Result) -> Self {
        SurfaceInitError::Vulkan(res)
    }
}

pub trait SurfaceProvider {
    fn get_required_instance_extensions(&self) -> Vec<CString>;

    fn init(&mut self, entry: &ash::Entry, instance: &ash::Instance) -> Result<vk::SurfaceKHR, SurfaceInitError>;

    fn get_handle(&self) -> Option<vk::SurfaceKHR>;
}

pub struct SurfaceCapabilities {
    presentable_queues: Box<[u32]>,
    surface_formats: Box<[vk::SurfaceFormatKHR]>,
    present_modes: Box<[vk::PresentModeKHR]>,
    capabilities: vk::SurfaceCapabilitiesKHR,
}

impl SurfaceCapabilities {
    pub fn new(instance: &InstanceContextImpl, physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> Option<Self> {
        let surface_fn = instance.surface_khr()?;
        let family_count = unsafe {
            instance.vk().get_physical_device_queue_family_properties(physical_device).len()
        } as u32;

        let presentable_queues = (0..family_count).filter(|family| unsafe {
            surface_fn.get_physical_device_surface_support(physical_device, *family, surface).unwrap()
        }).collect::<Vec<_>>().into_boxed_slice();

        if presentable_queues.len() == 0 {
            return None;
        }

        let capabilities = unsafe {
            surface_fn.get_physical_device_surface_capabilities(physical_device, surface)
        }.ok()?;

        let surface_formats = unsafe {
            surface_fn.get_physical_device_surface_formats(physical_device, surface)
        }.ok()?.into_boxed_slice();

        let present_modes = unsafe {
            surface_fn.get_physical_device_surface_present_modes(physical_device, surface)
        }.ok()?.into_boxed_slice();

        Some(Self{
            presentable_queues,
            surface_formats,
            present_modes,
            capabilities,
        })
    }

    pub fn get_capabilities(&self) -> &vk::SurfaceCapabilitiesKHR {
        &self.capabilities
    }

    pub fn get_presentable_queue_families(&self) -> &[u32] {
        self.presentable_queues.as_ref()
    }

    pub fn get_surface_formats(&self) -> &[vk::SurfaceFormatKHR] {
        self.surface_formats.as_ref()
    }

    pub fn get_present_modes(&self) -> &[vk::PresentModeKHR] {
        self.present_modes.as_ref()
    }
}