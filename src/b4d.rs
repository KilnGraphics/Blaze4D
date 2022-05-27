use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ash::vk;

use vk_profiles_rs::vp;

use crate::glfw_surface::GLFWSurfaceProvider;
use crate::instance::debug_messenger::RustLogDebugMessenger;
use crate::device::init::{create_device, DeviceCreateConfig};
use crate::device::surface::{DeviceSurface, SurfaceSwapchain, SwapchainConfig};
use crate::instance::init::{create_instance, InstanceCreateConfig};
use crate::instance::instance::VulkanVersion;
use crate::vk::objects::surface::SurfaceProvider;

use crate::prelude::*;
use crate::renderer::emulator::debug_pipeline::{DepthPipelineConfig, DepthPipelineCore, DepthTypeInfo};
use crate::renderer::emulator::{EmulatorRenderer, MeshData, VertexFormatSetBuilder};
use crate::renderer::emulator::pass::PassRecorder;
use crate::renderer::emulator::pipeline::{EmulatorPipeline, SwapchainOutput};

pub struct Blaze4D {
    instance: Arc<InstanceContext>,
    device: DeviceEnvironment,
    emulator: Arc<EmulatorRenderer>,

    render_config: Mutex<RenderConfig>,
}

impl Blaze4D {
    pub fn new(main_window: Box<dyn SurfaceProvider>, vertex_formats: VertexFormatSetBuilder) -> Self {
        crate::debug::text::ldfnt();

        let mut instance_config = InstanceCreateConfig::new(
            vp::KhrRoadmap2022::profile_properties(),
            VulkanVersion::VK_1_3,
            CString::new("Minecraft").unwrap(),
            vk::make_api_version(0, 0, 1, 0)
        );
        // instance_config.enable_validation();
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

        let render_config = Mutex::new(RenderConfig::new(device.clone(), main_surface));

        let emulator = EmulatorRenderer::new(device.clone(), vertex_formats);

        Self {
            instance,
            device,
            emulator,

            render_config,
        }
    }

    pub fn set_emulator_vertex_formats(&self, formats: Box<[B4DVertexFormat]>) {
        self.render_config.lock().unwrap().set_vertex_formats(formats);
    }

    pub fn try_start_frame(&self, window_size: Vec2u32) -> Option<PassRecorder> {
        if let Some(mut recorder) = self.render_config.lock().unwrap().try_start_frame(&self.emulator, window_size) {

            recorder.set_model_view_matrix(Mat4f32::identity());
            recorder.set_projection_matrix(Mat4f32::identity());

            Some(recorder)
        } else {
            None
        }
    }
}

struct RenderConfig {
    device: DeviceEnvironment,
    main_surface: Arc<DeviceSurface>,

    vertex_formats: Box<[B4DVertexFormat]>,

    last_rebuild: Instant,
    current_swapchain: Option<Arc<SurfaceSwapchain>>,
    current_pipeline: Option<(Arc<dyn EmulatorPipeline>, Arc<SwapchainOutput>)>,
}

impl RenderConfig {
    fn new(device: DeviceEnvironment, main_surface: Arc<DeviceSurface>) -> Self {
        Self {
            device,
            main_surface,

            vertex_formats: Box::new([]),

            last_rebuild: Instant::now() - Duration::from_secs(100),
            current_swapchain: None,
            current_pipeline: None,
        }
    }

    fn set_vertex_formats(&mut self, formats: Box<[B4DVertexFormat]>) {
        self.vertex_formats = formats;
        self.current_pipeline = None;
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

            let depth_infos: Box<_> = self.vertex_formats.iter().map(|format| {
                DepthTypeInfo {
                    vertex_stride: format.stride,
                    vertex_position_offset: format.position.0,
                    vertex_position_format: format.position.1,
                    topology: format.topology,
                    primitive_restart: false,
                    discard: false
                }
            }).collect();

            let pipeline = Arc::new(DepthPipelineCore::new(self.device.clone(), depth_infos.as_ref()));
            let pipeline = DepthPipelineConfig::new(pipeline, size) as Arc<dyn EmulatorPipeline>;
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
            formats: Box::new([vk::SurfaceFormatKHR{ format: vk::Format::R8G8B8A8_SRGB, color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR }]),
            required_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            optional_usage: vk::ImageUsageFlags::empty(),
            clipped: true
        };

        match self.main_surface.create_swapchain(&config, size) {
            Ok(swapchain) => {
                self.current_swapchain = Some(swapchain);
                true
            }
            Err(_) => {
                log::info!("Failed to create swapchain of size {:?}", size);
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


#[repr(C)]
#[derive(Copy, Clone)]
struct TestVertex {
    position: Vec3f32,
}

impl TestVertex {
    fn get_format() -> B4DVertexFormat {
        B4DVertexFormat {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            stride: std::mem::size_of::<Self>() as u32,
            position: (0, vk::Format::R32G32B32_SFLOAT),
            color: None,
            uv: None
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn b4d_init(surface: *mut GLFWSurfaceProvider) -> *mut Blaze4D {
    env_logger::init();

    let surface: Box<dyn SurfaceProvider> = Box::from_raw(surface);

    //let b4d = Box::leak(Box::new(Blaze4D::new(surface)));

    //b4d
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn b4d_destroy(b4d: *mut Blaze4D) {
    Box::from_raw(b4d);
}