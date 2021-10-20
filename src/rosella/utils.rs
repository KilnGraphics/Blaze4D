use std::ffi::CStr;

///
pub(crate) fn string_from_array(slice: &[i8]) -> String {
    let u8_array: &[u8] = unsafe { std::mem::transmute(slice) };
    unsafe { String::from_utf8_unchecked(Vec::from(CStr::from_ptr(u8_array.as_ptr() as *const _).to_bytes())) }
}