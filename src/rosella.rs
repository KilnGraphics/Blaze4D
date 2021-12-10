use std::sync::Arc;
use crate::ALLOCATION_CALLBACKS;
use ash::{Entry};

use crate::init::device::{create_device};
use crate::init::initialization_registry::InitializationRegistry;
use crate::init::instance_builder::create_instance;
use crate::objects::manager::ObjectManager;
use crate::window::{RosellaSurface, RosellaWindow};

pub struct Rosella {
    pub instance: Arc<InstanceContext>,
    pub surface: RosellaSurface,
    pub device: Arc<DeviceContext>,
    pub object_manager: ObjectManager,
}

impl Rosella {
    pub fn new(registry: InitializationRegistry, window: &RosellaWindow, application_name: &str) -> Rosella {
        let now = std::time::Instant::now();

        let ash_entry = ash::Entry::new();
        let ash_instance = create_instance(&registry, application_name, 0, window, &ash_entry);

        let instance = Arc::new(InstanceContext::new(ash_entry, ash_instance));

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

pub struct InstanceContext {
    entry: ash::Entry,
    instance: ash::Instance,
}

impl InstanceContext {
    fn new(entry: ash::Entry, instance: ash::Instance) -> Self {
        Self{ entry, instance }
    }

    pub fn get_entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn vk(&self) -> &ash::Instance {
        &self.instance
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