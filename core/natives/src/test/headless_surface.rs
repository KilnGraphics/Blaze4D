use std::ffi::CString;

use ash::{Entry, Instance};
use ash::vk;

use crate::vk::objects::surface::{SurfaceInitError, SurfaceProvider};

pub struct HeadlessSurfaceProvider {
    surface: Option<(vk::SurfaceKHR, ash::extensions::khr::Surface)>,
}

impl HeadlessSurfaceProvider {
    pub const REQUIRED_INSTANCE_EXTENSIONS: [&'static str; 2] = [
        "VK_KHR_surface",
        "VK_EXT_headless_surface",
    ];
}

impl SurfaceProvider for HeadlessSurfaceProvider {
    fn get_required_instance_extensions(&self) -> Vec<CString> {
        HeadlessSurfaceProvider::REQUIRED_INSTANCE_EXTENSIONS.iter().map(CString::new).collect::<Result<_, _>>().unwrap()
    }

    unsafe fn init(&mut self, entry: &Entry, instance: &Instance) -> Result<vk::SurfaceKHR, SurfaceInitError> {
        if self.surface.is_some() {
            panic!("Called HeadlessSurfaceProvider::init on already initialized instance");
        }

        let surface_khr = ash::extensions::khr::Surface::new(entry, instance);
        let headless_surface_khr = ash::extensions::ext::HeadlessSurface::new(entry, instance);

        let info = vk::HeadlessSurfaceCreateInfoEXT::builder();

        let surface = headless_surface_khr.create_headless_surface(&info, None)?;
        self.surface = Some((surface, surface_khr));
        Ok(surface)
    }

    unsafe fn destroy(&mut self) {
        let (surface, surface_khr) = self.surface.take().expect("Called HeadlessSurfaceProvider::destroy on uninitialized instance");
        surface_khr.destroy_surface(surface, None);
    }

    unsafe fn get_handle(&self) -> vk::SurfaceKHR {
        *self.surface.as_ref().expect("Called HeadlessSurfaceProvider::get_handle on uninitialized instance").0
    }
}

impl Drop for HeadlessSurfaceProvider {
    fn drop(&mut self) {
        if self.surface.is_some() {
            panic!("HeadlessSurfaceProvider is being dropped before destroy was called!");
        }
    }
}