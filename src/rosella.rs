use std::sync::Arc;
use crate::ALLOCATION_CALLBACKS;
use ash::{Entry};

use crate::init::device::{create_device};
use crate::init::initialization_registry::InitializationRegistry;
use crate::init::instance_builder::create_instance;
use crate::objects::manager::ObjectManager;
use crate::window::{RosellaSurface, RosellaWindow};

use ash::vk;

pub struct Rosella {
    pub instance: InstanceContext,
    pub surface: RosellaSurface,
    pub device: DeviceContext,
    pub object_manager: ObjectManager,
}

impl Rosella {
    pub fn new(registry: InitializationRegistry, window: &RosellaWindow, application_name: &str) -> Rosella {
        let now = std::time::Instant::now();

        let ash_entry = ash::Entry::new();
        let ash_instance = create_instance(&registry, application_name, 0, window, &ash_entry);

        let instance = InstanceContext::new(ash_entry, ash_instance);

        let surface = RosellaSurface::new(instance.vk(), &Entry::new(), window);
        let (ash_device, physical_device) = create_device(instance.vk(), registry, &surface);

        let device = DeviceContext::new(instance.clone(), ash_device, physical_device).unwrap();

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

        Rosella {
            instance: instance.clone(),
            surface,
            device: device.clone(),
            object_manager: ObjectManager::new(instance, device)
        }
    }

    pub fn window_update(&self) {}

    pub fn recreate_swapchain(&self, width: u32, height: u32) {
        println!("resize to {}x{}", width, height);
    }
}

struct InstanceContextImpl {
    entry: ash::Entry,
    instance: ash::Instance,
}

#[derive(Clone)]
pub struct InstanceContext(Arc<InstanceContextImpl>);

impl InstanceContext {
    fn new(entry: ash::Entry, instance: ash::Instance) -> Self {
        Self(Arc::new(InstanceContextImpl{
            entry,
            instance,
        }))
    }

    pub fn get_entry(&self) -> &ash::Entry {
        &self.0.entry
    }

    pub fn vk(&self) -> &ash::Instance {
        &self.0.instance
    }
}

impl Drop for InstanceContext {
    fn drop(&mut self) {
        unsafe {
            self.0.instance.destroy_instance(ALLOCATION_CALLBACKS);
        }
    }
}

pub struct DeviceContextImpl {
    #[allow(unused)]
    instance: InstanceContext,
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
    synchronization_2: ash::extensions::khr::Synchronization2,
    timeline_semaphore: ash::extensions::khr::TimelineSemaphore,
}

#[derive(Clone)]
pub struct DeviceContext(Arc<DeviceContextImpl>);

impl DeviceContext {
    fn new(instance: InstanceContext, device: ash::Device, physical_device: vk::PhysicalDevice) -> Result<Self, &'static str> {
        let synchronization_2 = ash::extensions::khr::Synchronization2::new(instance.vk(), &device);
        let timeline_semaphore = ash::extensions::khr::TimelineSemaphore::new(instance.get_entry(), instance.vk());

        Ok(Self(Arc::new(DeviceContextImpl{
            instance,
            device,
            physical_device,
            synchronization_2,
            timeline_semaphore
        })))
    }

    pub fn vk(&self) -> &ash::Device {
        &self.0.device
    }

    pub fn get_physical_device(&self) -> &vk::PhysicalDevice {
        &self.0.physical_device
    }

    pub fn get_synchronization_2(&self) -> &ash::extensions::khr::Synchronization2 {
        &self.0.synchronization_2
    }

    pub fn get_timeline_semaphore(&self) -> &ash::extensions::khr::TimelineSemaphore {
        &self.0.timeline_semaphore
    }
}

impl Drop for DeviceContext {
    fn drop(&mut self) {
        unsafe {
            self.0.device.destroy_device(ALLOCATION_CALLBACKS);
        }
    }
}