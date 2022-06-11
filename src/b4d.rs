use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ash::vk;

use vk_profiles_rs::vp;

use crate::instance::debug_messenger::RustLogDebugMessenger;
use crate::device::init::{create_device, DeviceCreateConfig};
use crate::device::surface::{DeviceSurface, SurfaceSwapchain, SwapchainConfig};
use crate::instance::init::{create_instance, InstanceCreateConfig};
use crate::instance::instance::VulkanVersion;
use crate::vk::objects::surface::SurfaceProvider;

use crate::prelude::*;
use crate::renderer::emulator::{EmulatorRenderer, MeshData, StaticMeshId};
use crate::renderer::emulator::debug_pipeline::DebugPipeline;
use crate::renderer::emulator::PassRecorder;
use crate::renderer::emulator::pipeline::{EmulatorPipeline, SwapchainOutput};

pub struct Blaze4D {
    instance: Arc<InstanceContext>,
    device: DeviceEnvironment,
    emulator: Arc<EmulatorRenderer>,

    render_config: Mutex<RenderConfig>,
}

impl Blaze4D {
    /// Creates a new Blaze4D instance and starts all engine modules.
    ///
    /// The supported vertex formats for the [`EmulatorRenderer`] must be provided here.
    pub fn new(main_window: Box<dyn SurfaceProvider>, enable_validation: bool) -> Self {
        crate::debug::text::ldfnt();

        let mut instance_config = InstanceCreateConfig::new(
            vp::KhrRoadmap2022::profile_properties(),
            VulkanVersion::VK_1_3,
            CString::new("Minecraft").unwrap(),
            vk::make_api_version(0, 0, 1, 0)
        );
        if enable_validation {
            instance_config.enable_validation();
        }
        let main_surface = instance_config.add_surface_provider(main_window);
        instance_config.add_debug_messenger(Box::new(RustLogDebugMessenger::new()));

        let instance = create_instance(instance_config).unwrap();

        let mut device_config = DeviceCreateConfig::new();
        device_config.add_surface(main_surface);
        device_config.disable_robustness();

        let (device, surfaces) = create_device(device_config, instance.clone()).unwrap();
        let main_surface = surfaces.into_iter().fold(None, |res, (id, surface)| {
            if id == main_surface {
                Some(surface)
            } else {
                res
            }
        }).unwrap();

        let emulator = EmulatorRenderer::new(device.clone());

        let render_config = Mutex::new(RenderConfig::new(device.clone(), emulator.clone(), main_surface));

        Self {
            instance,
            device,
            emulator,

            render_config,
        }
    }

    pub fn create_static_mesh(&self, data: &MeshData) -> StaticMeshId {
        self.emulator.create_static_mesh(data)
    }

    pub fn drop_static_mesh(&self, id: StaticMeshId) {
        self.emulator.drop_static_mesh(id);
    }

    pub fn try_start_frame(&self, window_size: Vec2u32) -> Option<PassRecorder> {
        if let Some(recorder) = self.render_config.lock().unwrap().try_start_frame(&self.emulator, window_size) {
            Some(recorder)
        } else {
            None
        }
    }
}

struct RenderConfig {
    device: DeviceEnvironment,
    emulator: Arc<EmulatorRenderer>,
    main_surface: Arc<DeviceSurface>,

    last_rebuild: Instant,
    current_swapchain: Option<Arc<SurfaceSwapchain>>,
    current_pipeline: Option<(Arc<dyn EmulatorPipeline>, Arc<SwapchainOutput>)>,
}

impl RenderConfig {
    fn new(device: DeviceEnvironment, emulator: Arc<EmulatorRenderer>, main_surface: Arc<DeviceSurface>) -> Self {
        Self {
            device,
            emulator,
            main_surface,

            last_rebuild: Instant::now() - Duration::from_secs(100),
            current_swapchain: None,
            current_pipeline: None,
        }
    }

    fn try_start_frame(&mut self, renderer: &EmulatorRenderer, size: Vec2u32) -> Option<PassRecorder> {
        let mut force_rebuild = false;

        // This if block only exists because of wayland
        if let Some(current) = self.current_swapchain.as_ref() {
            if current.get_image_size() != size {
                force_rebuild = true;
            }
        }

        if self.current_swapchain.is_none() || force_rebuild {
            if !self.try_create_swapchain(size) {
                return None;
            }
            self.current_pipeline = None;
        }

        if self.current_pipeline.is_none() {
            log::info!("No pipeline present. Rebuilding for size {:?}", size);

            let pipeline = DebugPipeline::new(self.device.clone(), self.emulator.clone(), size);
            let swapchain_output = SwapchainOutput::new(&self.device, pipeline.clone(), self.current_swapchain.as_ref().unwrap().clone());

            self.current_pipeline = Some((pipeline, swapchain_output));
        }

        let (pipeline, output) = self.current_pipeline.as_ref().unwrap();

        let (output, suboptimal) = match output.next_image() {
            None => {
                self.current_pipeline = None;
                self.current_swapchain = None;
                return None;
            }
            Some(result) => result,
        };

        let mut recorder = renderer.start_pass(pipeline.clone());
        recorder.use_output(output);

        if suboptimal {
            self.current_pipeline = None;
            self.current_swapchain = None;
        }

        Some(recorder)
    }

    fn try_create_swapchain(&mut self, size: Vec2u32) -> bool {
        log::info!("Attempting to rebuild swapchain with size {:?}", size);

        let diff = (self.last_rebuild + Duration::from_millis(50)).saturating_duration_since(Instant::now());
        if !diff.is_zero() {
            // Need to wait
            std::thread::sleep(diff);
        }
        self.last_rebuild = Instant::now();

        let config = SwapchainConfig {
            allow_tearing: true, // We set this to true to unlock fps for testing
            formats: Box::new([
                vk::SurfaceFormatKHR{ format: vk::Format::R8G8B8A8_SRGB, color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR },
                vk::SurfaceFormatKHR{ format: vk::Format::B8G8R8A8_SRGB, color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR },
            ]),
            required_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            optional_usage: vk::ImageUsageFlags::empty(),
            clipped: true
        };

        match self.main_surface.create_swapchain(&config, size) {
            Ok(swapchain) => {
                self.current_swapchain = Some(swapchain);
                true
            }
            Err(err) => {
                log::info!("Failed to create swapchain of size {:?}: {:?}", size, err);
                self.current_swapchain = None;
                false
            }
        }
    }
}

pub struct B4DVertexFormat {
    pub topology: vk::PrimitiveTopology,
    pub stride: u32,
    pub position: (u32, vk::Format),
    pub color: Option<(u32, vk::Format)>,
    pub uv: Option<(u32, vk::Format)>,
}