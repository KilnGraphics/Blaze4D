use std::ffi::c_void;
use std::os::raw::c_char;
use ash::vk;

#[allow(non_camel_case_types)]
pub type PFN_glfwInitVulkanLoader = unsafe extern "C" fn(vk::PFN_vkGetInstanceProcAddr);

#[allow(non_camel_case_types)]
pub type PFN_glfwGetRequiredInstanceExtensions = unsafe extern "C" fn(*mut u32) -> *const *const c_char;

#[allow(non_camel_case_types)]
pub type PFN_glfwCreateWindowSurface = unsafe extern "C" fn(vk::Instance, *const c_void, *const vk::AllocationCallbacks, *mut vk::SurfaceKHR) -> vk::Result;

#[derive(Copy, Clone)]
pub struct GLFWFunctions {
    glfw_get_required_instance_extensions: Option<PFN_glfwGetRequiredInstanceExtensions>,
    glfw_create_window_surface: Option<PFN_glfwCreateWindowSurface>,
}

impl GLFWFunctions {
    pub fn new() -> Self {
        Self {
            glfw_get_required_instance_extensions: None,
            glfw_create_window_surface: None,
        }
    }

    pub fn validate_complete(&self) -> bool {
        self.glfw_get_required_instance_extensions.is_some() &&
            self.glfw_create_window_surface.is_some()
    }
}

#[no_mangle]
pub unsafe extern "C" fn b4d_create_glfw_functions() -> *mut GLFWFunctions {
    Box::leak(Box::new(GLFWFunctions::new()))
}

#[no_mangle]
pub unsafe extern "C" fn b4d_destroy_glfw_functions(functions: *mut GLFWFunctions) {
    Box::from_raw(functions);
}

#[no_mangle]
pub unsafe extern "C" fn b4d_set_PFN_glfwGetRequiredInstanceExtensions(table: *mut GLFWFunctions, func: PFN_glfwGetRequiredInstanceExtensions) {
    table.as_mut().unwrap().glfw_get_required_instance_extensions = Some(func);
}

#[no_mangle]
pub unsafe extern "C" fn b4d_set_PFN_glfwCreateWindowSurface(table: *mut GLFWFunctions, func: PFN_glfwCreateWindowSurface) {
    table.as_mut().unwrap().glfw_create_window_surface = Some(func);
}

#[no_mangle]
pub unsafe extern "C" fn b4d_pre_init_glfw(func: PFN_glfwInitVulkanLoader) {
    let entry = ash::Entry::linked();
    func(entry.static_fn().get_instance_proc_addr);
}