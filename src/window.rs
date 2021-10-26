use ash::extensions::khr::Surface;
use ash::vk::SurfaceKHR;
use ash::{Entry, Instance};
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use crate::ALLOCATION_CALLBACKS;

/// Represents a ash surface and a KHR surface
pub struct RosellaSurface {
    pub ash_surface: Surface,
    pub khr_surface: SurfaceKHR,
}

pub struct RosellaWindow {
    pub event_loop: EventLoop<()>,
    pub handle: winit::window::Window,
}

impl RosellaSurface {
    pub fn new(instance: &Instance, vk: &Entry, window: &RosellaWindow) -> Self {
        RosellaSurface {
            ash_surface: Surface::new(vk, instance),
            khr_surface: unsafe { ash_window::create_surface(vk, instance, &window.handle, ALLOCATION_CALLBACKS) }
                .expect("Failed to create window surface."),
        }
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
