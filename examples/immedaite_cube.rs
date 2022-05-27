extern crate b4d_core;

use ash::vk;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use b4d_core::b4d::B4DVertexFormat;

use b4d_core::prelude::*;
use b4d_core::renderer::emulator::{MeshData, VertexFormatInfo, VertexFormatSetBuilder};

use b4d_core::window::WinitWindow;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = Box::new(WinitWindow::new("ImmediateCube", 800.0, 600.0, &event_loop));

    let mut format_set = VertexFormatSetBuilder::new();
    let format_id = format_set.add_format(VertexFormatInfo {
        stride: Vertex::make_b4d_vertex_format().stride as usize,
    });
    let b4d = b4d_core::b4d::Blaze4D::new(window, format_set);
    b4d.set_emulator_vertex_formats(Box::new([Vertex::make_b4d_vertex_format()]));

    let mut draw_times = Vec::with_capacity(1000);
    let mut last_update = std::time::Instant::now();

    let mut current_size = Vec2u32::new(800, 600);


    let data = MeshData {
        vertex_data: b4d_core::util::slice::to_byte_slice(&CUBE_VERTICES),
        index_data: b4d_core::util::slice::to_byte_slice(&CUBE_INDICES),
        index_count: CUBE_INDICES.len() as u32,
        index_type: vk::IndexType::UINT32,
        vertex_format_id: format_id,
    };

    let mesh_id = b4d.create_static_mesh(&data);


    let start = std::time::Instant::now();

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
                if let Some(mut recorder) = b4d.try_start_frame(current_size) {

                    recorder.set_projection_matrix(make_projection_matrix(current_size, 90f32));

                    let elapsed = start.elapsed().as_secs_f32();
                    let rotation = Mat4f32::new_rotation(Vec3f32::new(elapsed / 2.34f32, elapsed / 2.783f32, elapsed / 2.593f32));
                    let translation = Mat4f32::new_translation(&Vec3f32::new(0f32, 0f32, 5f32));
                    recorder.set_model_view_matrix(translation * rotation);

                    let data = MeshData {
                        vertex_data: b4d_core::util::slice::to_byte_slice(&CUBE_VERTICES),
                        index_data: b4d_core::util::slice::to_byte_slice(&CUBE_INDICES),
                        index_count: CUBE_INDICES.len() as u32,
                        index_type: vk::IndexType::UINT32,
                        vertex_format_id: format_id,
                    };

                    let translation = Mat4f32::new_translation(&Vec3f32::new(
                        0f32,
                        0f32,
                        5f32
                    ));
                    recorder.set_model_view_matrix(translation * rotation);
                    // recorder.draw_static(mesh_id, 0);


                    for x in -10i32..=10i32 {
                        for y in -10i32..=10i32 {
                            for z in -10i32..=10i32 {
                                let translation = Mat4f32::new_translation(&Vec3f32::new(
                                    0f32 + ((x as f32) / 5f32),
                                    0f32 + ((y as f32) / 5f32),
                                    5f32 + ((z as f32) / 5f32)
                                ));
                                recorder.set_model_view_matrix(translation * rotation);
                                // recorder.draw_static(mesh_id, 0);
                                recorder.draw_immediate(&data, 0);
                            }
                        }
                    }

                    drop(recorder);
                }
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

const CUBE_VERTICES: [Vertex; 8] = [
    Vertex {
        position: Vec3f32::new(-1f32, -1f32, -1f32),
    },
    Vertex {
        position: Vec3f32::new(1f32, -1f32, -1f32),
    },
    Vertex {
        position: Vec3f32::new(-1f32, 1f32, -1f32),
    },
    Vertex {
        position: Vec3f32::new(1f32, 1f32, -1f32),
    },
    Vertex {
        position: Vec3f32::new(-1f32, -1f32, 1f32),
    },
    Vertex {
        position: Vec3f32::new(1f32, -1f32, 1f32),
    },
    Vertex {
        position: Vec3f32::new(-1f32, 1f32, 1f32),
    },
    Vertex {
        position: Vec3f32::new(1f32, 1f32, 1f32),
    },
];

const CUBE_INDICES: [u32; 36] = [
    4, 6, 7, 7, 5, 4, // Front
    3, 2, 0, 0, 1, 3, // Back
    6, 2, 3, 3, 7, 6, // Top
    0, 4, 5, 5, 1, 0, // Bottom
    0, 2, 6, 6, 4, 0, // Left
    5, 7, 3, 3, 1, 5, // Right
];

#[derive(Copy, Clone)]
struct Vertex {
    position: Vec3f32,
}

impl Vertex {
    fn make_b4d_vertex_format() -> B4DVertexFormat {
        B4DVertexFormat {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            stride: std::mem::size_of::<Vertex>() as u32,
            position: (0, vk::Format::R32G32B32_SFLOAT),
            color: None,
            uv: None
        }
    }
}

fn make_projection_matrix(window_size: Vec2u32, fov: f32) -> Mat4f32 {
    let t = (fov / 2f32).tan();
    let a1 = (window_size[1] as f32) / (window_size[0] as f32);

    let f = 5f32;
    let n = 1f32;

    Mat4f32::new(
        a1 / t, 0f32, 0f32, 0f32,
        0f32, 1f32 / t, 0f32, 0f32,
        0f32, 0f32, f / (f - n), -n * (f - n),
        0f32, 0f32, 1f32, 0f32
    )
}