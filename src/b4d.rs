use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ash::vk;

use crate::instance::debug_messenger::RustLogDebugMessenger;
use crate::device::init::{create_device, DeviceCreateConfig};
use crate::device::surface::{DeviceSurface, SurfaceSwapchain, SwapchainConfig};
use crate::instance::init::{create_instance, InstanceCreateConfig};
use crate::vk::objects::surface::SurfaceProvider;

use crate::prelude::*;
use crate::renderer::emulator::{EmulatorRenderer, GlobalMesh, MeshData};
use crate::renderer::emulator::debug_pipeline::{DebugPipeline, DebugPipelineMode};
use crate::renderer::emulator::mc_shaders::{McUniform, ShaderId, VertexFormat};
use crate::renderer::emulator::PassRecorder;
use crate::renderer::emulator::pipeline::{EmulatorPipeline, SwapchainOutput};

pub struct Blaze4D {
    instance: Arc<InstanceContext>,
    device: Arc<DeviceContext>,
    emulator: Arc<EmulatorRenderer>,

    render_config: Mutex<RenderConfig>,
}

impl Blaze4D {
    /// Creates a new Blaze4D instance and starts all engine modules.
    ///
    /// The supported vertex formats for the [`EmulatorRenderer`] must be provided here.
    pub fn new(mut main_window: Box<dyn SurfaceProvider>, enable_validation: bool) -> Self {
        let mut instance_config = InstanceCreateConfig::new(
            CString::new("Minecraft").unwrap(),
            vk::make_api_version(0, 0, 1, 0)
        );
        if enable_validation {
            instance_config.enable_validation();
        }
        instance_config.add_debug_messenger(Box::new(RustLogDebugMessenger::new()));
        for ext in main_window.get_required_instance_extensions() {
            instance_config.add_required_extension(&ext);
        }

        let instance = create_instance(instance_config).unwrap();

        let window_surface = main_window.init(instance.get_entry(), instance.vk()).unwrap();

        let mut device_config = DeviceCreateConfig::new();
        device_config.require_swapchain();
        device_config.add_surface(window_surface);
        device_config.disable_robustness();

        let device = create_device(device_config, instance.clone()).unwrap_or_else(|err| {
            log::error!("Failed to create device in Blaze4D::new(): {:?}", err);
            panic!()
        });
        let main_surface = DeviceSurface::new(device.get_functions().clone(), main_window);

        let emulator = Arc::new(EmulatorRenderer::new(device.clone()));

        let render_config = Mutex::new(RenderConfig::new(device.clone(), emulator.clone(), main_surface));

        Self {
            instance,
            device,
            emulator,

            render_config,
        }
    }

    /// Configures the current debug mode. Any frame started after calling this function will use
    /// the specified debug mode until another call to this function is made.
    ///
    /// If [`None`] is passed the debug mode is disabled.
    pub fn set_debug_mode(&self, mode: Option<DebugPipelineMode>) {
        self.render_config.lock().unwrap().set_debug_mode(mode);
    }

    pub fn create_global_mesh(&self, data: &MeshData) -> Arc<GlobalMesh> {
        self.emulator.create_global_mesh(data)
    }

    pub fn create_shader(&self, vertex_format: &VertexFormat, used_uniforms: McUniform) -> ShaderId {
        self.emulator.create_shader(vertex_format, used_uniforms)
    }

    pub fn drop_shader(&self, id: ShaderId) {
        self.emulator.drop_shader(id);
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
    device: Arc<DeviceContext>,
    emulator: Arc<EmulatorRenderer>,
    main_surface: Arc<DeviceSurface>,

    last_rebuild: Instant,
    current_swapchain: Option<Arc<SurfaceSwapchain>>,
    current_pipeline: Option<(Arc<dyn EmulatorPipeline>, Arc<SwapchainOutput>)>,

    debug_mode: Option<DebugPipelineMode>,
    debug_pipeline: Option<(Arc<dyn EmulatorPipeline>, Arc<SwapchainOutput>)>,
}

impl RenderConfig {
    fn new(device: Arc<DeviceContext>, emulator: Arc<EmulatorRenderer>, main_surface: Arc<DeviceSurface>) -> Self {
        Self {
            device,
            emulator,
            main_surface,

            last_rebuild: Instant::now() - Duration::from_secs(100),
            current_swapchain: None,
            current_pipeline: None,

            debug_mode: Some(DebugPipelineMode::Color),
            debug_pipeline: None
        }
    }

    fn set_debug_mode(&mut self, mode: Option<DebugPipelineMode>) {
        if self.debug_mode != mode {
            self.debug_mode = mode;
            self.debug_pipeline = None;
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
            self.debug_pipeline = None;
        }

        let (pipeline, output) = self.prepare_pipeline(size);

        let (output, suboptimal) = match output.next_image() {
            None => {
                self.current_pipeline = None;
                self.debug_pipeline = None;
                self.current_swapchain = None;
                return None;
            }
            Some(result) => result,
        };

        let mut recorder = renderer.start_pass(pipeline.clone());
        recorder.use_output(output);

        if suboptimal {
            self.current_pipeline = None;
            self.debug_pipeline = None;
            self.current_swapchain = None;
        }

        Some(recorder)
    }

    fn prepare_pipeline(&mut self, output_size: Vec2u32) -> (Arc<dyn EmulatorPipeline>, &Arc<SwapchainOutput>) {
        if let Some(debug_mode) = &self.debug_mode {
            if self.debug_pipeline.is_none() {
                log::info!("No debug pipeline present. Rebuilding for size {:?}", output_size);

                let pipeline = DebugPipeline::new(self.emulator.clone(), *debug_mode, output_size).unwrap();
                let swapchain_output = SwapchainOutput::new(&self.device, pipeline.clone(), self.current_swapchain.as_ref().cloned().unwrap());

                self.debug_pipeline = Some((pipeline, swapchain_output));
            }

            let (pipeline, output) = self.debug_pipeline.as_ref().unwrap();
            (pipeline.clone(), output)
        } else {
            todo!()
        }
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