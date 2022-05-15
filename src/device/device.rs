use std::cmp::Ordering;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use ash::prelude::VkResult;

use ash::vk;
use crate::device::device_utils::DeviceUtils;

use crate::NamedUUID;
use crate::instance::instance::InstanceContext;
use crate::vk::objects::surface::{SurfaceId, SurfaceProvider};
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
    surfaces: ManuallyDrop<HashMap<SurfaceId, DeviceSurface>>,
}

impl DeviceContext {
    pub(crate) fn new(
        instance: Arc<InstanceContext>,
        device: ash::Device,
        physical_device: vk::PhysicalDevice,
        swapchain_khr: Option<ash::extensions::khr::Swapchain>,
        main_queue: VkQueueTemplate,
        transfer_queue: VkQueueTemplate,
        surfaces: HashMap<SurfaceId, (Box<dyn SurfaceProvider>, Box<[bool]>)>,
    ) -> Arc<Self> {
        let surfaces = surfaces.into_iter().map(|(id, (prov, supported_queues))| {
            (id, DeviceSurface {
                handle: prov.get_handle().unwrap(),
                swapchain_info: Mutex::new(SurfaceSwapchainInfo::None),
                supported_queues,
                provider: prov,
            })
        }).collect();

        Arc::new_cyclic(|weak| Self {
            weak: weak.clone(),
            instance,
            id: NamedUUID::with_str("Device"),
            device,
            swapchain_khr,
            physical_device,
            main_queue,
            transfer_queue,
            surfaces: ManuallyDrop::new(surfaces),
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

    pub(crate) fn get_surface(&self, id: SurfaceId) -> Option<(vk::SurfaceKHR, &Mutex<SurfaceSwapchainInfo>)> {
        self.surfaces.get(&id).map(|s| {
            (s.handle, &s.swapchain_info)
        })
    }

    pub fn get_surface_capabilities(&self, id: SurfaceId) -> Option<vk::SurfaceCapabilitiesKHR> {
        let (handle, _) = self.get_surface(id)?;

        Some(unsafe { self.instance.surface_khr().unwrap()
            .get_physical_device_surface_capabilities(self.physical_device, handle) }.unwrap())
    }

    pub fn get_main_queue(&self) -> VkQueue {
        self.main_queue.promote(self.weak.upgrade().unwrap())
    }

    pub fn get_transfer_queue(&self) -> VkQueue {
        self.transfer_queue.promote(self.weak.upgrade().unwrap())
    }

    pub fn get_surface_queue_support(&self, id: SurfaceId, queue: &VkQueue) -> Option<bool> {
        self.surfaces.get(&id).map(|surface|
            *surface.supported_queues.get(queue.get_queue_family_index() as usize).unwrap()
        )
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
            ManuallyDrop::drop(&mut self.surfaces);

            self.device.destroy_device(None);
        }
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

    pub fn promote(&self, device: Arc<DeviceContext>) -> VkQueue {
        VkQueue {
            device,
            queue: self.queue.clone(),
            family: self.family
        }
    }
}

#[derive(Clone)]
pub struct VkQueue {
    device: Arc<DeviceContext>,
    queue: Arc<Mutex<vk::Queue>>,
    family: u32,
}

impl VkQueue {
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

struct DeviceSurface {
    handle: vk::SurfaceKHR,
    swapchain_info: Mutex<SurfaceSwapchainInfo>,
    supported_queues: Box<[bool]>,

    #[allow(unused)] // We just need to keep the provider alive
    provider: Box<dyn SurfaceProvider>,
}

/// Contains information about the current non retired swapchain associated with the surface.
pub(crate) enum SurfaceSwapchainInfo {
    Some {
        handle: vk::SwapchainKHR,
    },
    None
}

impl SurfaceSwapchainInfo {
    pub fn get_current_handle(&self) -> Option<vk::SwapchainKHR> {
        match self {
            SurfaceSwapchainInfo::Some { handle, .. } => Some(*handle),
            SurfaceSwapchainInfo::None => None
        }
    }

    pub fn set_swapchain(&mut self, handle: vk::SwapchainKHR) {
        *self = SurfaceSwapchainInfo::Some {
            handle
        };
    }

    pub fn clear(&mut self) {
        *self = SurfaceSwapchainInfo::None;
    }
}

#[derive(Clone)]
pub struct DeviceEnvironment {
    instance: Arc<InstanceContext>,
    device: Arc<DeviceContext>,
    allocator: Arc<Allocator>,
    utils: Arc<DeviceUtils>,
}

impl DeviceEnvironment {
    pub(super) fn new(device: Arc<DeviceContext>) -> Self {
        let instance = device.get_instance().clone();

        let allocator = Arc::new(Allocator::new(instance.vk().clone(), device.vk().clone(), *device.get_physical_device()));
        let utils = DeviceUtils::new(device.clone(), allocator.clone());

        Self {
            instance,
            device,
            allocator,
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

    pub fn get_utils(&self) -> &Arc<DeviceUtils> {
        &self.utils
    }
}