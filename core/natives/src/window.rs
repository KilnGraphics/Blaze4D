use std::ffi::{CStr, CString};
use ash::{Entry, Instance, vk};
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use crate::vk::objects::surface::{SurfaceInitError, SurfaceProvider};

pub struct WinitWindow {
    handle: winit::window::Window,
    ash_surface: Option<ash::extensions::khr::Surface>,
    khr_surface: Option<vk::SurfaceKHR>,
}

impl WinitWindow {
    pub fn new<E>(title: &str, width: f64, height: f64, event_loop: &EventLoop<E>) -> Self {
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width, height))
            .build(&event_loop)
            .unwrap();
        window.set_visible(true);

        Self {
            handle: window,
            ash_surface: None,
            khr_surface: None,
        }
    }
}

impl SurfaceProvider for WinitWindow {
    fn get_required_instance_extensions(&self) -> Vec<CString> {
        ash_window::enumerate_required_extensions(&self.handle).unwrap().into_iter().map(|str| {
            CString::from(unsafe { CStr::from_ptr(*str) })
        }).collect()
    }

    unsafe fn init(&mut self, entry: &Entry, instance: &Instance) -> Result<vk::SurfaceKHR, SurfaceInitError> {
        let surface = ash_window::create_surface(entry, instance, &self.handle, None)?;

        self.khr_surface = Some(surface);
        self.ash_surface = Some(ash::extensions::khr::Surface::new(entry, instance));

        Ok(surface)
    }

    unsafe fn destroy(&mut self) {
        if let Some(surface) = self.khr_surface.take() {
            let khr = self.ash_surface.take().unwrap();
            khr.destroy_surface(surface, None);
        }
    }

    unsafe fn get_handle(&self) -> vk::SurfaceKHR {
        self.khr_surface.unwrap()
    }
}