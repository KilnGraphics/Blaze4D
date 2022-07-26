extern crate b4d_core;

use ash::vk;
use bytemuck::{cast_slice, Pod, Zeroable};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use b4d_core::prelude::*;
use b4d_core::renderer::emulator::debug_pipeline::DebugPipelineMode;
use b4d_core::renderer::emulator::mc_shaders::{McUniform, McUniformData, VertexFormat, VertexFormatEntry};
use b4d_core::renderer::emulator::MeshData;

use b4d_core::window::WinitWindow;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let event_loop = EventLoop::new();
    let window = Box::new(WinitWindow::new("ImmediateCube", 800.0, 600.0, &event_loop));

    let b4d = b4d_core::b4d::Blaze4D::new(window, true);
    b4d.set_debug_mode(Some(DebugPipelineMode::Textured0));
    let vertex_format = Vertex::make_b4d_vertex_format();
    let mut shader = b4d.create_shader(&vertex_format, McUniform::MODEL_VIEW_MATRIX | McUniform::PROJECTION_MATRIX);

    let data = MeshData {
        vertex_data: cast_slice(&CUBE_VERTICES),
        index_data: cast_slice(&CUBE_INDICES),
        vertex_stride: std::mem::size_of::<Vertex>() as u32,
        index_count: CUBE_INDICES.len() as u32,
        index_type: vk::IndexType::UINT32,
        primitive_topology: vk::PrimitiveTopology::TRIANGLE_LIST,
    };

    let mut mesh = b4d.create_global_mesh(&data);

    let mut draw_times = Vec::with_capacity(1000);
    let mut last_update = std::time::Instant::now();

    let mut current_size = Vec2u32::new(800, 600);

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

                mesh = b4d.create_global_mesh(&data);

                if let Some(mut recorder) = b4d.try_start_frame(current_size) {

                    recorder.update_uniform(&McUniformData::ProjectionMatrix(make_projection_matrix(current_size, 90f32)), shader);

                    let elapsed = start.elapsed().as_secs_f32();
                    let rotation = Mat4f32::new_rotation(Vec3f32::new(elapsed / 2.34f32, elapsed / 2.783f32, elapsed / 2.593f32));

                    for x in -5i32..=5i32 {
                        for y in -5i32..=5i32 {
                            for z in 1i32..=11i32 {
                                let translation = Mat4f32::new_translation(&Vec3f32::new(
                                    0f32 + ((x as f32) / 1f32),
                                    0f32 + ((y as f32) / 1f32),
                                    5f32 + ((z as f32) / 1f32)
                                ));
                                recorder.update_uniform(&McUniformData::ModelViewMatrix(translation * rotation), shader);

                                let id = recorder.upload_immediate(&data);
                                recorder.draw_immediate(id, shader, true);
                                // recorder.draw_global(mesh.clone(), shader, true);
                            }
                        }
                    }

                    drop(recorder);

                    // Stress test the shader stuff
                    b4d.drop_shader(shader);
                    shader = b4d.create_shader(&vertex_format, McUniform::MODEL_VIEW_MATRIX | McUniform::PROJECTION_MATRIX);
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
        color: Vec4f32::new(0f32, 0f32, 0f32, 1f32),
        uv: Vec2f32::new(0f32, 0f32),
    },
    Vertex {
        position: Vec3f32::new(1f32, -1f32, -1f32),
        color: Vec4f32::new(1f32, 0f32, 0f32, 1f32),
        uv: Vec2f32::new(1f32, 0f32),
    },
    Vertex {
        position: Vec3f32::new(-1f32, 1f32, -1f32),
        color: Vec4f32::new(0f32, 1f32, 0f32, 1f32),
        uv: Vec2f32::new(0f32, 1f32),
    },
    Vertex {
        position: Vec3f32::new(1f32, 1f32, -1f32),
        color: Vec4f32::new(1f32, 1f32, 0f32, 1f32),
        uv: Vec2f32::new(1f32, 1f32),
    },
    Vertex {
        position: Vec3f32::new(-1f32, -1f32, 1f32),
        color: Vec4f32::new(0f32, 0f32, 1f32, 1f32),
        uv: Vec2f32::new(0f32, 0f32),
    },
    Vertex {
        position: Vec3f32::new(1f32, -1f32, 1f32),
        color: Vec4f32::new(1f32, 0f32, 1f32, 1f32),
        uv: Vec2f32::new(1f32, 0f32),
    },
    Vertex {
        position: Vec3f32::new(-1f32, 1f32, 1f32),
        color: Vec4f32::new(0f32, 1f32, 1f32, 1f32),
        uv: Vec2f32::new(0f32, 1f32),
    },
    Vertex {
        position: Vec3f32::new(1f32, 1f32, 1f32),
        color: Vec4f32::new(1f32, 1f32, 1f32, 1f32),
        uv: Vec2f32::new(1f32, 1f32),
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
    #[allow(unused)]
    position: Vec3f32,
    #[allow(unused)]
    color: Vec4f32,
    #[allow(unused)]
    uv: Vec2f32,
}

impl Vertex {
    fn make_b4d_vertex_format() -> VertexFormat {
        VertexFormat {
            stride: std::mem::size_of::<Vertex>() as u32,
            position: VertexFormatEntry { offset: 0, format: vk::Format::R32G32B32_SFLOAT },
            normal: None,
            color: Some(VertexFormatEntry { offset: std::mem::size_of::<Vec3f32>() as u32, format: vk::Format::R32G32B32A32_SFLOAT }),
            uv0: Some(VertexFormatEntry { offset: std::mem::size_of::<Vec3f32>() as u32 + std::mem::size_of::<Vec4f32>() as u32, format: vk::Format::R32G32_SFLOAT }),
            uv1: None,
            uv2: None
        }
    }
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

fn make_projection_matrix(window_size: Vec2u32, fov: f32) -> Mat4f32 {
    let t = (fov / 2f32).tan();
    let a1 = (window_size[1] as f32) / (window_size[0] as f32);

    let f = 15f32;
    let n = 0.5f32;

    Mat4f32::new(
        a1 / t, 0f32, 0f32, 0f32,
        0f32, 1f32 / t, 0f32, 0f32,
        0f32, 0f32, f / (f - n), -n * (f - n),
        0f32, 0f32, 1f32, 0f32
    )
}