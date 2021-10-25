use std::ffi::CStr;

/// Converts a null-terminated UTF-8 C string into a Rust heap-allocated string
pub(crate) fn string_from_array(slice: &[i8]) -> String {
    let u8_array: &[u8] = unsafe { std::mem::transmute(slice) };
    unsafe { String::from_utf8_unchecked(Vec::from(CStr::from_ptr(u8_array.as_ptr() as *const _).to_bytes())) }
}

pub fn string_from_c(c_str: &CStr) -> String {
    unsafe { String::from_utf8_unchecked(Vec::from(c_str.to_bytes())) }
}
