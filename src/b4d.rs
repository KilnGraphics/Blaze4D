use std::ffi::CString;
use std::sync::Arc;
use std::thread::JoinHandle;
use ash::vk;
use vk_profiles_rs::vp;
use crate::debug::{DebugOverlay, DebugRenderer};
use crate::glfw_surface::GLFWSurfaceProvider;
use crate::prelude::Vec2u32;
use crate::renderer::B4DRenderWorker;
use crate::renderer::emulator::EmulatorRenderer;
use crate::transfer::Transfer;
use crate::instance::debug_messenger::RustLogDebugMessenger;
use crate::device::init::{create_device, DeviceCreateConfig};
use crate::instance::init::{create_instance, InstanceCreateConfig};
use crate::instance::instance::VulkanVersion;
use crate::vk::{DeviceEnvironment, InstanceContext};
use crate::vk::objects::surface::{SurfaceId, SurfaceProvider};

pub struct Blaze4D {
    instance: Arc<InstanceContext>,
    device: DeviceEnvironment,
    transfer: Transfer,
    main_surface: SurfaceId,
    emulator: EmulatorRenderer,
    worker: JoinHandle<()>,
}

impl Blaze4D {
    pub fn new(main_window: Box<dyn SurfaceProvider>) -> Self {
        crate::debug::text::ldfnt();

        let mut instance_config = InstanceCreateConfig::new(
            vp::KhrRoadmap2022::profile_properties(),
            VulkanVersion::VK_1_3,
            CString::new("Minecraft").unwrap(),
            vk::make_api_version(0, 0, 1, 0)
        );
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

        let emulator = EmulatorRenderer::new(device.clone(), transfer.clone());

        Self {
            instance,
            device,
            main_surface,
            transfer,
            emulator,
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