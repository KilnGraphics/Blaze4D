use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;
use ash::vk;
use crate::vk::objects::surface::{SurfaceInitError, SurfaceProvider};

#[allow(non_camel_case_types)]
pub type PFN_glfwInitVulkanLoader = unsafe extern "C" fn(vk::PFN_vkGetInstanceProcAddr);

#[allow(non_camel_case_types)]
pub type PFN_glfwGetRequiredInstanceExtensions = unsafe extern "C" fn(*mut u32) -> *const *const c_char;

#[allow(non_camel_case_types)]
pub type PFN_glfwCreateWindowSurface = unsafe extern "C" fn(vk::Instance, *const c_void, *const vk::AllocationCallbacks, *mut vk::SurfaceKHR) -> vk::Result;

pub struct GLFWSurfaceProvider {
    required_extension: Vec<CString>,
    create_surface_fn: PFN_glfwCreateWindowSurface,
    glfw_window: *const c_void,
    surface: Option<(vk::SurfaceKHR, ash::extensions::khr::Surface)>,
}

impl GLFWSurfaceProvider {
    pub fn new(
        window: *const c_void,
        glfw_get_required_instance_extensions: PFN_glfwGetRequiredInstanceExtensions,
        glfw_create_window_surface: PFN_glfwCreateWindowSurface,
    ) -> Self {
        let mut count = 0u32;
        let extensions = unsafe { glfw_get_required_instance_extensions(&mut count) };
        if extensions.is_null() {
            panic!("Extensions returned by glfwGetRequiredInstanceExtensions is null");
        }

        let extensions = unsafe { std::slice::from_raw_parts(extensions, count as usize) };
        let extensions: Vec<_> = extensions.into_iter().map(|str| {
            unsafe { CString::from(CStr::from_ptr(*str)) }
        }).collect();

        Self {
            required_extension: extensions,
            create_surface_fn: glfw_create_window_surface,
            glfw_window: window,
            surface: None
        }
    }
}

impl SurfaceProvider for GLFWSurfaceProvider {
    fn get_required_instance_extensions(&self) -> Vec<CString> {
        self.required_extension.clone()
    }

    fn init(&mut self, entry: &ash::Entry, instance: &ash::Instance) -> Result<vk::SurfaceKHR, SurfaceInitError> {
        let surface_khr = ash::extensions::khr::Surface::new(entry, instance);

        let mut surface = vk::SurfaceKHR::null();
        unsafe { (self.create_surface_fn)(instance.handle(), self.glfw_window, std::ptr::null(), &mut surface) }.result()?;
        self.surface = Some((surface, surface_khr));

        Ok(surface)
    }

    fn get_handle(&self) -> Option<vk::SurfaceKHR> {
        self.surface.as_ref().map(|s| s.0)
    }
}

// THIS IS NOT CORRECT!!! TODO find a better way
unsafe impl Send for GLFWSurfaceProvider {
}
unsafe impl Sync for GLFWSurfaceProvider {
}

impl Drop for GLFWSurfaceProvider {
    fn drop(&mut self) {
        self.surface.take().map(|s| {
            unsafe { s.1.destroy_surface(s.0, None) };
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn b4d_pre_init_glfw(func: PFN_glfwInitVulkanLoader) {
    let entry = ash::Entry::linked();
    func(entry.static_fn().get_instance_proc_addr);
}

#[no_mangle]
pub unsafe extern "C" fn b4d_create_glfw_surface_provider(
    window: *const c_void,
    glfw_get_required_instance_extensions: PFN_glfwGetRequiredInstanceExtensions,
    glfw_create_window_surface: PFN_glfwCreateWindowSurface,
) -> *mut GLFWSurfaceProvider {
    Box::leak(Box::new(GLFWSurfaceProvider::new(
        window,
        glfw_get_required_instance_extensions,
        glfw_create_window_surface
    )))
}