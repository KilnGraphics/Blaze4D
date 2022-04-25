extern crate b4d_core;

use std::io::{BufRead, stdin};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use b4d_core::window::WinitWindow;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = Box::new(WinitWindow::new("TEEESSTETE", 800.0, 600.0, &event_loop));

    let b4d = b4d_core::b4d::Blaze4D::new(window);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit
            },
            Event::MainEventsCleared => {
            }
            _ => {
            }
        }
    });
}