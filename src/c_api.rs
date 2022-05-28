use std::panic::catch_unwind;
use std::process::exit;
use crate::b4d::Blaze4D;

use crate::renderer::emulator::{VertexFormatInfo, VertexFormatSetBuilder};

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
/// This function will take ownership of the provided vertex format set builder. The pointer must
/// not be used again afterwards.
#[no_mangle]
unsafe extern "C" fn b4d_init(vertex_formats: *mut VertexFormatSetBuilder, enable_validation: bool) -> *mut Blaze4D {
    catch_unwind(|| {
        let vertex_formats = *Box::from_raw(vertex_formats);

        todo!()
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_init");
        exit(1);
    })
}

/// Destroys a [`Blaze4D`] instance.
#[no_mangle]
unsafe extern "C" fn b4d_destroy(instance: *mut Blaze4D) {
    catch_unwind(|| {
        Box::from_raw(instance);
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_destroy");
        exit(1);
    })
}