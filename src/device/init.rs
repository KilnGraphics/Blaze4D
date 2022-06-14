use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;
use ash::prelude::VkResult;

use ash::vk;
use bumpalo::Bump;
use vk_profiles_rs::{vp, VulkanProfiles};

use crate::device::device::{DeviceFunctions, Queue};
use crate::device::surface::DeviceSurface;
use crate::instance::instance::{InstanceContext, VulkanVersion};
use crate::objects::id::SurfaceId;
use crate::vk::objects::surface::SurfaceProvider;

use crate::prelude::*;

#[derive(Debug)]
pub struct DeviceCreateConfig {
    used_surfaces: Vec<vk::SurfaceKHR>,
    disable_robustness: bool,
    required_extensions: HashSet<CString>,
}

impl DeviceCreateConfig {
    pub fn new() -> Self {
        Self {
            used_surfaces: Vec::new(),
            required_extensions: HashSet::new(),
            disable_robustness: false,
        }
    }

    pub fn add_surface(&mut self, surface: vk::SurfaceKHR) {
        self.used_surfaces.push(surface);
    }

    pub fn disable_robustness(&mut self) {
        self.disable_robustness = true;
    }

    pub fn add_required_extension(&mut self, extension: &CStr) {
        self.required_extensions.insert(CString::from(extension));
    }

    pub fn require_swapchain(&mut self) {
        self.required_extensions.insert(CString::new("VK_KHR_swapchain").unwrap());
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

pub fn create_device(mut config: DeviceCreateConfig, instance: Arc<InstanceContext>) -> Result<Arc<DeviceContext>, DeviceCreateError> {
    log::info!("Creating vulkan device with config: {:?}", config);

    let vk_vp = VulkanProfiles::linked();

    let has_swapchain = config.required_extensions.contains(&CString::new("VK_KHR_swapchain").unwrap());

    let allocator = Bump::new();

    let (device_config, device_create_info, physical_device) = filter_devices(
        unsafe { instance.vk().enumerate_physical_devices()? },
        &instance,
        &vk_vp,
        &config,
        &allocator
    )?;

    let priority = 1f32;
    let mut queue_create_infos = Vec::with_capacity(3);
    queue_create_infos.push(vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(device_config.main_queue_family)
        .queue_priorities(std::slice::from_ref(&priority))
        .build()
    );
    if let Some(family) = &device_config.async_compute_family {
        queue_create_infos.push(vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(*family)
            .queue_priorities(std::slice::from_ref(&priority))
            .build()
        );
    }
    if let Some(family) = &device_config.async_transfer_family {
        queue_create_infos.push(vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(*family)
            .queue_priorities(std::slice::from_ref(&priority))
            .build()
        );
    }

    let device_create_info = device_create_info.queue_create_infos(queue_create_infos.as_slice());

    let mut flags = vp::DeviceCreateFlagBits::MERGE_EXTENSIONS | vp::DeviceCreateFlagBits::OVERRIDE_FEATURES;
    if config.disable_robustness {
        flags |= vp::DeviceCreateFlagBits::DISABLE_ROBUST_ACCESS;
    }

    let vp_device_create_info = vp::DeviceCreateInfo::builder()
        .profile(instance.get_profile())
        .create_info(&device_create_info)
        .flags(flags);

    let device = unsafe { vk_vp.create_device(instance.vk(), physical_device, &vp_device_create_info, None)? };

    let synchronization_2_khr = ash::extensions::khr::Synchronization2::new(instance.vk(), &device);
    let timeline_semaphore_khr = ash::extensions::khr::TimelineSemaphore::new(instance.vk(), &device);
    let push_descriptor_khr = ash::extensions::khr::PushDescriptor::new(instance.vk(), &device);

    let swapchain_khr = if has_swapchain {
        Some(ash::extensions::khr::Swapchain::new(instance.vk(), &device))
    } else {
        None
    };

    let maintenance_4_khr = if device_config.has_maintenance4 {
        Some(ash::extensions::khr::Maintenance4::new(instance.vk(), &device))
    } else {
        None
    };

    let functions = Arc::new(DeviceFunctions {
        instance,
        physical_device,
        device,
        synchronization_2_khr,
        timeline_semaphore_khr,
        push_descriptor_khr,
        swapchain_khr,
        maintenance_4_khr
    });

    let main_queue = Arc::new(Queue::new(functions.clone(), device_config.main_queue_family, 0));
    let async_compute_queue = device_config.async_compute_family.map(|family| {
        Arc::new(Queue::new(functions.clone(), family, 0))
    });
    let async_transfer_queue = device_config.async_transfer_family.map(|family| {
        Arc::new(Queue::new(functions.clone(), family, 0))
    });

    Ok(DeviceContext::new(
        functions,
        main_queue,
        async_compute_queue,
        async_transfer_queue
    ))
}

fn filter_devices<'a>(
    devices: Vec<vk::PhysicalDevice>,
    instance: &InstanceContext,
    vk_vp: &VulkanProfiles,
    config: &DeviceCreateConfig,
    allocator: &'a Bump
) -> Result<(DeviceConfigInfo, vk::DeviceCreateInfoBuilder<'a>, vk::PhysicalDevice), DeviceCreateError> {
    let profile = instance.get_profile();

    let mut best_device = None;
    for device in devices {
        if let Some(mut configurator) = DeviceConfigurator::new(
            instance,
            vk_vp,
            config,
            profile,
            device,
            allocator
        )? {
            if let Some(device_config) = configure_device(&mut configurator)? {
                best_device = if let Some(old) = best_device {
                    if device_config.rating > old.0.rating {
                        Some((device_config, configurator.build(), device))
                    } else {
                        Some(old)
                    }
                } else {
                    Some((device_config, configurator.build(), device))
                }
            }
        }
    }

    best_device.ok_or(DeviceCreateError::NoSupportedDevice)
}

struct DeviceConfigurator<'a, 'b> {
    instance: &'a InstanceContext,
    config: &'a DeviceCreateConfig,
    physical_device: vk::PhysicalDevice,
    device_name: CString,
    available_extensions: HashSet<CString>,
    used_extensions: HashSet<CString>,
    queue_family_surface_support: Box<[bool]>,
    alloc: &'b Bump,
    create_info: vk::DeviceCreateInfoBuilder<'b>,
}

impl<'a, 'b> DeviceConfigurator<'a, 'b> {
    fn new(instance: &InstanceContext, vk_vp: &VulkanProfiles, config: &DeviceCreateConfig, profile: &vp::ProfileProperties, physical_device: vk::PhysicalDevice, alloc: &Bump) -> Result<Option<Self>, DeviceCreateError> {
        let properties = unsafe {
            instance.vk().get_physical_device_properties(physical_device)
        };
        let device_name = CString::from(unsafe { CStr::from_ptr(properties.device_name.as_ptr()) });
        log::info!("Checking physical device {:?} {:?}", device_name, VulkanVersion::from_raw(properties.api_version));

        let used_extensions = config.required_extensions.clone();

        let available_extensions: HashSet<_> = unsafe { instance.vk().enumerate_device_extension_properties(physical_device)? }
            .into_iter().map(|ext| {
            CString::from(unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) })
        }).collect();

        if !used_extensions.is_subset(&available_extensions) {
            log::info!("Physical device {:?} is missing required extensions {:?}",
                device_name,
                used_extensions.difference(&available_extensions)
            );
            return Ok(None);
        }

        if !unsafe { vk_vp.get_physical_device_profile_support(instance.vk(), physical_device, profile) }? {
            log::info!("Physical device {:?} does not support used profile", device_name);
            return Ok(None);
        }

        let queue_family_count = unsafe {
            instance.vk().get_physical_device_queue_family_properties(physical_device)
        }.len();

        let mut queue_family_surface_support = std::iter::repeat(true).take(queue_family_count).collect();
        for surface in &config.used_surfaces {
            let surface_khr = instance.surface_khr().unwrap();
            for (index, support) in queue_family_surface_support.iter_mut().enumerate() {
                if !unsafe { surface_khr.get_physical_device_surface_support(physical_device, index as u32, *surface) }? {
                    *support = false;
                }
            }
        }

        if !queue_family_surface_support.iter().any(|b| b) {
            log::info!("Physical device {:?} does not contain queue family which supports all surfaces", device_name);
            return Ok(None);
        }

        Ok(Some(DeviceConfigurator {
            instance,
            config,
            physical_device,
            device_name,
            available_extensions,
            used_extensions,
            queue_family_surface_support,
            alloc,
            create_info: vk::DeviceCreateInfo::builder()
        }))
    }

    fn get_name(&self) -> &CStr {
        self.device_name.as_c_str()
    }

    fn get_properties(&self, mut properties: vk::PhysicalDeviceProperties2Builder) -> vk::PhysicalDeviceProperties {
        unsafe {
            self.instance.vk().get_physical_device_properties2(self.physical_device, &mut properties)
        };
        properties.properties
    }

    fn get_features(&self, mut features: vk::PhysicalDeviceFeatures2Builder) -> vk::PhysicalDeviceFeatures {
        unsafe {
            self.instance.vk().get_physical_device_features2(self.physical_device, &mut features)
        };
        features.features
    }

    fn filter_sort_queues<F: Fn(u32, &vk::QueueFamilyProperties, bool) -> Option<u32>>(&self, func: F) -> Vec<u32> {
        let properties = unsafe {
            self.instance.vk().get_physical_device_queue_family_properties(self.physical_device)
        };

        let mut families = Vec::with_capacity(properties.len());
        for (family, properties) in properties.iter().enumerate() {
            let surface_supported = self.queue_family_surface_support[family];

            if let Some(value) = func(family as u32, properties, surface_supported) {
                families.push((family as u32, value));
            }
        }

        families.sort_by(|(a, _), (b, _)| a.cmp(b));
        families.into_iter().map(|(_, family)| *family).collect()
    }

    /// Checks if the extension is supported and if so adds it to the list of used extensions.
    ///
    /// Returns true if the extension is supported.
    fn add_extension(&mut self, name: &CStr) -> bool {
        if self.available_extensions.contains(name) {
            self.used_extensions.insert(CString::from(name));
            true
        } else {
            false
        }
    }

    fn allocate<T: 'b>(&self, data: T) -> &'b mut T {
        self.alloc.alloc(data)
    }

    fn push_next<T: vk::ExtendsDeviceCreateInfo + 'b>(&mut self, data: T) {
        let data = self.alloc.alloc(data);
        self.create_info = self.create_info.push_next(data);
    }

    fn build(mut self) -> vk::DeviceCreateInfoBuilder<'b> {
        let c_extensions = self.alloc.alloc_slice_fill_copy(self.used_extensions.len(), std::ptr::null());

        for (index, extension) in self.used_extensions.iter().enumerate() {
            let c_str = self.alloc.alloc_slice_copy(extension.as_bytes()).as_ptr() as *const c_char;
            c_extensions[index] = c_str;
        }

        self.create_info.enabled_extension_names(c_extensions)
    }
}

struct DeviceConfigInfo {
    rating: f32,
    has_maintenance4: bool,

    /// The main queue family. It is guaranteed to support presentation to all surfaces as well as
    /// graphics, compute and transfer operations.
    main_queue_family: u32,

    /// The queue family used for async compute operations. It is guaranteed to support compute and
    /// transfer operations and must be a different queue family than the main queue family.
    async_compute_family: Option<u32>,

    /// The queue family used for async transfer operations. It is guaranteed to support transfer
    /// operations and must be a different queue family than both the main and compute queue family.
    async_transfer_family: Option<u32>,
}

fn configure_device(device: &mut DeviceConfigurator) -> Result<Option<DeviceConfigInfo>, DeviceCreateError> {
    // Any device features/properties we need to validate get pushed into this p_next chain
    let mut features = vk::PhysicalDeviceFeatures2::builder();
    let mut properties = vk::PhysicalDeviceProperties2::builder();

    if !device.add_extension(&CString::new("VK_KHR_synchronization2").unwrap()) {
        log::info!("Physical device {:?} does not support VK_KHR_synchronization2", device.get_name());
        return Ok(None);
    }
    if !device.add_extension(&CString::new("VK_KHR_push_descriptor").unwrap()) {
        log::info!("Physical device {:?} does not support VK_KHR_push_descriptor", device.get_name());
        return Ok(None);
    }

    let mut maintenance_4;
    if !device.add_extension(&CString::new("VK_KHR_maintenance4").unwrap()) {
        maintenance_4 = Some((
            vk::PhysicalDeviceMaintenance4Features::builder(),
            vk::PhysicalDeviceMaintenance4Properties::builder()
        ));
        let (f, p) = maintenance_4.as_mut().unwrap();
        features.push_next(f);
        properties.push_next(p);
    } else {
        maintenance_4 = None;
    }

    let mut timeline_features = vk::PhysicalDeviceTimelineSemaphoreFeatures::builder();
    features = features.push_next(&mut timeline_features);

    let mut timeline_properties = vk::PhysicalDeviceTimelineSemaphoreProperties::builder();
    properties = properties.push_next(&mut timeline_properties);

    let mut push_descriptor_properties = vk::PhysicalDevicePushDescriptorPropertiesKHR::builder();
    properties = properties.push_next(&mut push_descriptor_properties);

    // Read supported features and properties
    device.get_features(features);
    device.get_properties(properties);
    let timeline_features = timeline_features.build();
    let timeline_properties = timeline_properties.build();
    let push_descriptor_properties = push_descriptor_properties.build();
    let maintenance_4 = maintenance_4.map(|(f, p)| (f.build(), p.build()));

    // Process the supported features and properties
    if timeline_features.timeline_semaphore != vk::TRUE {
        log::info!("Physical device {:?} does not support the timeline semaphore feature", device.get_name());
        return Ok(None);
    } else {
        device.push_next(vk::PhysicalDeviceTimelineSemaphoreFeatures::builder()
            .timeline_semaphore(true)
        );
    }

    if timeline_properties.max_timeline_semaphore_value_difference < u8::MAX as u64 {
        log::info!("Physical device {:?} max_timeline_semaphore_value_difference is too low {:?}", device.get_name(), timeline_properties.max_timeline_semaphore_value_difference);
        return Ok(None);
    }

    if push_descriptor_properties.max_push_descriptors < 8 {
        log::info!("Physical device {:?} max_push_descriptors is too low {:?}", device.get_name(), push_descriptor_properties.max_push_descriptors);
        return Ok(None);
    }

    // Calculate queue family assignments
    let main_families = device.filter_sort_queues(|(family, properties, surface_support)| {
        Some(family)
    });
    let main_queue_family;
    if let Some(family) = main_families.get(0) {
        main_queue_family = *family;
    } else {
        log::info!("Physical device {:?} does not have suitable main queue family", device.get_name());
        return Ok(None);
    }

    Ok(Some(DeviceConfigInfo {
        rating: 0.0,
        has_maintenance4: maintenance_4.is_some(),
        main_queue_family,
        async_compute_family: None,
        async_transfer_family: None
    }))
}