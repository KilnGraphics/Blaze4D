use std::num::NonZeroU32;
use std::panic::{catch_unwind, RefUnwindSafe, UnwindSafe};
use std::process::exit;
use ash::vk;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use crate::b4d::{B4DVertexFormat, Blaze4D};
use crate::glfw_surface::GLFWSurfaceProvider;
use crate::prelude::{Mat4f32, UUID, Vec2f32, Vec2u32, Vec3f32, Vec4f32};

use crate::renderer::emulator::{MeshData, PassRecorder, StaticMeshId};
use crate::renderer::emulator::mc_shaders::{DevUniform, McUniform, McUniformData, ShaderId, VertexFormat, VertexFormatEntry};
use crate::vk::objects::surface::SurfaceProvider;
use crate::window::WinitWindow;

#[repr(C)]
struct NativeMetadata {
    /// The number of bytes of the size type
    size_bytes: u32,
}

const NATIVE_METADATA: NativeMetadata = NativeMetadata {
    size_bytes: std::mem::size_of::<usize>() as u32,
};

#[repr(C)]
#[derive(Debug)]
struct CMeshData {
    vertex_data_ptr: *const u8,
    vertex_data_len: usize,
    index_data_ptr: *const u8,
    index_data_len: usize,
    vertex_stride: u32,
    index_count: u32,
    index_type: i32,
    primitive_topology: i32,
}

impl CMeshData {
    unsafe fn to_mesh_data(&self) -> MeshData {
        if self.vertex_data_ptr.is_null() {
            log::error!("Vertex data pointer is null");
            panic!();
        }
        if self.index_data_ptr.is_null() {
            log::error!("Index data pointer is null");
            panic!();
        }

        MeshData {
            vertex_data: std::slice::from_raw_parts(self.vertex_data_ptr, self.vertex_data_len as usize),
            index_data: std::slice::from_raw_parts(self.index_data_ptr, self.index_data_len as usize),
            vertex_stride: self.vertex_stride,
            index_count: self.index_count,
            index_type: vk::IndexType::from_raw(self.index_type),
            primitive_topology: vk::PrimitiveTopology::from_raw(self.primitive_topology),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct CVertexFormat {
    stride: u32,
    position_offset: u32,
    position_format: i32,
    normal_offset: u32,
    normal_format: i32,
    color_offset: u32,
    color_format: i32,
    uv0_offset: u32,
    uv0_format: i32,
    uv1_offset: u32,
    uv1_format: i32,
    uv2_offset: u32,
    uv2_format: i32,
    has_normal: bool,
    has_color: bool,
    has_uv0: bool,
    has_uv1: bool,
    has_uv2: bool,
}

impl CVertexFormat {
    fn to_vertex_format(&self) -> VertexFormat {
        let normal = if self.has_normal {
            Some(VertexFormatEntry {
                offset: self.normal_offset,
                format: vk::Format::from_raw(self.normal_format)
            })
        } else {
            None
        };

        let color = if self.has_color {
            Some(VertexFormatEntry {
                offset: self.color_offset,
                format: vk::Format::from_raw(self.color_format)
            })
        } else {
            None
        };

        let uv0 = if self.has_uv0 {
            Some( VertexFormatEntry {
                offset: self.uv0_offset,
                format: vk::Format::from_raw(self.uv0_format)
            })
        } else {
            None
        };

        let uv1 = if self.has_uv1 {
            Some(VertexFormatEntry {
                offset: self.uv1_offset,
                format: vk::Format::from_raw(self.uv1_format)
            })
        } else {
            None
        };

        let uv2 = if self.has_uv2 {
            Some(VertexFormatEntry {
                offset: self.uv2_offset,
                format: vk::Format::from_raw(self.uv2_format)
            })
        } else {
            None
        };

        VertexFormat {
            stride: self.stride,
            position: VertexFormatEntry {
                offset: self.position_offset,
                format: vk::Format::from_raw(self.position_format)
            },
            normal,
            color,
            uv0,
            uv1,
            uv2
        }
    }
}

#[repr(C)]
union CMcUniformDataPayload {
    u32: u32,
    f32: f32,
    vec2f32: Vec2f32,
    vec3f32: Vec3f32,
    vec4f32: Vec4f32,
    mat4f32: Mat4f32,
}

#[repr(C)]
struct CMcUniformData {
    uniform: u64,
    payload: CMcUniformDataPayload,
}

impl CMcUniformData {
    unsafe fn to_mc_uniform_data(&self) -> McUniformData {
        match McUniform::from_raw(self.uniform) {
            McUniform::MODEL_VIEW_MATRIX => {
                McUniformData::ModelViewMatrix(self.payload.mat4f32)
            },
            McUniform::PROJECTION_MATRIX => {
                McUniformData::ProjectionMatrix(self.payload.mat4f32)
            },
            McUniform::INVERSE_VIEW_ROTATION_MATRIX => {
                McUniformData::InverseViewRotationMatrix(self.payload.mat4f32)
            },
            McUniform::TEXTURE_MATRIX => {
                McUniformData::TextureMatrix(self.payload.mat4f32)
            },
            McUniform::SCREEN_SIZE => {
                McUniformData::ScreenSize(self.payload.vec2f32)
            },
            McUniform::COLOR_MODULATOR => {
                McUniformData::ColorModulator(self.payload.vec4f32)
            },
            McUniform::LIGHT0_DIRECTION => {
                McUniformData::Light0Direction(self.payload.vec3f32)
            },
            McUniform::LIGHT1_DIRECTION => {
                McUniformData::Light1Direction(self.payload.vec3f32)
            },
            McUniform::FOG_START => {
                McUniformData::FogStart(self.payload.f32)
            },
            McUniform::FOG_END => {
                McUniformData::FogEnd(self.payload.f32)
            },
            McUniform::FOG_COLOR => {
                McUniformData::FogColor(self.payload.vec4f32)
            },
            McUniform::FOG_SHAPE => {
                McUniformData::FogShape(self.payload.u32)
            },
            McUniform::LINE_WIDTH => {
                McUniformData::LineWidth(self.payload.f32)
            },
            McUniform::GAME_TIME => {
                McUniformData::GameTime(self.payload.f32)
            },
            McUniform::CHUNK_OFFSET => {
                McUniformData::ChunkOffset(self.payload.vec3f32)
            },
            _ => {
                log::error!("Invalid uniform type {:?}", self.uniform);
                panic!()
            }
        }
    }
}

/// Returns static information about the natives.
#[no_mangle]
unsafe extern "C" fn b4d_get_native_metadata() -> *const NativeMetadata {
    &NATIVE_METADATA
}

/// Creates a new [`Blaze4D`] instance.
///
/// This function will take ownership of the provided surface and vertex format set builder. The
/// pointers must not be used again afterwards.
#[no_mangle]
unsafe extern "C" fn b4d_init(surface: *mut GLFWSurfaceProvider, enable_validation: u32) -> *mut Blaze4D {
    catch_unwind(|| {
        if surface.is_null() {
            log::error!("Passed null surface to b4d_init");
            exit(1);
        }

        let surface_provider: Box<dyn SurfaceProvider> = Box::from_raw(surface);

        let enable_validation = enable_validation != 0;

        Box::leak(Box::new(Blaze4D::new(surface_provider, enable_validation)))
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_init");
        exit(1);
    })
}

/// Destroys a [`Blaze4D`] instance.
#[no_mangle]
unsafe extern "C" fn b4d_destroy(b4d: *mut Blaze4D) {
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
unsafe extern "C" fn b4d_create_static_mesh(b4d: *const Blaze4D, data: *const CMeshData) -> u64 {
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

        b4d.create_static_mesh(&mesh_data).as_uuid().get_raw()
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_create_static_mesh");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_destroy_static_mesh(b4d: *const Blaze4D, mesh_id: u64) {
    catch_unwind(|| {
        let b4d = b4d.as_ref().unwrap_or_else(|| {
            log::error!("Passed null b4d to b4d_destroy_static_mesh");
            exit(1);
        });

        b4d.drop_static_mesh(StaticMeshId::from_uuid(UUID::from_raw(mesh_id)));
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_destroy_static_mesh");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_create_shader(b4d: *const Blaze4D, vertex_format: *const CVertexFormat, used_uniforms: u64) -> u64 {
    catch_unwind(|| {
        let b4d = b4d.as_ref().unwrap_or_else(|| {
            log::error!("Passed null b4d to b4d_create_shader");
            exit(1);
        });
        let vertex_format = vertex_format.as_ref().unwrap_or_else(|| {
            log::error!("Passed null vertex_format to b4d_create_shader");
            exit(1);
        });

        let vertex_format = vertex_format.to_vertex_format();
        let mc_uniform = McUniform::from_raw(used_uniforms);

        b4d.create_shader(&vertex_format, mc_uniform).as_uuid().get_raw()
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_create_shader");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_destroy_shader(b4d: *const Blaze4D, shader_id: u64) {
    catch_unwind(|| {
        let b4d = b4d.as_ref().unwrap_or_else(|| {
            log::error!("Passed null b4d to b4d_destroy_shader");
            exit(1);
        });

        b4d.drop_shader(ShaderId::from_uuid(UUID::from_raw(shader_id)));
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_destroy_shader");
        exit(1);
    })
}

/// Calls [`Blaze4D::try_start_frame`].
///
/// If [`Blaze4D::try_start_frame`] returns [`None`] this function returns null.
#[no_mangle]
unsafe extern "C" fn b4d_start_frame(b4d: *mut Blaze4D, window_width: u32, window_height: u32) -> *mut PassRecorder {
    catch_unwind(|| {
        let b4d = b4d.as_mut().unwrap_or_else(|| {
            log::error!("Passed null b4d to b4d_start_frame");
            exit(1);
        });

        let frame = b4d.try_start_frame(Vec2u32::new(window_width, window_height));
        frame.map_or(std::ptr::null_mut(), |recorder| {
            Box::leak(Box::new(recorder))
        })
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_start_frame");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_pass_update_uniform(pass: *mut PassRecorder, data: *const CMcUniformData, shader_id: u64) {
    catch_unwind(|| {
        let pass = pass.as_mut().unwrap_or_else(|| {
            log::error!("Passed null pass to b4d_pass_update_dev_uniform");
            exit(1);
        });
        let data = data.as_ref().unwrap_or_else(|| {
            log::error!("Passed null data to b4d_pass_update_dev_uniform");
            exit(1);
        });

        let data = data.to_mc_uniform_data();
        let shader_id = ShaderId::from_uuid(UUID::from_raw(shader_id));

        pass.update_uniform(&data, shader_id);
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_pass_update_dev_uniform");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_pass_draw_static(pass: *mut PassRecorder, mesh_id: u64, shader_id: u64) {
    catch_unwind(|| {
        let pass = pass.as_mut().unwrap_or_else(|| {
            log::error!("Passed null pass to b4d_pass_draw_static");
            exit(1);
        });
        let shader_id = ShaderId::from_uuid(UUID::from_raw(shader_id));

        pass.draw_static(StaticMeshId::from_uuid(UUID::from_raw(mesh_id)), shader_id);
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_pass_draw_static");
        exit(1);
    })
}

#[no_mangle]
unsafe extern "C" fn b4d_pass_draw_immediate(pass: *mut PassRecorder, data: *const CMeshData, shader_id: u64) {
    catch_unwind(|| {
        let pass = pass.as_mut().unwrap_or_else(|| {
            log::error!("Passed null pass to b4d_pass_draw_immediate");
            exit(1);
        });
        let data = data.as_ref().unwrap_or_else(|| {
            log::error!("Passed null mesh data to b4d_pass_draw_immediate");
            exit(1);
        });
        let shader_id = ShaderId::from_uuid(UUID::from_raw(shader_id));

        let mesh_data = data.to_mesh_data();

        pass.draw_immediate(&mesh_data, shader_id);
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