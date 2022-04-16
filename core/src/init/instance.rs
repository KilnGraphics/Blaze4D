use std::collections::{HashMap, HashSet};
use std::ffi::{c_void, CStr, CString};
use std::str::Utf8Error;
use std::sync::Arc;
use ash::vk;
use vk_profiles_rs::vp;
use crate::instance::{VulkanVersion, InstanceContextImpl, InstanceContext};
use crate::objects::surface::{SurfaceInitError, SurfaceProvider};
use crate::util::debug_messenger::DebugMessengerCallback;

pub struct InstanceCreateConfig {
    profile: vp::ProfileProperties,
    min_api_version: VulkanVersion,
    application_name: CString,
    application_version: u32,
    surfaces: HashMap<String, Box<dyn SurfaceProvider>>,
    debug_messengers: Vec<DebugUtilsMessengerWrapper>,
    enable_validation: bool,
}

impl InstanceCreateConfig {
    pub fn new(profile: vp::ProfileProperties, min_api_version: VulkanVersion, application_name: CString, application_version: u32) -> Self {
        Self {
            profile,
            min_api_version,
            application_name,
            application_version,
            surfaces: HashMap::new(),
            debug_messengers: Vec::new(),
            enable_validation: false,
        }
    }

    pub fn request_min_api_version(&mut self, version: VulkanVersion) {
        if self.min_api_version < version {
            self.min_api_version = version;
        }
    }

    pub fn add_surface_provider(&mut self, name: String, surface: Box<dyn SurfaceProvider>) -> Result<(), ()> {
        if self.surfaces.contains_key(&name) {
            Err(())
        } else {
            self.surfaces.insert(name, surface);
            Ok(())
        }
    }

    pub fn add_debug_messenger(&mut self, messenger: Box<dyn DebugMessengerCallback>) {
        self.debug_messengers.push(DebugUtilsMessengerWrapper{ callback: messenger });
    }

    pub fn enable_validation(&mut self) {
        self.enable_validation = true;
    }
}

#[derive(Debug)]
pub enum InstanceCreateError {
    Vulkan(vk::Result),
    ProfileNotSupported,
    MissingExtension(CString),
    Utf8Error(Utf8Error),
    SurfaceInitError(SurfaceInitError),
}

impl From<vk::Result> for InstanceCreateError {
    fn from(result: vk::Result) -> Self {
        InstanceCreateError::Vulkan(result)
    }
}

impl From<Utf8Error> for InstanceCreateError {
    fn from(err: Utf8Error) -> Self {
        InstanceCreateError::Utf8Error(err)
    }
}

pub fn create_instance(config: InstanceCreateConfig) -> Result<InstanceContext, InstanceCreateError> {
    let entry = ash::Entry::linked();
    let vp_fn = vk_profiles_rs::VulkanProfiles::linked();

    if !unsafe { vp_fn.get_instance_profile_support(None, &config.profile)? } {
        return Err(InstanceCreateError::ProfileNotSupported)
    }

    let mut required_extensions = HashSet::new();
    for surface in config.surfaces.values() {
        required_extensions.extend(surface.get_required_instance_extensions());
    }

    if !config.debug_messengers.is_empty() {
        required_extensions.insert(CString::from(CStr::from_bytes_with_nul(b"VK_EXT_debug_utils\0").unwrap()));
    }

    let available_extensions: HashSet<_> = entry.enumerate_instance_extension_properties(None)?
        .into_iter().map(|ext| {
            CString::from(unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) })
        }).collect();

    let mut required_extensions_str = Vec::with_capacity(required_extensions.len());
    for name in &required_extensions {
        if available_extensions.contains(name) {
            required_extensions_str.push(name.as_c_str().as_ptr())
        } else {
            return Err(InstanceCreateError::MissingExtension(name.clone()));
        }
    }

    let required_layers = if config.enable_validation {
        vec![CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0").unwrap().as_ptr()]
    } else {
        Vec::new()
    };

    let application_info = vk::ApplicationInfo::builder()
        .application_name(config.application_name.as_c_str())
        .application_version(config.application_version)
        .engine_name(CStr::from_bytes_with_nul(b"Blaze4D\0").unwrap())
        .engine_version(vk::make_api_version(0, 0, 1, 0))
        .api_version(config.min_api_version.into());

    let mut instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_layer_names(required_layers.as_slice())
        .enabled_extension_names(required_extensions_str.as_slice());

    let debug_messengers = config.debug_messengers.into_boxed_slice();
    let mut debug_messenger_create_infos: Vec<_> = debug_messengers.iter().map(|messenger| {
        vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::INFO | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE | vk::DebugUtilsMessageTypeFlagsEXT::GENERAL)
            .pfn_user_callback(Some(debug_utils_messenger_callback_wrapper))
            // Sadly this const to mut cast is necessary since the callback provides a mut pointer
            .user_data(messenger as *const DebugUtilsMessengerWrapper as *mut DebugUtilsMessengerWrapper as *mut c_void)
    }).collect();
    for debug_messenger in debug_messenger_create_infos.iter_mut() {
        instance_create_info = instance_create_info.push_next(debug_messenger);
    }

    let vp_instance_create_info = vp::InstanceCreateInfo::builder()
        .profile(&config.profile)
        .create_info(&instance_create_info)
        .flags(vp::InstanceCreateFlagBits::MERGE_EXTENSIONS);

    let instance = unsafe { vp_fn.create_instance(&entry, &vp_instance_create_info, None) }?;

    let surface_khr = if required_extensions.contains(CStr::from_bytes_with_nul(b"VK_KHR_surface\0").unwrap()) {
        Some(ash::extensions::khr::Surface::new(&entry, &instance))
    } else {
        None
    };

    let mut surfaces = config.surfaces;
    if let Err(error) = init_surfaces(&entry, &instance, &mut surfaces) {
        // Destroy initialized surfaces first then destroy the instance
        drop(surfaces);
        unsafe { instance.destroy_instance(None) };
        return Err(InstanceCreateError::SurfaceInitError(error));
    }

    Ok(Arc::new(InstanceContextImpl::new(
        config.min_api_version,
        config.profile,
        entry,
        instance,
        surface_khr,
        surfaces,
        debug_messengers
    )))
}

fn init_surfaces(entry: &ash::Entry, instance: &ash::Instance, surfaces: &mut HashMap<String, Box<dyn SurfaceProvider>>) -> Result<(), SurfaceInitError> {
    for surface in surfaces.values_mut() {
        surface.init(entry, instance)?;
    }
    Ok(())
}

pub struct DebugUtilsMessengerWrapper {
    callback: Box<dyn DebugMessengerCallback>
}

extern "system" fn debug_utils_messenger_callback_wrapper(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut c_void
) -> vk::Bool32 {
    if let Some(callback) = unsafe { (p_user_data as *const DebugUtilsMessengerWrapper).as_ref() } {
        let data = unsafe { p_callback_data.as_ref().unwrap() };
        let message = unsafe { CStr::from_ptr(data.p_message) };

        callback.callback.on_message(message_severity, message_types, message, data);
    } else {
        log::warn!("Wrapped debug utils messenger was called with null user data!");
    }

    return vk::FALSE;
}