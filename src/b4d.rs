use std::ffi::CString;
use std::thread::JoinHandle;
use ash::vk;
use vk_profiles_rs::vp;
use crate::debug::{DebugOverlay, DebugRenderer};
use crate::glfw_surface::GLFWSurfaceProvider;
use crate::prelude::Vec2u32;
use crate::renderer::B4DRenderWorker;
use crate::transfer::Transfer;
use crate::vk::debug_messenger::RustLogDebugMessenger;
use crate::vk::init::device::{create_device, DeviceCreateConfig};
use crate::vk::init::instance::{create_instance, InstanceCreateConfig};
use crate::vk::instance::VulkanVersion;
use crate::vk::{DeviceContext, InstanceContext};
use crate::vk::objects::surface::SurfaceProvider;
use crate::vk::objects::types::SurfaceId;

pub struct Blaze4D {
    instance: InstanceContext,
    device: DeviceContext,
    transfer: Transfer,
    main_surface: SurfaceId,
    worker: JoinHandle<()>,
}

impl Blaze4D {
    pub fn new(main_window: Box<dyn SurfaceProvider>) -> Self {
        crate::debug::text::ldfnt();

        let mut instance_config = InstanceCreateConfig::new(
            vp::LunargDesktopPortability2021::profile_properties(),
            VulkanVersion::VK_1_1,
            CString::new("Minecraft").unwrap(),
            vk::make_api_version(0, 0, 1, 0)
        );
        instance_config.request_min_api_version(VulkanVersion::VK_1_3);
        instance_config.enable_validation();
        let main_surface = instance_config.add_surface_provider(main_window);
        instance_config.add_debug_messenger(Box::new(RustLogDebugMessenger::new()));

        let instance = create_instance(instance_config).unwrap();

        let mut device_config = DeviceCreateConfig::new();
        device_config.add_surface(main_surface);

        let device = create_device(device_config, instance.clone()).unwrap();

        let worker = B4DRenderWorker::new(device.clone(), main_surface);
        let handle = std::thread::spawn(move || worker.run());

        let overlay = DebugOverlay::new(device.clone());
        let target = overlay.create_target(Vec2u32::new(800, 600));

        let transfer = Transfer::new(device.clone());

        Self {
            instance,
            device,
            main_surface,
            transfer,
            worker: handle
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn b4d_init(surface: *mut GLFWSurfaceProvider) -> *mut Blaze4D {
    env_logger::init();

    let surface: Box<dyn SurfaceProvider> = Box::from_raw(surface);

    let b4d = Box::leak(Box::new(Blaze4D::new(surface)));

    b4d
}

#[no_mangle]
pub unsafe extern "C" fn b4d_destroy(b4d: *mut Blaze4D) {
    Box::from_raw(b4d);
}