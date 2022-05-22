use std::collections::VecDeque;
use std::ffi::CString;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;
use ash::prelude::VkResult;

use ash::vk;
use nalgebra::clamp;

use vk_profiles_rs::vp;
use crate::device::device::VkQueue;

use crate::glfw_surface::GLFWSurfaceProvider;
use crate::renderer::emulator::{EmulatorRenderer, OutputConfiguration, RenderConfiguration, RenderPath, TestVertex};
use crate::instance::debug_messenger::RustLogDebugMessenger;
use crate::device::init::{create_device, DeviceCreateConfig};
use crate::device::surface::{DeviceSurface, SurfaceSwapchain, SwapchainConfig};
use crate::instance::init::{create_instance, InstanceCreateConfig};
use crate::instance::instance::VulkanVersion;
use crate::objects::{ObjectSet, ObjectSetProvider};
use crate::vk::objects::surface::{SurfaceProvider};

use crate::prelude::*;
use crate::renderer::emulator::pass::{ObjectData, PassRecorder, PassEventListener};

pub struct Blaze4D {
    instance: Arc<InstanceContext>,
    device: DeviceEnvironment,
    emulator: Arc<EmulatorRenderer>,
    emulator_path: Arc<RenderPath>,
    main_window: Mutex<MainWindow>,

    tmp_queue: VkQueue,
    tmp_pool: vk::CommandPool,
    tmp_buffers: Box<[vk::CommandBuffer]>,
    tmp_next_buffer: AtomicUsize,
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

        let (device, surfaces) = create_device(device_config, instance.clone()).unwrap();
        let main_surface = surfaces.into_iter().fold(None, |res, (id, surface)| {
            if id == main_surface {
                Some(surface)
            } else {
                res
            }
        }).unwrap();

        let emulator = EmulatorRenderer::new(device.clone());
        let emulator_path = emulator.crate_test_render_path();

        let main_window = Mutex::new(MainWindow::new(main_surface));



        let queue = device.get_device().get_main_queue();
        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue.get_queue_family_index());

        let pool = unsafe {
            device.vk().create_command_pool(&info, None)
        }.unwrap();

        let info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(32);

        let buffers = unsafe {
            device.vk().allocate_command_buffers(&info)
        }.unwrap();


        Self {
            instance,
            device,
            emulator,
            emulator_path,
            main_window,

            tmp_queue: queue,
            tmp_pool: pool,
            tmp_buffers: buffers.into_boxed_slice(),
            tmp_next_buffer: AtomicUsize::new(0),
        }
    }

    pub fn try_acquire_next_image<T: Fn() -> Option<Vec2u32>>(&self, size_cb: T) -> Option<MainWindowImage> {
        let mut guard = self.main_window.lock().unwrap();
        guard.process_old_swapchains(self.device.get_device());

        // We try to rebuild once otherwise we just return. We need to do this since some window
        // systems require polling from the application so we cant just lock here forever.
        match guard.try_acquire_next_image() {
            None => {
                guard.try_build_swapchain(self.device.get_device(), &self.emulator_path, size_cb())?;
            }
            Some((image, suboptimal)) => {
                if suboptimal {
                    guard.try_build_swapchain(self.device.get_device(), &self.emulator_path, size_cb())?;
                } else {
                    return Some(image);
                }
            }
        }

        Some(guard.try_acquire_next_image()?.0)
    }

    pub fn try_start_frame<T: Fn() -> Option<Vec2u32>>(&self, size_cb: T) -> Option<PassRecorder> {
        let image = self.try_acquire_next_image(size_cb)?;
        let mut frame = self.emulator.start_frame(image.swapchain.emulator_render_config.clone());
        let queue = self.device.get_device().get_main_queue();

        frame.add_output(image.swapchain.emulator_output_config.clone(), image.image_index as usize, image.acquire_semaphore, None);
        frame.add_signal_op(image.present_semaphore, None);
        frame.add_signal_op(image.ready_semaphore, Some(image.ready_value));
        frame.add_event_listener(Box::new(image));

        let data = [
            TestVertex {
                position: Vec3f32::new(-1.0, -1.0, 0.05),
                uv: Vec2f32::new(0.0, 0.0),
            },
            TestVertex {
                position: Vec3f32::new(-1.0, 1.0, 0.05),
                uv: Vec2f32::new(1.0, 0.0),
            },
            TestVertex {
                position: Vec3f32::new(0.0, 1.0, 0.05),
                uv: Vec2f32::new(0.0, 1.0),
            }
        ];

        let index = [0u32, 1u32, 2u32];

        let d = ObjectData {
            vertex_data: crate::util::slice::to_byte_slice(&data),
            index_data: crate::util::slice::to_byte_slice(&index),
            draw_count: 3
        };

        frame.record_object(&d);

        Some(frame)
    }

    pub fn tmp_present(&self, image: MainWindowImage) {
        let mut index;
        loop {
            index = self.tmp_next_buffer.load(Ordering::SeqCst);
            let next_index = (index + 1) % self.tmp_buffers.len();
            if self.tmp_next_buffer.compare_exchange(index, next_index, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                break;
            }
        }

        let cmd = self.tmp_buffers[index];

        unsafe {
            self.device.vk().reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
        }.unwrap();

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device.vk().begin_command_buffer(cmd, &info)
        }.unwrap();

        let image_barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .image(image.image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            });

        unsafe {
            self.device.vk().cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&image_barrier)
            );
        }

        unsafe {
            self.device.vk().end_command_buffer(cmd)
        }.unwrap();

        let stage_mask = vk::PipelineStageFlags::ALL_COMMANDS;

        let signal_values = [0u64, image.ready_value];
        let mut timeline_info = vk::TimelineSemaphoreSubmitInfo::builder()
            .signal_semaphore_values(&signal_values);

        let signal_semaphores = [image.present_semaphore, image.ready_semaphore];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(std::slice::from_ref(&image.acquire_semaphore))
            .wait_dst_stage_mask(std::slice::from_ref(&stage_mask))
            .command_buffers(std::slice::from_ref(&cmd))
            .signal_semaphores(&signal_semaphores)
            .push_next(&mut timeline_info);

        unsafe {
            self.tmp_queue.submit(std::slice::from_ref(&submit_info), None)
        }.unwrap();

        let swapchain_guard = image.swapchain.swapchain.get_swapchain().lock().unwrap();
        let swapchain = *swapchain_guard;
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(std::slice::from_ref(&image.present_semaphore))
            .swapchains(std::slice::from_ref(&swapchain))
            .image_indices(std::slice::from_ref(&image.image_index));

        unsafe {
            self.tmp_queue.present(&present_info)
        };
        drop(swapchain_guard);
    }
}

struct MainWindow {
    surface: Arc<DeviceSurface>,
    current_swapchain: Option<Arc<MainWindowSwapchain>>,
    last_rebuild: Instant,
    old_swapchains: Vec<(Option<Instant>, Arc<MainWindowSwapchain>)>,
}

impl MainWindow {
    fn new(surface: Arc<DeviceSurface>) -> Self {
        Self {
            surface,
            current_swapchain: None,
            last_rebuild: Instant::now() - std::time::Duration::from_secs(60),
            old_swapchains: Vec::with_capacity(20)
        }
    }

    fn try_acquire_next_image(&self) -> Option<(MainWindowImage, bool)> {
        if let Some(swapchain) = &self.current_swapchain {
            swapchain.acquire_next_image()
        } else {
            None
        }
    }

    /// Attempts to build a new swapchain.
    ///
    /// If the new_size parameter is [`None`] no new swapchain will be created but any old one will
    /// be retired.
    ///
    /// Returns [`Some`] if a new swapchain has been created or [`None`] if no current swapchain
    /// exists.
    fn try_build_swapchain(&mut self, device: &Arc<DeviceContext>, render_path: &Arc<RenderPath>, new_size: Option<Vec2u32>) -> Option<()> {
        log::error!("Rebuilding swapchain");
        if let Some(old) = self.current_swapchain.take() {
            self.old_swapchains.push((None, old));
        }

        if let Some(new_size) = new_size {
            if self.last_rebuild.elapsed().as_millis() < 100 {
                // While were waiting we can at least destroy any old swapchains
                self.process_old_swapchains(device);

                let elapsed = self.last_rebuild.elapsed();
                if elapsed.as_millis() < 100 {
                    let diff = std::time::Duration::from_millis(100) - elapsed;
                    std::thread::sleep(diff);
                }
            }

            let config = SwapchainConfig {
                formats: Box::new([vk::SurfaceFormatKHR {
                    format: vk::Format::R8G8B8A8_SRGB,
                    color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR
                }]),
                required_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
                optional_usage: vk::ImageUsageFlags::empty(),
                clipped: true,
            };

            let new_swapchain = self.surface.create_swapchain(&config, new_size).ok()?;
            let new_swapchain = Arc::new_cyclic(|weak| {
                MainWindowSwapchain::new(weak.clone(), device.clone(), new_swapchain, render_path)
            });

            self.current_swapchain = Some(new_swapchain);
            self.last_rebuild = Instant::now();

            Some(())
        } else {
            None
        }
    }

    /// One of the issues with vulkan swapchains is that its impossible to safely destroy them since
    /// its impossible to tell when a present operation has completed. As a workaround any old
    /// swapchain is pushed into the old_swapchains Vec. Once the arc only has one reference we wait
    /// for the device to be idle and then store the instant after the wait. After some time has
    /// elapsed the swpachain is finally destroyed. The timer is necessary since device wait idle
    /// does not guarantee that present operations have completed.
    ///
    /// Since a swapchain rebuild should be very rare doing this will not have any performance
    /// impact.
    fn process_old_swapchains(&mut self, device: &DeviceContext) {
        if !self.old_swapchains.is_empty() {
            log::error!("Old swapchains: {:?}", self.old_swapchains.len());
            let wait = self.old_swapchains.iter().fold(false, |wait, (time, old)| {
                wait || (time.is_none() && Arc::strong_count(old) == 1)
            });
            if wait {
                unsafe { device.vk().device_wait_idle() };
            }
            let now = Instant::now();
            for (time, old) in &mut self.old_swapchains {
                if time.is_none() && Arc::strong_count(old) == 1 {
                    *time = Some(now)
                }
            }
            self.old_swapchains.retain(|(time, _)| {
                if let Some(time) = time {
                    // We destroy after 200ms
                    time.elapsed().as_millis() < 200
                } else {
                    true
                }
            })
        }
    }
}

struct MainWindowSwapchain {
    weak: Weak<MainWindowSwapchain>,
    device: Arc<DeviceContext>,
    swapchain: Arc<SurfaceSwapchain>,
    sync_next_index: AtomicUsize,
    sync_objects: Box<[SyncObjects]>,
    swapchain_images: Box<[ImageObjects]>,
    emulator_render_config: Arc<RenderConfiguration>,
    emulator_output_config: Arc<OutputConfiguration>,
}

impl MainWindowSwapchain {
    fn new(weak: Weak<MainWindowSwapchain>, device: Arc<DeviceContext>, swapchain: Arc<SurfaceSwapchain>, render_path: &Arc<RenderPath>) -> Self {
        let swapchain_images: Box<_> = swapchain.get_images().iter().map(|(_, image)| {
            ImageObjects::new(&device, *image)
        }).collect();

        let sync_objects: Box<_> = (0..4).map(|_| SyncObjects::new(&device)).collect();

        let ids: Box<_> = swapchain.get_images().iter().map(|(id, _)| *id).collect();

        let emulator_render_config = Arc::new(RenderConfiguration::new(render_path.clone(), swapchain.get_image_size(), 2));
        let emulator_output_config = Arc::new(OutputConfiguration::new(
            emulator_render_config.clone(),
            swapchain.get_image_size(),
            ids.as_ref(),
            ObjectSet::new(swapchain.clone()),
            swapchain.get_image_format().format,
            vk::ImageLayout::PRESENT_SRC_KHR
        ));

        Self {
            weak,
            device,
            swapchain,
            sync_next_index: AtomicUsize::new(0),
            sync_objects,
            swapchain_images,
            emulator_render_config,
            emulator_output_config
        }
    }

    fn acquire_next_image(&self) -> Option<(MainWindowImage, bool)> {
        let sync = &self.sync_objects[self.get_next_sync()];

        let (ready_semaphore, ready_value) = sync.wait_and_get(&self.device);

        let present_queue = self.device.get_main_queue();

        match self.swapchain.acquire_next_image(u64::MAX, Some(sync.acquire_semaphore), None) {
            Ok((index, suboptimal)) => {
                let image_objects = &self.swapchain_images[index as usize];
                Some((MainWindowImage {
                    present_queue,
                    swapchain: self.weak.upgrade().unwrap(),
                    ready_semaphore,
                    ready_value,
                    acquire_semaphore: sync.acquire_semaphore,
                    present_semaphore: image_objects.present_semaphore,
                    image: image_objects.image,
                    image_index: index,
                }, suboptimal))
            },
            Err(_) => {
                None
            }
        }
    }

    fn get_next_sync(&self) -> usize {
        loop {
            let index = self.sync_next_index.load(Ordering::SeqCst);
            let next_index = (index + 1) % self.sync_objects.len();

            if self.sync_next_index.compare_exchange(index, next_index, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                return index;
            }
        }
    }
}

impl Drop for MainWindowSwapchain {
    fn drop(&mut self) {
        for sync_object in self.sync_objects.as_mut() {
            // We should not wait here since its possible that fences are unsignaled and have no op pending on them
            sync_object.destroy(&self.device);
        }
        for image_object in self.swapchain_images.as_mut() {
            image_object.destroy(&self.device);
        }
    }
}

struct SyncObjects {
    acquire_semaphore: vk::Semaphore,
    ready_semaphore: vk::Semaphore,
    ready_semaphore_value: AtomicU64,
}

impl SyncObjects {
    fn new(device: &DeviceContext) -> Self {
        let binary_info = vk::SemaphoreCreateInfo::builder();

        let mut timeline = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(0);
        let timeline_info = vk::SemaphoreCreateInfo::builder()
            .push_next(&mut timeline);

        unsafe {
            Self {
                acquire_semaphore: device.vk().create_semaphore(&binary_info, None).unwrap(),
                ready_semaphore: device.vk().create_semaphore(&timeline_info, None).unwrap(),
                ready_semaphore_value: AtomicU64::new(0),
            }
        }
    }

    fn wait_and_get(&self, device: &DeviceContext) -> (vk::Semaphore, u64) {
        let old = self.ready_semaphore_value.fetch_add(1, Ordering::SeqCst);
        let wait_info = vk::SemaphoreWaitInfo::builder()
            .semaphores(std::slice::from_ref(&self.ready_semaphore))
            .values(std::slice::from_ref(&old));
        unsafe {
            device.vk().wait_semaphores(&wait_info, u64::MAX)
        }.unwrap();

        (self.ready_semaphore, old + 1)
    }

    fn destroy(&mut self, device: &DeviceContext) {
        unsafe {
            device.vk().destroy_semaphore(self.acquire_semaphore, None);
            device.vk().destroy_semaphore(self.ready_semaphore, None);
        }
    }
}

struct ImageObjects {
    image: vk::Image,
    present_semaphore: vk::Semaphore,
}

impl ImageObjects {
    fn new(device: &DeviceContext, image: vk::Image) -> Self {
        let semaphore_info = vk::SemaphoreCreateInfo::builder();

        let present_semaphore = unsafe {
            device.vk().create_semaphore(&semaphore_info, None)
        }.unwrap();

        Self {
            image,
            present_semaphore,
        }
    }

    fn destroy(&self, device: &DeviceContext) {
        unsafe {
            // The image is owned by the swapchain
            device.vk().destroy_semaphore(self.present_semaphore, None);
        }
    }
}

pub struct MainWindowImage {
    present_queue: VkQueue,
    swapchain: Arc<MainWindowSwapchain>,
    ready_semaphore: vk::Semaphore,
    ready_value: u64,
    acquire_semaphore: vk::Semaphore,
    present_semaphore: vk::Semaphore,
    image: vk::Image,
    image_index: u32,
}

impl PassEventListener for MainWindowImage {
    fn on_post_submit(&self) {
        let guard = self.swapchain.swapchain.get_swapchain().lock().unwrap();
        let swapchain = *guard;
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(std::slice::from_ref(&self.present_semaphore))
            .swapchains(std::slice::from_ref(&swapchain))
            .image_indices(std::slice::from_ref(&self.image_index));

        match unsafe {
            self.present_queue.present(&present_info)
        } {
            Ok(_) => {}
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
            }
            Err(_) => {
                panic!()
            }
        }
    }

    fn on_execution_completed(&self) {
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