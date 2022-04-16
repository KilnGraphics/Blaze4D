use std::cmp::Ordering;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use ash::prelude::VkResult;

use ash::vk;

use crate::instance::{InstanceContext, InstanceContextImpl};
use crate::objects::id::SurfaceId;
use crate::objects::surface::{SurfaceCapabilities, SurfaceProvider};
use crate::NamedUUID;
use crate::objects::allocator::Allocator;

pub struct DeviceContextImpl {
    id: NamedUUID,
    instance: InstanceContext,
    device: ash::Device,
    swapchain_khr: Option<ash::extensions::khr::Swapchain>,
    physical_device: vk::PhysicalDevice,
    main_queue: VkQueueTemplate,
    transfer_queue: VkQueueTemplate,
    surfaces: HashMap<SurfaceId, (SurfaceCapabilities, VkQueueTemplate)>,
    allocator: ManuallyDrop<Allocator>, // We need manually drop to ensure it is dropped before the device
}

impl DeviceContextImpl {
    pub(crate) fn new(
        instance: InstanceContext,
        device: ash::Device,
        physical_device: vk::PhysicalDevice,
        swapchain_khr: Option<ash::extensions::khr::Swapchain>,
        main_queue: VkQueueTemplate,
        transfer_queue: VkQueueTemplate,
        surfaces: HashMap<SurfaceId, (SurfaceCapabilities, VkQueueTemplate)>,
    ) -> Self {
        let allocator = Allocator::new(instance.vk().clone(), device.clone(), physical_device);

        Self{
            id: NamedUUID::with_str("Device"),
            instance,
            device,
            swapchain_khr,
            physical_device,
            main_queue,
            transfer_queue,
            surfaces,
            allocator: ManuallyDrop::new(allocator),
        }
    }

    pub fn get_uuid(&self) -> &NamedUUID {
        &self.id
    }

    pub fn get_entry(&self) -> &ash::Entry {
        self.instance.get_entry()
    }

    pub fn get_instance(&self) -> &InstanceContextImpl {
        &self.instance
    }

    pub fn vk(&self) -> &ash::Device {
        &self.device
    }

    pub fn swapchain_khr(&self) -> Option<&ash::extensions::khr::Swapchain> {
        self.swapchain_khr.as_ref()
    }

    pub fn get_physical_device(&self) -> &vk::PhysicalDevice {
        &self.physical_device
    }

    pub fn get_allocator(&self) -> &Allocator {
        &self.allocator
    }

    pub fn get_surface(&self, id: SurfaceId) -> Option<&dyn SurfaceProvider> {
        if !self.surfaces.contains_key(&id) {
            None
        } else {
            self.instance.get_surface(id)
        }
    }

    pub fn get_surface_capabilities(&self, id: SurfaceId) -> Option<&SurfaceCapabilities> {
        self.surfaces.get(&id).map(|(cap, _)| cap)
    }
}

impl Drop for DeviceContextImpl {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.allocator);

            self.device.destroy_device(None);
        }
    }
}

impl PartialEq for DeviceContextImpl {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for DeviceContextImpl {
}

impl PartialOrd for DeviceContextImpl {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for DeviceContextImpl {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Clone)]
pub struct DeviceContext(pub(crate) Arc<DeviceContextImpl>);

impl DeviceContext {
    pub fn get_main_queue(&self) -> VkQueue {
        self.0.main_queue.promote(self.clone())
    }

    pub fn get_transfer_queue(&self) -> VkQueue {
        self.0.transfer_queue.promote(self.clone())
    }
}

impl Deref for DeviceContext {
    type Target = DeviceContextImpl;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// Internal struct used to prevent a cyclic dependency between the DeviceContext and the Queue
#[derive(Clone)]
pub(crate) struct VkQueueTemplate {
    queue: Arc<Mutex<vk::Queue>>,
    family: u32,
}

impl VkQueueTemplate {
    pub fn new(queue: vk::Queue, family: u32) -> Self {
        Self {
            queue: Arc::new(Mutex::new(queue)),
            family,
        }
    }

    pub fn promote(&self, device: DeviceContext) -> VkQueue {
        VkQueue {
            device,
            queue: self.queue.clone(),
            family: self.family
        }
    }
}

#[derive(Clone)]
pub struct VkQueue {
    device: DeviceContext,
    queue: Arc<Mutex<vk::Queue>>,
    family: u32,
}

impl VkQueue {
    pub fn submit_2(&self, submits: &[vk::SubmitInfo2], fence: Option<vk::Fence>) -> VkResult<()> {
        let fence = fence.unwrap_or(vk::Fence::null());

        let queue = self.queue.lock().unwrap();
        unsafe { self.device.vk().queue_submit2(*queue, submits, fence) }
    }

    pub fn wait_idle(&self) -> VkResult<()> {
        let queue = self.queue.lock().unwrap();
        unsafe { self.device.vk().queue_wait_idle(*queue) }
    }

    pub fn bind_sparse(&self, bindings: &[vk::BindSparseInfo], fence: Option<vk::Fence>) -> VkResult<()> {
        let fence = fence.unwrap_or(vk::Fence::null());

        let queue = self.queue.lock().unwrap();
        unsafe { self.device.vk().queue_bind_sparse(*queue, bindings, fence) }
    }

    pub fn present(&self, present_info: &vk::PresentInfoKHR) -> VkResult<bool> {
        let queue = self.queue.lock().unwrap();
        unsafe { self.device.swapchain_khr().unwrap().queue_present(*queue, present_info) }
    }
}