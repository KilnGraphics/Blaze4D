use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::sync::Arc;
use ash::vk;
use ash::vk::PhysicalDeviceType;
use vk_profiles_rs::vp;
use crate::device::{DeviceContext, DeviceContextImpl, VkQueueTemplate};
use crate::instance::InstanceContext;
use crate::objects::id::SurfaceId;
use crate::objects::surface::{SurfaceCapabilities, SurfaceProvider};

pub type DeviceRatingFn = dyn Fn(&InstanceContext, vk::PhysicalDevice) -> Option<f32>;

pub struct DeviceCreateConfig {
    surfaces: HashSet<SurfaceId>,
    require_swapchain: bool,
    disable_robustness: bool,
    rating_fn: Box<DeviceRatingFn>,
}

impl DeviceCreateConfig {
    pub fn new() -> Self {
        Self {
            surfaces: HashSet::new(),
            require_swapchain: false,
            disable_robustness: false,
            rating_fn: Box::new(Self::default_rating)
        }
    }

    pub fn add_surface(&mut self, surface: SurfaceId) {
        self.surfaces.insert(surface);
        self.require_swapchain = true;
    }

    pub fn require_swapchain(&mut self) {
        self.require_swapchain = true;
    }

    pub fn disable_robustness(&mut self) {
        self.disable_robustness = true;
    }

    fn default_rating(instance: &InstanceContext, device: vk::PhysicalDevice) -> Option<f32> {
        let properties = unsafe { instance.vk().get_physical_device_properties(device) };
        Some(match properties.device_type {
            PhysicalDeviceType::DISCRETE_GPU => 10.0f32,
            PhysicalDeviceType::INTEGRATED_GPU => 5.0f32,
            _ => 0.0f32,
        })
    }
}

#[derive(Debug)]
pub enum DeviceCreateError {
    Vulkan(vk::Result),
    NoSupportedDevice,
    SurfaceNotFound,
}

impl From<vk::Result> for DeviceCreateError {
    fn from(result: vk::Result) -> Self {
        DeviceCreateError::Vulkan(result)
    }
}

pub fn create_device(config: DeviceCreateConfig, instance: InstanceContext) -> Result<DeviceContext, DeviceCreateError> {
    let vk_vp = vk_profiles_rs::VulkanProfiles::linked();

    let mut surfaces = Vec::with_capacity(config.surfaces.len());
    for id in &config.surfaces {
        if let Some(surface) = instance.take_surface(*id) {
            surfaces.push((*id, surface));
        } else {
            return Err(DeviceCreateError::SurfaceNotFound);
        }
    }

    let has_swapchain;
    let mut required_extensions = HashSet::new();
    if config.require_swapchain || !config.surfaces.is_empty() {
        required_extensions.insert(CString::from(CStr::from_bytes_with_nul(b"VK_KHR_swapchain\0").unwrap()));
        has_swapchain = true;
    } else {
        has_swapchain = false;
    }

    let selected_device = filter_devices(
        unsafe { instance.vk().enumerate_physical_devices()? },
        &instance,
        &required_extensions,
        &surfaces,
        config.rating_fn.as_ref()
    )?;

    let required_extensions_str: Vec<_> = required_extensions.iter().map(|ext| ext.as_c_str().as_ptr()).collect();

    let device_queue_create_infos: Vec<_> = selected_device.queues.iter().map(|q|
        // We have to build here
        vk::DeviceQueueCreateInfo::builder().queue_family_index(q.family).queue_priorities(&q.priorities).build()
    ).collect();

    let vk_device_create_info = vk::DeviceCreateInfo::builder()
        .enabled_extension_names(required_extensions_str.as_slice())
        .queue_create_infos(device_queue_create_infos.as_slice());

    let flags = if config.disable_robustness {
        vp::DeviceCreateFlagBits::MERGE_EXTENSIONS | vp::DeviceCreateFlagBits::DISABLE_ROBUST_ACCESS
    } else {
        vp::DeviceCreateFlagBits::MERGE_EXTENSIONS
    };
    let vp_device_create_info = vp::DeviceCreateInfo::builder()
        .profile(instance.get_profile())
        .create_info(&vk_device_create_info)
        .flags(flags);

    let device = unsafe { vk_vp.create_device(instance.vk(), selected_device.device, &vp_device_create_info, None)? };

    let queue_map = QueueMap::new(&device, selected_device.queues.as_ref());

    let main_queue = queue_map.get_queue(selected_device.graphics_compute_queue);
    let transfer_queue = queue_map.get_queue(selected_device.transfer_queue);



    let swapchain_khr = if has_swapchain {
        Some(ash::extensions::khr::Swapchain::new(instance.vk(), &device))
    } else {
        None
    };

    Ok(DeviceContext(Arc::new(DeviceContextImpl::new(
        instance,
        device,
        selected_device.device,
        swapchain_khr,
        main_queue,
        transfer_queue,
        HashMap::new()
    ))))
}

fn filter_devices(
    devices: Vec<vk::PhysicalDevice>,
    instance: &InstanceContext,
    required_extensions: &HashSet<CString>,
    surfaces: &Vec<(SurfaceId, Box<dyn SurfaceProvider>)>,
    rating_fn: &DeviceRatingFn
) -> Result<PhysicalDeviceConfig, DeviceCreateError> {
    let vk_vp = vk_profiles_rs::VulkanProfiles::linked();

    let mut best_device: Option<PhysicalDeviceConfig> = None;
    for device in devices {
        if let Some(config) = process_device(&vk_vp, instance, device, required_extensions, surfaces, rating_fn)? {
            best_device = if let Some(old) = best_device {
                if config.rating > old.rating {
                    Some(config)
                } else {
                    Some(old)
                }
            } else {
                Some(config)
            }
        }
    }

    best_device.ok_or(DeviceCreateError::NoSupportedDevice)
}

fn process_device(
    vk_vp: &vk_profiles_rs::VulkanProfiles,
    instance: &InstanceContext,
    device: vk::PhysicalDevice,
    required_extensions: &HashSet<CString>,
    surfaces: &Vec<(SurfaceId, Box<dyn SurfaceProvider>)>,
    rating_fn: &DeviceRatingFn,
) -> Result<Option<PhysicalDeviceConfig>, DeviceCreateError> {
    if !unsafe { vk_vp.get_physical_device_profile_support(instance.vk(), device, instance.get_profile())? } {
        return Ok(None);
    }

    // Verify extensions
    let available_extensions: HashSet<_> = unsafe { instance.vk().enumerate_device_extension_properties(device)? }
        .into_iter().map(|ext| {
        CString::from(unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) })
    }).collect();

    if !required_extensions.is_subset(&available_extensions) {
        return Ok(None);
    }

    // Initialize queue family data
    let mut queue_family_properties: Vec<_> = std::iter::repeat(
        vk::QueueFamilyProperties2::default()).take(
        unsafe { instance.vk().get_physical_device_queue_family_properties2_len(device) }
    ).collect();
    unsafe { instance.vk().get_physical_device_queue_family_properties2(device, queue_family_properties.as_mut_slice()) };
    let queue_family_properties = queue_family_properties;

    let mut main_queue = if let Some(family) = find_main_queue_family(&queue_family_properties) {
        QueueAllocation{ family, index: 0 }
    } else {
        return Ok(None);
    };
    let mut transfer_queue = QueueAllocation{ family: find_transfer_queue_family(&queue_family_properties, main_queue.family), index: 0 };

    let mut present_queues = Vec::new();
    for (id, surface) in surfaces {
        let handle = surface.get_handle().unwrap();

        let capabilities = match SurfaceCapabilities::new(instance, device, handle) {
            Some(caps) => caps,
            None => return Ok(None)
        };

        let present_family = find_present_queue_family(capabilities.get_presentable_queue_families(), main_queue.family, transfer_queue.family);

        present_queues.push((*id, QueueAllocation{ family: present_family, index: 0 }));
    }

    let rating = match rating_fn(instance, device) {
        Some(rating) => rating,
        None => return Ok(None)
    };

    let allocation = generate_queue_allocation(
        queue_family_properties.as_slice(),
        &mut main_queue,
        &mut transfer_queue,
        &mut present_queues
    );

    Ok(Some(PhysicalDeviceConfig {
        device,
        rating,
        queues: allocation.into_boxed_slice(),
        graphics_compute_queue: main_queue,
        transfer_queue,
        present_queues
    }))
}

fn find_main_queue_family(properties: &Vec<vk::QueueFamilyProperties2>) -> Option<u32> {
    let required_mask = vk::QueueFlags::COMPUTE | vk::QueueFlags::GRAPHICS;
    for (family, info) in properties.iter().enumerate() {
        if info.queue_family_properties.queue_flags.contains(required_mask) {
            return Some(family as u32);
        }
    }

    None
}

fn find_transfer_queue_family(properties: &Vec<vk::QueueFamilyProperties2>, main_queue_family: u32) -> u32 {
    let mut best: Option<(u32, u32)> = None;
    for (family, info) in properties.iter().enumerate() {
        let family = family as u32;
        if family != main_queue_family {
            if info.queue_family_properties.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                let g = info.queue_family_properties.min_image_transfer_granularity;
                let extent_sum = g.depth + g.height + g.width;

                if extent_sum == 3 {
                    // Found best possible family
                    return family
                }

                best = best.map_or(
                    Some((family, extent_sum)),
                    |(old_family, old_extent)| {
                        if (extent_sum < old_extent) || (old_extent == 0) {
                            Some((family, extent_sum))
                        } else {
                            Some((old_family, old_extent))
                        }
                    }
                );
            }
        }
    }

    best.map(|b| b.0).unwrap_or(main_queue_family)
}

fn find_present_queue_family(supported_families: &[u32], main_family: u32, transfer_family: u32) -> u32 {
    if supported_families.is_empty() { panic!("Empty queue family set") }
    // First search for a family that is disjoint with the main and transfer family
    for family in supported_families {
        if *family != main_family && *family != transfer_family {
            return *family;
        }
    }
    // Search for the transfer family
    for family in supported_families {
        if *family == transfer_family {
            return transfer_family;
        }
    }
    // Last resort use the main family
    return main_family;
}

fn generate_queue_allocation(
    properties: &[vk::QueueFamilyProperties2],
    main_queue: &mut QueueAllocation,
    transfer_queue: &mut QueueAllocation,
    present_queues: &mut [(SurfaceId, QueueAllocation)]
) -> Vec<QueueCreateInfo> {
    // We will assign queues in order of importance. If we run out we will repeat the last queue as its the least important
    let mut queues = properties.iter().map(|q|
        (0u32, q.queue_family_properties.queue_count)).collect::<Vec<(u32, u32)>>().into_boxed_slice();

    let mut main = queues.get_mut(main_queue.family as usize).unwrap();
    main_queue.index = main.0;
    main.0 += 1;

    let mut transfer = queues.get_mut(transfer_queue.family as usize).unwrap();
    if transfer.0 == transfer.1 {
        transfer_queue.index = transfer.0 - 1;
    } else {
        transfer_queue.index = transfer.0;
        transfer.0 += 1;
    }

    for present_queue in present_queues.iter_mut() {
        let mut present = queues.get_mut(present_queue.1.index as usize).unwrap();
        if present.0 == present.1 {
            present_queue.1.index = present.0 - 1;
        } else {
            present_queue.1.index = present.0;
            present.0 += 1;
        }
    }

    queues.into_vec().into_iter().enumerate().filter_map(|(index, (count, _))| {
        if count == 0 {
            None
        } else {
            Some(QueueCreateInfo::make(index as u32, count))
        }
    }).collect()
}

#[derive(Debug)]
struct QueueCreateInfo {
    family: u32,
    count: u32,
    priorities: Box<[f32]>,
}

impl QueueCreateInfo {
    fn make(family: u32, count: u32) -> Self {
        let priorities = std::iter::repeat(1.0f32).take(count as usize).collect::<Vec<_>>().into_boxed_slice();
        Self {
            family,
            count,
            priorities
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct QueueAllocation {
    family: u32,
    index: u32,
}

struct QueueMap {
    queues: HashMap<u32, Box<[VkQueueTemplate]>>,
}

impl QueueMap {
    fn new(device: &ash::Device, queues: &[QueueCreateInfo]) -> Self {
        let mut map = HashMap::new();

        for queue in queues.iter() {
            let mut vec = Vec::with_capacity(queue.count as usize);
            for index in 0..queue.count {
                let vk_queue = unsafe { device.get_device_queue(queue.family, index) };
                vec.push(VkQueueTemplate::new(vk_queue, queue.family));
            }
            map.insert(queue.family, vec.into_boxed_slice());
        }

        Self {
            queues: map
        }
    }

    fn get_queue(&self, allocation: QueueAllocation) -> VkQueueTemplate {
        self.queues.get(&allocation.family).unwrap().get(allocation.index as usize).unwrap().clone()
    }
}

#[derive(Debug)]
struct PhysicalDeviceConfig {
    device: vk::PhysicalDevice,
    rating: f32,
    queues: Box<[QueueCreateInfo]>,
    graphics_compute_queue: QueueAllocation,
    transfer_queue: QueueAllocation,
    present_queues: Vec<(SurfaceId, QueueAllocation)>,
}