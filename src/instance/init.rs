use std::collections::HashSet;
use std::ffi::{c_void, CStr, CString};
use std::fmt::Debug;
use std::str::Utf8Error;
use std::sync::Arc;

use ash::vk;

use vk_profiles_rs::vp;
use winit::event::VirtualKeyCode::V;
use crate::{B4D_CORE_VERSION_MAJOR, B4D_CORE_VERSION_MINOR, B4D_CORE_VERSION_PATCH};

use crate::instance::instance::{VulkanVersion, InstanceContext};
use crate::vk::objects::surface::{SurfaceInitError, SurfaceProvider};
use crate::instance::debug_messenger::DebugMessengerCallback;
use crate::objects::id::SurfaceId;

#[derive(Debug)]
pub struct InstanceCreateConfig {
    application_name: CString,
    application_version: u32,
    debug_messengers: Vec<DebugUtilsMessengerWrapper>,
    enable_validation: bool,
    required_extensions: HashSet<CString>,
    require_surface_khr: bool,
}

impl InstanceCreateConfig {
    pub fn new(application_name: CString, application_version: u32) -> Self {
        Self {
            application_name,
            application_version,
            debug_messengers: Vec::new(),
            enable_validation: false,
            required_extensions: HashSet::new(),
            require_surface_khr: false,
        }
    }

    pub fn add_debug_messenger(&mut self, messenger: Box<dyn DebugMessengerCallback>) {
        self.debug_messengers.push(DebugUtilsMessengerWrapper{ callback: messenger });
    }

    pub fn enable_validation(&mut self) {
        self.enable_validation = true;
    }

    pub fn add_required_extension(&mut self, extension: &CStr) {
        self.required_extensions.insert(CString::from(extension));
    }

    pub fn require_surface_khr(&mut self) {
        self.require_surface_khr = true;
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

pub fn create_instance(config: InstanceCreateConfig) -> Result<Arc<InstanceContext>, InstanceCreateError> {
    log::info!("Creating vulkan instance with config: {:?}", config);

    let profile = vp::LunargDesktopPortability2021::profile_properties();

    let entry = ash::Entry::linked();
    let vp_fn = vk_profiles_rs::VulkanProfiles::linked();

    let vulkan_version;
    if let Some(version) = entry.try_enumerate_instance_version()? {
        vulkan_version = VulkanVersion::from_raw(version);
    } else {
        vulkan_version = VulkanVersion::VK_1_0;
    }
    log::info!("Vulkan instance version: {:?}", vulkan_version);

    log::info!("Using profile {:?} for instance creation", unsafe { CStr::from_ptr(profile.profile_name.as_ptr()) });
    if !unsafe { vp_fn.get_instance_profile_support(None, &profile)? } {
        return Err(InstanceCreateError::ProfileNotSupported)
    }

    let mut required_extensions = config.required_extensions;
    if config.require_surface_khr {
        required_extensions.insert(CString::from(CStr::from_bytes_with_nul(b"VK_KHR_surface\0").unwrap()));
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
        log::info!("Validation layers enabled");
        vec![CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0").unwrap().as_ptr()]
    } else {
        log::info!("Validation layers disabled");
        Vec::new()
    };

    let application_info = vk::ApplicationInfo::builder()
        .application_name(config.application_name.as_c_str())
        .application_version(config.application_version)
        .engine_name(CStr::from_bytes_with_nul(b"Blaze4D-Core\0").unwrap())
        .engine_version(vk::make_api_version(0, B4D_CORE_VERSION_MAJOR, B4D_CORE_VERSION_MINOR, B4D_CORE_VERSION_PATCH))
        .api_version(config.api_version.into());

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

    Ok(InstanceContext::new(
        vulkan_version,
        profile,
        entry,
        instance,
        surface_khr,
        debug_messengers
    ))
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
    std::panic::catch_unwind(|| {
        if let Some(callback) = unsafe { (p_user_data as *const DebugUtilsMessengerWrapper).as_ref() } {
            let data = unsafe {
                p_callback_data.as_ref().unwrap_or_else(|| std::process::abort()) // If this is null something went very wrong
            };
            let message = unsafe { CStr::from_ptr(data.p_message) };

            // This is called by c code so we must catch any panics
            callback.callback.on_message(message_severity, message_types, message, data);
        } else {
            log::warn!("Wrapped debug utils messenger was called with null user data!");
        }
    }).unwrap_or_else(|_| {
        log::error!("Debug utils messenger panicked! Aborting...");
        // TODO is there a better way to deal with this?
        std::process::exit(1);
    });

    return vk::FALSE;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_init() {
        let config = InstanceCreateConfig::new(
            CString::from(CStr::from_bytes_with_nul(b"B4DCoreTest\0").unwrap()),
            1,
        );

        let instance = create_instance(config).unwrap();
    }
}