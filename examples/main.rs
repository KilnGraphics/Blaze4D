extern crate b4d_core;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use b4d_core::prelude::Vec2u32;

use b4d_core::window::WinitWindow;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = Box::new(WinitWindow::new("TEEESSTETE", 800.0, 600.0, &event_loop));

    let b4d = b4d_core::b4d::Blaze4D::new(window);

    let mut draw_times = Vec::with_capacity(1000);
    let mut last_update = std::time::Instant::now();

    let mut current_size = Vec2u32::new(800, 600);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                current_size[0] = new_size.width;
                current_size[1] = new_size.height;
            }
            Event::MainEventsCleared => {
                let now = std::time::Instant::now();
                /*if let Some(image) = b4d.try_acquire_next_image(|| Some(current_size)) {
                    b4d.tmp_present(image);
                }*/
                b4d.try_start_frame(current_size);
                draw_times.push(now.elapsed());

                if last_update.elapsed().as_secs() >= 2 {
                    let sum = draw_times.iter().fold(0f64, |sum, time| sum + time.as_secs_f64());
                    let avg = sum / (draw_times.len() as f64);
                    let fps = 1f64 / avg;
                    draw_times.clear();

                    log::error!("Average frame time over last 2 seconds: {:?} ({:?})", avg, fps);

                    last_update = std::time::Instant::now();
                }
            }
            _ => {
            }
        }
    });
}