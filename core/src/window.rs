use ash::vk::SurfaceKHR;
use ash::vk;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use crate::objects::surface::SurfaceProvider;

use crate::rosella::InstanceContext;

/// Represents a ash surface and a KHR surface
pub struct RosellaSurface {
    instance: InstanceContext,
    surface: vk::SurfaceKHR,
}

pub struct RosellaWindow {
    pub event_loop: EventLoop<()>,
    pub handle: winit::window::Window,
}

impl RosellaSurface {
    pub fn new(instance: &InstanceContext, window: &RosellaWindow) -> Self {
        let surface = unsafe {
            ash_window::create_surface(instance.get_entry(), instance.vk(), &window.handle, None)
        }.unwrap();

        RosellaSurface {
            instance: instance.clone(),
            surface,
        }
    }
}

impl SurfaceProvider for RosellaSurface {
    fn get_handle(&self) -> SurfaceKHR {
        self.surface
    }
}

impl Drop for RosellaSurface {
    fn drop(&mut self) {
        unsafe { self.instance.get_extension::<ash::extensions::khr::Surface>().unwrap().destroy_surface(self.surface, None) }
    }
}

impl RosellaWindow {
    pub fn new(title: &str, width: f64, height: f64) -> RosellaWindow {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width, height))
            .build(&event_loop)
            .unwrap();

        RosellaWindow {
            event_loop,
            handle: window,
        }
    }
}
