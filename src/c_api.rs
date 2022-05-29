use std::panic::catch_unwind;
use std::process::exit;
use crate::b4d::Blaze4D;
use crate::glfw_surface::GLFWSurfaceProvider;
use crate::prelude::Vec2u32;

use crate::renderer::emulator::{PassRecorder, VertexFormatInfo, VertexFormatSetBuilder};
use crate::vk::objects::surface::SurfaceProvider;

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
unsafe extern "C" fn b4d_init(surface: *mut GLFWSurfaceProvider, vertex_formats: *mut VertexFormatSetBuilder, enable_validation: bool) -> *mut Blaze4D {
    catch_unwind(|| {
        if surface.is_null() {
            log::error!("Passed null surface to b4d_init");
            exit(1);
        }
        if vertex_formats.is_null() {
            log::error!("Passed null vertex formats to b4d_init");
            exit(1);
        }

        let surface_provider: Box<dyn SurfaceProvider> = Box::from_raw(surface);
        let vertex_formats = *Box::from_raw(vertex_formats);

        Box::leak(Box::new(Blaze4D::new(surface_provider, vertex_formats, enable_validation)))
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

/// Calls [`Blaze4D::try_start_frame`].
///
/// If [`Blaze4D::try_start_frame`] returns [`None`] this function returns null.
#[no_mangle]
unsafe extern "C" fn b4d_start_frame(b4d: *const Blaze4D, window_width: u32, window_height: u32) -> *mut PassRecorder {
    catch_unwind(|| {
        let b4d = b4d.as_ref().unwrap_or_else(|| {
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