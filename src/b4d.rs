use crate::glfw_surface::GLFWFunctions;

pub struct Blaze4D {
    glfw_functions: GLFWFunctions,
}

impl Blaze4D {
    pub fn new(glfw_functions: &GLFWFunctions) -> Self {
        Self {
            glfw_functions: *glfw_functions,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn b4d_init(glfw_functions: *const GLFWFunctions) -> *mut Blaze4D {
    let glfw_functions = unsafe { glfw_functions.as_ref() }.unwrap();
    assert!(glfw_functions.validate_complete());

    let b4d = Box::leak(Box::new(Blaze4D::new(glfw_functions)));

    b4d
}

#[no_mangle]
pub unsafe extern "C" fn b4d_destroy(b4d: *mut Blaze4D) {
    Box::from_raw(b4d);
}