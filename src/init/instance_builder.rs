use std::ffi::{CStr, CString};
use ash::{Entry, Instance};
use ash::extensions::ext::DebugUtils;
use ash::vk::{ApplicationInfo, InstanceCreateInfo, make_api_version};
use crate::init::initialization_registry::InitializationRegistry;
use crate::window::RosellaWindow;

pub struct InstanceBuilder {
    registry: InitializationRegistry,
    enable_debug_utils: bool,
    vk: Entry,
}

impl InstanceBuilder {
    pub fn new(registry: InitializationRegistry) -> Self {
        InstanceBuilder {
            registry,
            enable_debug_utils: true,
            vk: Entry::new(),
        }
    }

    fn get_supported_version(&self) -> u32 {
        self.vk.try_enumerate_instance_version().unwrap().expect("Failed to enumerate over instance versions")
    }

    pub fn build(&self, application_name: &str, application_version: u32, window: &RosellaWindow) -> Instance {
        if self.get_supported_version() < self.registry.min_required_version {
            panic!("Minimum vulkan version {} is not supported!", self.registry.min_required_version)
        }

        let app_name = CString::new(application_name).unwrap();
        let app_info = ApplicationInfo::builder()
            .application_name(app_name.as_c_str())
            .application_version(application_version)
            .engine_name(CString::new("Rosella").unwrap().as_c_str())
            .engine_version(make_api_version(2, 0, 0, 0))
            .api_version(self.registry.max_supported_version)
            .build();

        // FIXME: do this properly. Just get it to compile for now
        let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
        let layers_names_raw: Vec<*const i8> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let surface_extensions = ash_window::enumerate_required_extensions(&window.handle).unwrap();
        let mut extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();
        extension_names_raw.push(DebugUtils::name().as_ptr());

        let create_info = InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names_raw)
            .build();
            // .push_next(createDebugUtilsCallback(VK10.VK_NULL_HANDLE));

        unsafe { self.vk.create_instance(&create_info, None).expect("Failed to create a Vulkan Instance.") }
    }
}

