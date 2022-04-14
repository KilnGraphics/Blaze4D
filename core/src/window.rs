use ash::vk::SurfaceKHR;
use ash::vk;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use crate::objects::surface::SurfaceProvider;

use crate::rosella::InstanceContextImpl;

pub struct RosellaWindow {
    pub event_loop: EventLoop<()>,
    pub handle: winit::window::Window,
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
