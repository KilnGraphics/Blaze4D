use std::panic::{catch_unwind, RefUnwindSafe, UnwindSafe};
use std::process::exit;
use ash::vk;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use crate::b4d::Blaze4D;
use crate::glfw_surface::GLFWSurfaceProvider;
use crate::prelude::{Mat4f32, UUID, Vec2u32};

use crate::renderer::emulator::{MeshData, PassRecorder, StaticMeshId, VertexFormatId, VertexFormatInfo, VertexFormatSetBuilder};
use crate::window::WinitWindow;

#[repr(C)]
struct CMeshData {
    vertex_data_ptr: *const u8,
    vertex_data_len: u64,
    index_data_ptr: *const u8,
    index_data_len: u64,
    index_count: u32,
    vertex_format_id: u32,
}

impl CMeshData {
    unsafe fn to_mesh_data(&self) -> MeshData {
        MeshData {
            vertex_data: std::slice::from_raw_parts(self.vertex_data_ptr, self.vertex_data_len as usize),
            index_data: std::slice::from_raw_parts(self.index_data_ptr, self.index_data_len as usize),
            index_count: self.index_count,
            index_type: vk::IndexType::UINT32,
            vertex_format_id: VertexFormatId::from_raw(self.vertex_format_id),
        }
    }
}

/// Creates a new [`VertexFormatSetBuilder`] instance and allocates memory for it.
#[no_mangle]
unsafe extern "C" fn b4d_emulator_vertex_format_set_builder_new() -> *mut VertexFormatSetBuilder {
    catch_unwind(|| {
        Box::leak(Box::new(VertexFormatSetBuilder::new())) as *mut VertexFormatSetBuilder
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_emulator_vertex_format_set_builder_new");
        exit(1);
    })
}

/// Destroys a [`VertexFormatSetBuilder`] instance and frees its memory.
#[no_mangle]
unsafe extern "C" fn b4d_emulator_vertex_format_set_builder_destroy(builder: *mut VertexFormatSetBuilder) {
    catch_unwind(|| {
        if builder.is_null() {
            log::error!("Passed null builder to b4d_emulator_vertex_format_set_builder_destroy");
            exit(1);
        }
        Box::from_raw(builder);
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_emulator_vertex_format_set_builder_destroy");
        exit(1);
    })
}

/// Calls [`VertexFormatSetBuilder::add_format`] on the provided builder.
#[no_mangle]
unsafe extern "C" fn b4d_emulator_vertex_format_builder_add_format(builder: *mut VertexFormatSetBuilder, stride: u32) {
    catch_unwind(|| {
        builder.as_mut().unwrap_or_else(|| {
            log::error!("Passed null builder to b4d_emulator_vertex_format_builder_add_format");
            exit(1);
        }).add_format(VertexFormatInfo {
            stride: stride as usize
        });
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_emulator_vertex_format_builder_add_format");
        exit(1);
    })
}

/// Creates a new [`Blaze4D`] instance.
///
/// This function will take ownership of the provided surface and vertex format set builder. The
/// pointers must not be used again afterwards.
#[no_mangle]
unsafe extern "C" fn b4d_init(surface: *mut GLFWSurfaceProvider, vertex_formats: *mut VertexFormatSetBuilder, enable_validation: bool) -> *mut B4DDebugWrapper {
    catch_unwind(|| {
        /*if surface.is_null() {
            log::error!("Passed null surface to b4d_init");
            exit(1);
        }*/
        if vertex_formats.is_null() {
            log::error!("Passed null vertex formats to b4d_init");
            exit(1);
        }

        // let surface_provider: Box<dyn SurfaceProvider> = Box::from_raw(surface);
        let vertex_formats = *Box::from_raw(vertex_formats);

        Box::leak(Box::new(B4DDebugWrapper::new(vertex_formats, enable_validation)))
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_init");
        exit(1);
    })
}

/// Destroys a [`Blaze4D`] instance.
#[no_mangle]
unsafe extern "C" fn b4d_destroy(b4d: *mut B4DDebugWrapper) {
    catch_unwind(|| {
        if b4d.is_null() {
            log::error!("Passed null to b4d_destroy");
            exit(1);
        }
        Box::from_raw(b4d);
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_destroy");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_create_static_mesh(b4d: *const B4DDebugWrapper, data: *const CMeshData) -> u64 {
    catch_unwind(|| {
        let b4d = b4d.as_ref().unwrap_or_else(|| {
            log::error!("Passed null b4d to b4d_create_static_mesh");
            exit(1);
        });
        let data = data.as_ref().unwrap_or_else(|| {
            log::error!("Passed null mesh data to b4d_create_static_mesh");
            exit(1);
        });

        let mesh_data = data.to_mesh_data();

        b4d.b4d.create_static_mesh(&mesh_data).as_uuid().get_raw()
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_create_static_mesh");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_destroy_static_mesh(b4d: *const B4DDebugWrapper, mesh_id: u64) {
    catch_unwind(|| {
        let b4d = b4d.as_ref().unwrap_or_else(|| {
            log::error!("Passed null b4d to b4d_destroy_static_mesh");
            exit(1);
        });

        b4d.b4d.drop_static_mesh(StaticMeshId::from_uuid(UUID::from_raw(mesh_id)));
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_destroy_static_mesh");
        exit(1);
    })
}

/// Calls [`Blaze4D::try_start_frame`].
///
/// If [`Blaze4D::try_start_frame`] returns [`None`] this function returns null.
#[no_mangle]
unsafe extern "C" fn b4d_start_frame(b4d: *mut B4DDebugWrapper, window_width: u32, window_height: u32) -> *mut PassRecorder {
    catch_unwind(|| {
        let b4d = b4d.as_mut().unwrap_or_else(|| {
            log::error!("Passed null b4d to b4d_start_frame");
            exit(1);
        });

        let frame = b4d.start_frame(Vec2u32::new(window_width, window_height));
        frame.map_or(std::ptr::null_mut(), |recorder| {
            Box::leak(Box::new(recorder))
        })
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_start_frame");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_pass_set_model_view_matrix(pass: *mut PassRecorder, matrix: *const Mat4f32) {
    catch_unwind(|| {
        let pass = pass.as_mut().unwrap_or_else(|| {
            log::error!("Passed null pass to b4d_pass_set_model_view_matrix");
            exit(1);
        });
        let matrix = matrix.as_ref().unwrap_or_else(|| {
            log::error!("Passed null matrix to b4d_pass_set_model_view_matrix");
            exit(1);
        });

        pass.set_model_view_matrix(*matrix);
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_pass_set_model_view_matrix");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_pass_set_projection_matrix(pass: *mut PassRecorder, matrix: *const Mat4f32) {
    catch_unwind(|| {
        let pass = pass.as_mut().unwrap_or_else(|| {
            log::error!("Passed null pass to b4d_pass_set_projection_matrix");
            exit(1);
        });
        let matrix = matrix.as_ref().unwrap_or_else(|| {
            log::error!("Passed null matrix to b4d_pass_set_projection_matrix");
            exit(1);
        });

        pass.set_projection_matrix(*matrix);
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_pass_set_model_view_matrix");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_pass_draw_static(pass: *mut PassRecorder, mesh_id: u64, type_id: u32) {
    catch_unwind(|| {
        let pass = pass.as_mut().unwrap_or_else(|| {
            log::error!("Passed null pass to b4d_pass_draw_static");
            exit(1);
        });

        pass.draw_static(StaticMeshId::from_uuid(UUID::from_raw(mesh_id)), type_id);
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_pass_draw_static");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_pass_draw_immediate(pass: *mut PassRecorder, data: *const CMeshData, type_id: u32) {
    catch_unwind(|| {
        let pass = pass.as_mut().unwrap_or_else(|| {
            log::error!("Passed null pass to b4d_pass_draw_immediate");
            exit(1);
        });
        let data = data.as_ref().unwrap_or_else(|| {
            log::error!("Passed null mesh data to b4d_pass_draw_immediate");
            exit(1);
        });

        let mesh_data = data.to_mesh_data();

        pass.draw_immediate(&mesh_data, type_id);
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_pass_draw_immediate");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_end_frame(recorder: *mut PassRecorder) {
    catch_unwind(|| {
        if recorder.is_null() {
            log::error!("Passed null to b4d_end_frame");
            exit(1);
        }
        Box::from_raw(recorder);
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_end_frame");
        exit(1);
    })
}

/// For now we want to draw to a separate window from the main minecraft window. This way we can
/// easily compare results. This struct wraps a [`Blaze4D`] instance and uses a winit window.
struct B4DDebugWrapper {
    b4d: Blaze4D,
    event_loop: EventLoop<()>,
    winit_window_size: Vec2u32,
}

impl B4DDebugWrapper {
    fn new(vertex_formats: VertexFormatSetBuilder, enable_validation: bool) -> Self {
        env_logger::init();

        let event_loop = EventLoop::new();
        let window = Box::new(WinitWindow::new("Blaze4D", 800.0, 600.0, &event_loop));

        let b4d = Blaze4D::new(window, vertex_formats, enable_validation);

        Self {
            b4d,
            event_loop,
            winit_window_size: Vec2u32::new(800, 600),
        }
    }

    fn start_frame(&mut self, _: Vec2u32) -> Option<PassRecorder> {
        self.event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    self.winit_window_size = Vec2u32::new(new_size.width, new_size.height);
                },
                Event::MainEventsCleared => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {
                }
            }
        });

        self.b4d.try_start_frame(self.winit_window_size)
    }
}

impl UnwindSafe for B4DDebugWrapper {
}
impl RefUnwindSafe for B4DDebugWrapper {
}