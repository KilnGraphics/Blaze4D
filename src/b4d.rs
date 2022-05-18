use std::collections::VecDeque;
use std::ffi::CString;
use std::sync::{Arc, Mutex, Weak};
use std::time::Instant;

use ash::vk;

use vk_profiles_rs::vp;

use crate::glfw_surface::GLFWSurfaceProvider;
use crate::renderer::emulator::EmulatorRenderer;
use crate::instance::debug_messenger::RustLogDebugMessenger;
use crate::device::init::{create_device, DeviceCreateConfig};
use crate::device::surface::{DeviceSurface, SurfaceSwapchain, SwapchainConfig};
use crate::instance::init::{create_instance, InstanceCreateConfig};
use crate::instance::instance::VulkanVersion;
use crate::vk::objects::surface::{SurfaceProvider};

use crate::prelude::*;

pub struct Blaze4D {
    instance: Arc<InstanceContext>,
    device: DeviceEnvironment,
    emulator: Arc<EmulatorRenderer>,
    main_window: Mutex<MainWindow>,
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

        let main_window = Mutex::new(MainWindow::new(main_surface));

        Self {
            instance,
            device,
            emulator,
            main_window,
        }
    }

    pub fn try_acquire_next_image<T: Fn() -> Option<Vec2u32>>(&self, size_cb: T) -> Option<MainWindowImage> {
        let mut guard = self.main_window.lock().unwrap();

        // We rebuild until we have a not out of date swapchain. The wait timer for rebuilding
        // prevents a massive buildup.
        loop {
            match guard.try_acquire_next_image() {
                None => {
                    guard.try_build_swapchain(self.device.get_device(), size_cb())?;
                }
                Some((image, suboptimal)) => {
                    if suboptimal {
                        guard.try_build_swapchain(self.device.get_device(), size_cb())?;
                    } else {
                        return Some(image);
                    }
                }
            }
        }
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
    fn try_build_swapchain(&mut self, device: &Arc<DeviceContext>, new_size: Option<Vec2u32>) -> Option<()> {
        if let Some(old) = self.current_swapchain.take() {
            self.old_swapchains.push((None, old));
        }

        if let Some(new_size) = new_size {
            let mut elapsed = self.last_rebuild.elapsed();
            while elapsed.as_millis() < 100 {
                // While were waiting we can at least destroy the old swapchains
                self.process_old_swapchains(device);
                elapsed = self.last_rebuild.elapsed();
                if elapsed.as_millis() < 100 {
                    break;
                }

                let diff = std::time::Duration::from_millis(100) - elapsed;
                std::thread::sleep(diff);
                elapsed = self.last_rebuild.elapsed();
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
                MainWindowSwapchain::new(weak.clone(), device.clone(), new_swapchain)
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
    sync_objects: Mutex<VecDeque<SyncObjects>>,
    swapchain_images: Box<[vk::Image]>,
}

impl MainWindowSwapchain {
    fn new(weak: Weak<MainWindowSwapchain>, device: Arc<DeviceContext>, swapchain: Arc<SurfaceSwapchain>) -> Self {
        let swapchain_images: Box<[_]> = swapchain.get_images().iter().map(|(_, image)| *image).collect();

        let sync_objects: VecDeque<_> = (0..3).map(|_| SyncObjects::new(&device)).collect();

        Self {
            weak,
            device,
            swapchain,
            sync_objects: Mutex::new(sync_objects),
            swapchain_images
        }
    }

    fn acquire_next_image(&self) -> Option<(MainWindowImage, bool)> {
        let sync = self.get_next_sync();

        let fences = [sync.acquire_fence, sync.present_fence];
        unsafe {
            self.device.vk().wait_for_fences(&fences, true, u64::MAX)
        }.unwrap();

        unsafe {
            self.device.vk().reset_fences(&fences)
        }.unwrap();

        match self.swapchain.acquire_next_image(u64::MAX, Some(sync.acquire_semaphore), Some(sync.acquire_fence)) {
            Ok((index, suboptimal)) => {
                let arc = Weak::upgrade(&self.weak).unwrap();

                Some((MainWindowImage {
                    swapchain: arc,
                    sync: Some(sync),
                    image_index: index,
                }, suboptimal))
            },
            Err(_) => {
                // We can't reuse these sync objects since they are now unsignaled.
                // Shouldn't be an issue either since we're destroying the swapchain after a failure.
                let mut sync = sync;
                sync.destroy(&self.device);
                None
            }
        }
    }

    fn get_next_sync(&self) -> SyncObjects {
        loop {
            let mut guard = self.sync_objects.lock().unwrap();
            if let Some(sync) = guard.pop_front() {
                return sync;
            }
            drop(guard);

            log::warn!("Out of sync objects! Either too many frames are currently in processing or we dont drop our old frames.");
            std::thread::yield_now();
        }
    }
}

impl Drop for MainWindowSwapchain {
    fn drop(&mut self) {
        let mut guard = self.sync_objects.lock().unwrap();
        while let Some(mut sync) = guard.pop_front() {
            sync.wait_destroy(&self.device);
        }
    }
}

struct SyncObjects {
    acquire_semaphore: vk::Semaphore,
    acquire_fence: vk::Fence,
    present_semaphore: vk::Semaphore,
    present_fence: vk::Fence,
}

impl SyncObjects {
    fn new(device: &DeviceContext) -> Self {
        let fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);

        let semaphore_info = vk::SemaphoreCreateInfo::builder();

        unsafe {
            Self {
                acquire_semaphore: device.vk().create_semaphore(&semaphore_info, None).unwrap(),
                acquire_fence: device.vk().create_fence(&fence_info, None).unwrap(),
                present_semaphore: device.vk().create_semaphore(&semaphore_info, None).unwrap(),
                present_fence: device.vk().create_fence(&fence_info, None).unwrap(),
            }
        }
    }

    fn wait_destroy(&mut self, device: &DeviceContext) {
        let fences = [self.acquire_fence, self.present_fence];
        unsafe {
            device.vk().wait_for_fences(&fences, true, u64::MAX)
        }.unwrap();

        self.destroy(device);
    }

    fn destroy(&mut self, device: &DeviceContext) {
        unsafe {
            device.vk().destroy_semaphore(self.acquire_semaphore, None);
            device.vk().destroy_fence(self.acquire_fence, None);
            device.vk().destroy_semaphore(self.present_semaphore, None);
            device.vk().destroy_fence(self.present_fence, None);
        }
    }
}

pub struct MainWindowImage {
    swapchain: Arc<MainWindowSwapchain>,
    sync: Option<SyncObjects>,
    image_index: u32,
}

impl Drop for MainWindowImage {
    fn drop(&mut self) {
        if let Some(sync) = self.sync.take() {
            // Return the sync objects
            self.swapchain.sync_objects.lock().unwrap().push_back(sync);
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