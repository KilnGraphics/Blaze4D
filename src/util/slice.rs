use std::mem::size_of;

pub fn from_byte_slice<T: Copy + Sized>(src: &[u8]) -> &[T] {
    let len = src.len() / size_of::<T>();

    if len * size_of::<T>() != src.len() {
        panic!("Slice size is not a multiple of target type size");
    }

    unsafe { std::slice::from_raw_parts(src.as_ptr() as *const T, len) }
}

pub fn to_byte_slice<T: Copy + Sized>(src: &[T]) -> &[u8] {
    let len = src.len() * size_of::<T>();
    unsafe { std::slice::from_raw_parts(src.as_ptr() as *const u8, len) }
}