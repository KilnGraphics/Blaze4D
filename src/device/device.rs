use core::panic::{UnwindSafe, RefUnwindSafe};

use std::cmp::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use ash::prelude::VkResult;

use ash::vk;
use crate::device::device_utils::DeviceUtils;
use crate::device::transfer::Transfer;

use crate::instance::instance::InstanceContext;
use crate::vk::objects::allocator::Allocator;

use crate::prelude::*;

pub struct DeviceFunctions {
    pub instance: Arc<InstanceContext>,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub synchronization_2_khr: ash::extensions::khr::Synchronization2,
    pub timeline_semaphore_khr: ash::extensions::khr::TimelineSemaphore,
    pub push_descriptor_khr: ash::extensions::khr::PushDescriptor,
    pub swapchain_khr: Option<ash::extensions::khr::Swapchain>,
    pub maintenance_4_khr: Option<ash::extensions::khr::Maintenance4>,
}

impl Drop for DeviceFunctions {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}

pub struct DeviceContext {
    id: NamedUUID,
    functions: Arc<DeviceFunctions>,
    main_queue: Arc<Queue>,
    async_compute_queue: Option<Arc<Queue>>,
    async_transfer_queue: Option<Arc<Queue>>,
}

impl DeviceContext {
    pub(crate) fn new(
        functions: Arc<DeviceFunctions>,
        main_queue: Arc<Queue>,
        async_compute_queue: Option<Arc<Queue>>,
        async_transfer_queue: Option<Arc<Queue>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id: NamedUUID::with_str("Device"),
            functions,
            main_queue,
            async_compute_queue,
            async_transfer_queue
        })
    }

    pub fn get_uuid(&self) -> &NamedUUID {
        &self.id
    }

    pub fn get_entry(&self) -> &ash::Entry {
        self.functions.instance.get_entry()
    }

    pub fn get_instance(&self) -> &Arc<InstanceContext> {
        &self.functions.instance
    }

    pub fn get_functions(&self) -> &Arc<DeviceFunctions> {
        &self.functions
    }

    pub fn get_main_queue(&self) -> &Arc<Queue> {
        &self.main_queue
    }

    pub fn get_async_compute_queue(&self) -> Option<&Arc<Queue>> {
        self.async_compute_queue.as_ref()
    }

    pub fn get_async_transfer_queue(&self) -> Option<&Arc<Queue>> {
        self.async_transfer_queue.as_ref()
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

assert_impl_all!(DeviceContext: Send, Sync, UnwindSafe, RefUnwindSafe);

#[derive(Clone)]
pub struct Queue {
    functions: Arc<DeviceFunctions>,
    queue: Arc<Mutex<vk::Queue>>,
    family: u32,
}

impl Queue {
    pub(super) fn new(functions: Arc<DeviceFunctions>, family: u32, index: u32) -> Self {
        let queue = unsafe {
            functions.device.get_device_queue(family, index)
        };

        Self {
            functions,
            queue: Arc::new(Mutex::new(queue)),
            family
        }
    }

    pub unsafe fn submit(&self, submits: &[vk::SubmitInfo], fence: Option<vk::Fence>) -> VkResult<()> {
        let fence = fence.unwrap_or(vk::Fence::null());

        let queue = self.queue.lock().unwrap();
        self.functions.device.queue_submit(*queue, submits, fence)
    }

    pub unsafe fn submit_2(&self, submits: &[vk::SubmitInfo2], fence: Option<vk::Fence>) -> VkResult<()> {
        let fence = fence.unwrap_or(vk::Fence::null());

        let queue = self.queue.lock().unwrap();
        self.functions.synchronization_2_khr.queue_submit2(*queue, submits, fence)
    }

    pub unsafe fn wait_idle(&self) -> VkResult<()> {
        let queue = self.queue.lock().unwrap();
        self.functions.device.queue_wait_idle(*queue)
    }

    pub unsafe fn bind_sparse(&self, bindings: &[vk::BindSparseInfo], fence: Option<vk::Fence>) -> VkResult<()> {
        let fence = fence.unwrap_or(vk::Fence::null());

        let queue = self.queue.lock().unwrap();
        self.functions.device.queue_bind_sparse(*queue, bindings, fence)
    }

    // TODO this also needs to lock the swapchain. How do we properly deal with this?
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