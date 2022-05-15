use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::sync::Arc;
use ash::vk;
use ash::vk::PhysicalDeviceType;
use vk_profiles_rs::vp;
use crate::device::device::{DeviceEnvironment, VkQueueTemplate};
use crate::instance::instance::InstanceContext;
use crate::prelude::DeviceContext;
use crate::vk::objects::types::SurfaceId;
use crate::vk::objects::surface::SurfaceProvider;

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

pub fn create_device(config: DeviceCreateConfig, instance: Arc<InstanceContext>) -> Result<DeviceEnvironment, DeviceCreateError> {
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
        &*instance,
        &required_extensions,
        &surfaces,
        config.rating_fn.as_ref()
    )?;


    let mut features1_2 = vk::PhysicalDeviceVulkan12Features::builder().build();
    unsafe { vk_vp.get_profile_features(instance.get_profile(), &mut features1_2) };
    features1_2.vulkan_memory_model_availability_visibility_chains = vk::FALSE;

    let required_extensions_str: Vec<_> = required_extensions.iter().map(|ext| ext.as_c_str().as_ptr()).collect();

    let device_queue_create_infos: Vec<_> = selected_device.queues.queue_create_infos.iter().map(|queue_info|
        queue_info.build().build()
    ).collect();
    let vk_device_create_info = vk::DeviceCreateInfo::builder()
        .enabled_extension_names(required_extensions_str.as_slice())
        .queue_create_infos(device_queue_create_infos.as_slice())
        .push_next(&mut features1_2);

    let flags = if config.disable_robustness {
        vp::DeviceCreateFlagBits::MERGE_EXTENSIONS | vp::DeviceCreateFlagBits::DISABLE_ROBUST_ACCESS | vp::DeviceCreateFlagBits::OVERRIDE_FEATURES
    } else {
        vp::DeviceCreateFlagBits::MERGE_EXTENSIONS | vp::DeviceCreateFlagBits::OVERRIDE_FEATURES
    };
    let vp_device_create_info = vp::DeviceCreateInfo::builder()
        .profile(instance.get_profile())
        .create_info(&vk_device_create_info)
        .flags(flags);

    let device = unsafe { vk_vp.create_device(instance.vk(), selected_device.device, &vp_device_create_info, None)? };

    let queue_map = QueueMap::new(&device, selected_device.queues.queue_create_infos.as_ref());

    let main_queue = queue_map.get_queue(selected_device.queues.main_queue);
    let transfer_queue = queue_map.get_queue(selected_device.queues.transfer_queue);


    let mut surfaces: HashMap<_, _> = surfaces.into_iter().collect();
    let mut out_surfaces = HashMap::new();
    for surface_config in selected_device.surfaces {
        let surface = surfaces.remove(&surface_config.id).unwrap();

        out_surfaces.insert(surface_config.id, (surface, surface_config.present_supported));
    }

    let swapchain_khr = if has_swapchain {
        Some(ash::extensions::khr::Swapchain::new(instance.vk(), &device))
    } else {
        None
    };


    Ok(DeviceEnvironment::new(DeviceContext::new(
        instance,
        device,
        selected_device.device,
        swapchain_khr,
        main_queue,
        transfer_queue,
        out_surfaces
    )))
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
        // TODO re-add this. Temporary workaround for AMD missing support
        //return Ok(None);
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

    // Generate surface data
    let mut surface_infos: Vec<_> = Vec::with_capacity(surfaces.len());
    for (id, surface) in surfaces.iter() {
        let handle = surface.get_handle().unwrap();
        let surface_khr = instance.surface_khr().unwrap();

        let mut supported_queues = Vec::with_capacity(queue_family_properties.len());
        for family in 0u32..(queue_family_properties.len() as u32) {
            supported_queues.push(unsafe {
                surface_khr.get_physical_device_surface_support(device, family, handle)
            }?);
        }

        surface_infos.push(PhysicalDeviceSurfaceConfig {
            id: *id,
            present_supported: supported_queues.into_boxed_slice(),
        });
    }

    // Generate queue requests
    let queue_config = match generate_queue_allocation(queue_family_properties.as_slice(), surface_infos.as_slice()) {
        Some(config) => config,
        None => return Ok(None)
    };

    let rating = match rating_fn(instance, device) {
        Some(rating) => rating,
        None => return Ok(None)
    };

    Ok(Some(PhysicalDeviceConfig {
        device,
        rating,
        surfaces: surface_infos,
        queues: queue_config
    }))
}

fn generate_queue_allocation(
    properties: &[vk::QueueFamilyProperties2],
    surfaces: &[PhysicalDeviceSurfaceConfig],
) -> Option<PhysicalDeviceQueueConfig> {
    let main_family = properties.iter().enumerate().filter_map(|(family, props)| {
        let props = &props.queue_family_properties;
        if !props.queue_flags.contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE) {
            return None;
        }

        for surface in surfaces.iter() {
            if !surface.present_supported.get(family).unwrap() {
                return None;
            }
        }

        let has_sparse_binding = props.queue_flags.contains(vk::QueueFlags::SPARSE_BINDING);

        Some((family as u32, has_sparse_binding as u32))
    }).max_by(|a, b| {
        // Prefer queue families with sparse binding support
        a.1.cmp(&b.1)
    })?.0;

    let transfer_family = properties.iter().enumerate().filter_map(|(family, props)| {
        if (family as u32) == main_family {
            return None;
        }

        let props = &props.queue_family_properties;
        if !props.queue_flags.contains(vk::QueueFlags::TRANSFER) {
            return None;
        }

        let has_sparse_binding = props.queue_flags.contains(vk::QueueFlags::SPARSE_BINDING);

        Some((family as u32, has_sparse_binding as u32))
    }).max_by(|a, b| {
        // Prefer queue families with sparse binding support
        a.1.cmp(&b.1)
    })?.0;

    if main_family != transfer_family {
        Some(PhysicalDeviceQueueConfig {
            main_queue: QueueAllocation { family: main_family, index: 0 },
            transfer_queue: QueueAllocation { family: transfer_family, index: 0 },
            queue_create_infos: vec![QueueCreateInfo::new(main_family, 1), QueueCreateInfo::new(transfer_family, 1)]
        })

    } else {
        let transfer_index;
        let create_count;
        if properties.get(main_family as usize).unwrap().queue_family_properties.queue_count == 1 {
            transfer_index = 0;
            create_count = 1;
        } else {
            transfer_index = 1;
            create_count = 2;
        }

        Some(PhysicalDeviceQueueConfig {
            main_queue: QueueAllocation { family: main_family, index: 0 },
            transfer_queue: QueueAllocation { family: main_family, index: transfer_index },
            queue_create_infos: vec![QueueCreateInfo::new(main_family, create_count)]
        })
    }
}

struct QueueMap {
    queues: HashMap<u32, Box<[VkQueueTemplate]>>,
}

impl QueueMap {
    fn new(device: &ash::Device, queues: &[QueueCreateInfo]) -> Self {
        let mut map = HashMap::new();

        for queue in queues.iter() {
            let mut vec = Vec::with_capacity(queue.priorities.len());
            for index in 0..(queue.priorities.len() as u32) {
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
    surfaces: Vec<PhysicalDeviceSurfaceConfig>,
    queues: PhysicalDeviceQueueConfig,
}

#[derive(Debug)]
struct PhysicalDeviceSurfaceConfig {
    id: SurfaceId,
    present_supported: Box<[bool]>,
}

#[derive(Debug)]
struct PhysicalDeviceQueueConfig {
    main_queue: QueueAllocation,
    transfer_queue: QueueAllocation,
    queue_create_infos: Vec<QueueCreateInfo>,
}

#[derive(Debug)]
struct QueueCreateInfo {
    family: u32,
    priorities: Box<[f32]>,
}

impl QueueCreateInfo {
    fn new(family: u32, count: u32) -> Self {
        let priorities: Box<[f32]> = std::iter::repeat(1.0f32).take(count as usize).collect();

        Self {
            family,
            priorities,
        }
    }

    fn build(&self) -> vk::DeviceQueueCreateInfoBuilder {
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(self.family)
            .queue_priorities(self.priorities.as_ref())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct QueueAllocation {
    family: u32,
    index: u32,
}