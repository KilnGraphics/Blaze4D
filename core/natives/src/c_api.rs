use std::panic::catch_unwind;
use std::process::exit;

use crate::b4d::Blaze4D;
use crate::glfw_surface::GLFWSurfaceProvider;

use crate::vk::objects::surface::SurfaceProvider;

#[repr(C)]
struct NativeMetadata {
    /// The number of bytes of the size type
    size_bytes: u32,
}

const NATIVE_METADATA: NativeMetadata = NativeMetadata {
    size_bytes: std::mem::size_of::<usize>() as u32,
};

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
        let _ = Box::from_raw(b4d);
    }).unwrap_or_else(|_| {
        log::error!("panic in b4d_destroy");
        exit(1);
    })
}