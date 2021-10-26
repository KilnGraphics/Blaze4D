use std::ffi::CString;

use ash::extensions::ext::DebugUtils;
use ash::vk::{make_api_version, ApplicationInfo, InstanceCreateInfo};
use ash::{Entry, Instance};
use crate::ALLOCATION_CALLBACKS;
use crate::init::device::VulkanInstance;

use crate::init::initialization_registry::InitializationRegistry;
use crate::window::RosellaWindow;

pub fn create_instance(
    registry: &InitializationRegistry,
    application_name: &str,
    application_version: u32,
    window: &RosellaWindow,
) -> VulkanInstance {
    let vk = Entry::new();

    let supported_version = vk
        .try_enumerate_instance_version()
        .ok()
        .flatten()
        .expect("Failed to enumerate over instance versions");
    assert!(
        supported_version >= registry.min_required_version,
        "minimum vulkan version is not supported"
    );

    let app_name = CString::new(application_name).unwrap();
    let app_info = ApplicationInfo::builder()
        .application_name(app_name.as_c_str())
        .application_version(application_version)
        .engine_name(CString::new("Rosella").unwrap().as_c_str())
        .engine_version(make_api_version(2, 0, 0, 0))
        .api_version(registry.max_supported_version)
        .build();

    // FIXME: do this properly. Just get it to compile for now
    let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
    let layers_names_raw: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();

    let surface_extensions = ash_window::enumerate_required_extensions(&window.handle).unwrap();
    let mut extension_names_raw = surface_extensions.iter().map(|ext| ext.as_ptr()).collect::<Vec<_>>();
    extension_names_raw.push(DebugUtils::name().as_ptr());

    let create_info = InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&extension_names_raw)
        .build();
    // .push_next(createDebugUtilsCallback(VK10.VK_NULL_HANDLE));

    VulkanInstance {
        instance: unsafe { vk.create_instance(&create_info, ALLOCATION_CALLBACKS) }.expect("Failed to create a Vulkan Instance."),
        version: 0
    }
}
