use std::sync::Arc;
use crate::ALLOCATION_CALLBACKS;
use ash::{Entry};

use ash::vk;

use crate::init::device::{create_device};
use crate::init::initialization_registry::InitializationRegistry;
use crate::init::instance_builder::create_instance;
use crate::window::{RosellaSurface, RosellaWindow};

pub struct Rosella {
    pub instance: Arc<InstanceContext>,
    pub surface: RosellaSurface,
    pub device: Arc<DeviceContext>,
}

impl Rosella {
    pub fn new(registry: InitializationRegistry, window: &RosellaWindow, application_name: &str) -> Rosella {
        let now = std::time::Instant::now();

        let ash_entry = ash::Entry::new();
        let ash_instance = create_instance(&registry, application_name, 0, window, &ash_entry);

        let instance = Arc::new(InstanceContext::new(ash_entry, ash_instance, VulkanVersion::VK_1_0));

        let surface = RosellaSurface::new(instance.vk(), &Entry::new(), window);
        let ash_device = create_device(instance.vk(), registry, &surface);

        let device = Arc::new(DeviceContext::new(instance.clone(), ash_device).unwrap());

        let elapsed = now.elapsed();
        println!("Instance & Device Initialization took: {:.2?}", elapsed);

        /*        let vk = Entry::new();
        let app_name = CString::new(application_name);
        let surface_extensions = ash_window::enumerate_required_extensions(&window.handle).unwrap();
        let mut extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();
        extension_names_raw.push(DebugUtils::name().as_ptr());

        let debug_utils_loader = DebugUtils::new(&vk, &instance);

        unsafe {
            let debug_call_back = debug_utils_loader
                .create_debug_utils_messenger(&debug_info, ALLOCATION_CALLBACKS)
                .unwrap();
        }*/

        Rosella { instance, surface, device }
    }

    pub fn window_update(&self) {}

    pub fn recreate_swapchain(&self, width: u32, height: u32) {
        println!("resize to {}x{}", width, height);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VulkanVersion(u32);

impl VulkanVersion {
    pub const VK_1_0: VulkanVersion = VulkanVersion(vk::API_VERSION_1_0);
    pub const VK_1_1: VulkanVersion = VulkanVersion(vk::API_VERSION_1_1);
    pub const VK_1_2: VulkanVersion = VulkanVersion(vk::API_VERSION_1_2);

    pub fn new(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
        Self(vk::make_api_version(variant, major, minor, patch))
    }

    pub fn is_supported(&self, version: VulkanVersion) -> bool {
        vk::api_version_major(self.0) >= vk::api_version_major(version.0)
    }
}

pub struct InstanceContext {
    version: VulkanVersion,
    entry: ash::Entry,
    instance: ash::Instance,
    khr_get_physical_device_properties_2: Option<ash::extensions::khr::GetPhysicalDeviceProperties2>,
}

impl InstanceContext {
    fn new(entry: ash::Entry, instance: ash::Instance, version: VulkanVersion) -> Self {
        Self{ entry, instance, version,
            khr_get_physical_device_properties_2: None
        }
    }

    pub fn get_entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn vk(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn get_khr_get_physical_device_properties_2(&self) -> Option<&ash::extensions::khr::GetPhysicalDeviceProperties2> {
        self.khr_get_physical_device_properties_2.as_ref()
    }

    pub fn get_version(&self) -> VulkanVersion {
        self.version
    }
}

impl Drop for InstanceContext {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(ALLOCATION_CALLBACKS);
        }
    }
}

pub struct DeviceContext {
    #[allow(unused)]
    instance: Arc<InstanceContext>,
    device: ash::Device,
    synchronization_2: ash::extensions::khr::Synchronization2,
    timeline_semaphore: ash::extensions::khr::TimelineSemaphore,
}

impl DeviceContext {
    fn new(instance: Arc<InstanceContext>, device: ash::Device) -> Result<Self, &'static str> {
        let synchronization_2 = ash::extensions::khr::Synchronization2::new(instance.vk(), &device);
        let timeline_semaphore = ash::extensions::khr::TimelineSemaphore::new(instance.get_entry(), instance.vk());

        Ok(Self{
            instance,
            device,
            synchronization_2,
            timeline_semaphore
        })
    }

    pub fn vk(&self) -> &ash::Device {
        &self.device
    }

    pub fn get_synchronization_2(&self) -> &ash::extensions::khr::Synchronization2 {
        &self.synchronization_2
    }

    pub fn get_timeline_semaphore(&self) -> &ash::extensions::khr::TimelineSemaphore {
        &self.timeline_semaphore
    }
}

impl Drop for DeviceContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(ALLOCATION_CALLBACKS);
        }
    }
}