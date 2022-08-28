use std::os::raw::c_char;

pub struct TmpEmulator;

#[no_mangle]
unsafe extern "C" fn b4d_emulator_gl_create_shader(emulator: *mut TmpEmulator, ty: u32) -> u32 {
    todo!()
}

#[no_mangle]
unsafe extern "C" fn b4d_emulator_gl_shader_source(emulator: *mut TmpEmulator, shader: u32, count: u32, source: *const *const c_char, length: *const i32) {
    todo!()
}

#[no_mangle]
unsafe extern "C" fn b4d_emulator_gl_compile_shader(emulator: *mut TmpEmulator, shader: u32) {
    todo!()
}

#[no_mangle]
unsafe extern "C" fn b4d_emulator_gl_delete_shader(emulator: *mut TmpEmulator, shader: u32) {
    todo!()
}

#[no_mangle]
unsafe extern "C" fn b4d_emulator_gl_create_program(emulator: *mut TmpEmulator) -> u32 {
    todo!()
}

#[no_mangle]
unsafe extern "C" fn b4d_emulator_gl_attach_shader(emulator: *mut TmpEmulator, program: u32, shader: u32) {
    todo!()
}

#[no_mangle]
unsafe extern "C" fn b4d_emulator_gl_link_program(emulator: *mut TmpEmulator, program: u32) {
    todo!()
}

#[no_mangle]
unsafe extern "C" fn b4d_emulator_gl_use_program(emulator: *mut TmpEmulator, program: u32) {
    todo!()
}

#[no_mangle]
unsafe extern "C" fn b4d_emulator_gl_delete_program(emulator: *mut TmpEmulator, program: u32) {
    todo!()
}