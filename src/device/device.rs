use core::panic::{UnwindSafe, RefUnwindSafe};

use std::cmp::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use ash::prelude::VkResult;

use ash::vk;
use crate::device::device_utils::DeviceUtils;
use crate::device::transfer::Transfer;

use crate::NamedUUID;
use crate::instance::instance::InstanceContext;
use crate::vk::objects::allocator::Allocator;

pub struct DeviceContext {
    weak: Weak<DeviceContext>,
    instance: Arc<InstanceContext>,
    id: NamedUUID,
    device: ash::Device,
    swapchain_khr: Option<ash::extensions::khr::Swapchain>,
    physical_device: vk::PhysicalDevice,
    main_queue: VkQueueTemplate,
    transfer_queue: VkQueueTemplate,
}

impl DeviceContext {
    pub(crate) fn new(
        instance: Arc<InstanceContext>,
        device: ash::Device,
        physical_device: vk::PhysicalDevice,
        swapchain_khr: Option<ash::extensions::khr::Swapchain>,
        main_queue: VkQueueTemplate,
        transfer_queue: VkQueueTemplate,
    ) -> Arc<Self> {
        Arc::new_cyclic(|weak| Self {
            weak: weak.clone(),
            instance,
            id: NamedUUID::with_str("Device"),
            device,
            swapchain_khr,
            physical_device,
            main_queue,
            transfer_queue,
        })
    }

    pub fn get_uuid(&self) -> &NamedUUID {
        &self.id
    }

    pub fn get_entry(&self) -> &ash::Entry {
        self.instance.get_entry()
    }

    pub fn get_instance(&self) -> &Arc<InstanceContext> {
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

    pub fn get_main_queue(&self) -> Queue {
        self.main_queue.promote(self.weak.upgrade().unwrap())
    }

    pub fn get_transfer_queue(&self) -> Queue {
        self.transfer_queue.promote(self.weak.upgrade().unwrap())
    }
}

impl PartialEq for DeviceContext {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for DeviceContext {
}

impl PartialOrd for DeviceContext {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for DeviceContext {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Drop for DeviceContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}

assert_impl_all!(DeviceContext: Send, Sync, UnwindSafe, RefUnwindSafe);

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

    pub fn promote(&self, device: Arc<DeviceContext>) -> Queue {
        Queue {
            device,
            queue: self.queue.clone(),
            family: self.family
        }
    }
}

#[derive(Clone)]
pub struct Queue {
    device: Arc<DeviceContext>,
    queue: Arc<Mutex<vk::Queue>>,
    family: u32,
}

impl Queue {
    pub unsafe fn submit(&self, submits: &[vk::SubmitInfo], fence: Option<vk::Fence>) -> VkResult<()> {
        let fence = fence.unwrap_or(vk::Fence::null());

        let queue = self.queue.lock().unwrap();
        self.device.vk().queue_submit(*queue, submits, fence)
    }

    pub unsafe fn submit_2(&self, submits: &[vk::SubmitInfo2], fence: Option<vk::Fence>) -> VkResult<()> {
        let fence = fence.unwrap_or(vk::Fence::null());

        let queue = self.queue.lock().unwrap();
        self.device.vk().queue_submit2(*queue, submits, fence)
    }

    pub unsafe fn wait_idle(&self) -> VkResult<()> {
        let queue = self.queue.lock().unwrap();
        self.device.vk().queue_wait_idle(*queue)
    }

    pub unsafe fn bind_sparse(&self, bindings: &[vk::BindSparseInfo], fence: Option<vk::Fence>) -> VkResult<()> {
        let fence = fence.unwrap_or(vk::Fence::null());

        let queue = self.queue.lock().unwrap();
        self.device.vk().queue_bind_sparse(*queue, bindings, fence)
    }

    pub unsafe fn present(&self, present_info: &vk::PresentInfoKHR) -> VkResult<bool> {
        let queue = self.queue.lock().unwrap();
        self.device.swapchain_khr().unwrap().queue_present(*queue, present_info)
    }

    pub fn lock_queue(&self) -> MutexGuard<vk::Queue> {
        self.queue.lock().unwrap()
    }

    pub fn get_queue_family_index(&self) -> u32 {
        self.family
    }
}

assert_impl_all!(Queue: Send, Sync, UnwindSafe, RefUnwindSafe);

#[derive(Clone)]
pub struct DeviceEnvironment {
    instance: Arc<InstanceContext>,
    device: Arc<DeviceContext>,
    allocator: Arc<Allocator>,
    transfer: Arc<Transfer>,
    utils: Arc<DeviceUtils>,
}

impl DeviceEnvironment {
    pub(super) fn new(device: Arc<DeviceContext>) -> Self {
        let instance = device.get_instance().clone();

        let allocator = Arc::new(Allocator::new(instance.vk().clone(), device.vk().clone(), *device.get_physical_device()));
        let transfer = Arc::new(Transfer::new(device.clone(), allocator.clone(), device.get_transfer_queue()));
        let utils = DeviceUtils::new(device.clone(), allocator.clone());

        Self {
            instance,
            device,
            allocator,
            transfer,
            utils,
        }
    }

    pub fn vk(&self) -> &ash::Device {
        self.device.vk()
    }

    pub fn get_device(&self) -> &Arc<DeviceContext> {
        &self.device
    }

    pub fn get_instance(&self) -> &Arc<InstanceContext> {
        &self.instance
    }

    pub fn get_entry(&self) -> &ash::Entry {
        self.instance.get_entry()
    }

    pub fn get_allocator(&self) -> &Arc<Allocator> {
        &self.allocator
    }

    pub fn get_transfer(&self) -> &Arc<Transfer> {
        &self.transfer
    }

    pub fn get_utils(&self) -> &Arc<DeviceUtils> {
        &self.utils
    }
}