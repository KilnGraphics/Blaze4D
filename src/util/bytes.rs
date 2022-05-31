//! Utilities to convert structs to and from byte arrays.

use std::ptr::{addr_of, NonNull};

/// Designates a type that can be safely viewed as a byte array.
///
/// # Safety
/// In some cases (for example in structs with interior mutability) this is not safe to implement.
pub unsafe trait ToBytes {
    /// Creates a u8 slice view of the struct.
    fn as_bytes(&self) -> &[u8] where Self: Sized {
        let size = std::mem::size_of::<Self>();
        unsafe {
            std::slice::from_raw_parts(self as *const Self as *const u8, size)
        }
    }
}

/// Designates a type that can be safely viewed and modified as a byte array.
///
/// # Safety
/// On top of the safety requirements of [`AsBytes`] this trait has additional issues for example
/// it must not be implemented for types which contain pointers or references.
pub unsafe trait FromBytes: ToBytes {
    /// Creates a mutable u8 slice view of the struct.
    fn as_bytes_mut(&mut self) -> &mut [u8] where Self: Sized {
        let size = std::mem::size_of::<Self>();
        unsafe {
            std::slice::from_raw_parts_mut(self as *mut Self as *mut u8, size)
        }
    }

    /// Creates a reference from a u8 slice.
    ///
    /// Returns [`None`] if the slice cannot be converted to Self. This can happen for a variety of
    /// reasons most notably if the size of the slice does not match the size of Self exactly or
    /// if the slice does not satisfy the alignment requirements of self.
    fn try_from_bytes(bytes: &[u8]) -> Option<&Self> where Self: Sized {
        let size = std::mem::size_of::<Self>();
        if size != bytes.len() {
            return None;
        }
        if !is_well_aligned::<Self>(bytes) {
            return None;
        }

        unsafe {
            let ptr = NonNull::new_unchecked(bytes.as_ptr() as *mut u8); // Why no const NonNull?
            Some(ptr.cast::<Self>().as_ref())
        }
    }

    /// Creates a mutable reference from a u8 slice.
    ///
    /// Returns [`None`] if the slice cannot be converted to Self. This can happen for a variety of
    /// reasons most notably if the size of the slice does not match the size of Self exactly or
    /// if the slice does not satisfy the alignment requirements of self.
    fn try_from_bytes_mut(bytes: &mut [u8]) -> Option<&mut Self> where Self: Sized {
        let size = std::mem::size_of::<Self>();
        if size != bytes.len() {
            return None;
        }
        if !is_well_aligned::<Self>(bytes) {
            return None;
        }

        unsafe {
            let ptr = NonNull::new_unchecked(bytes.as_mut_ptr());
            Some(ptr.cast::<Self>().as_mut())
        }
    }
}

unsafe impl<T: ToBytes + Sized> ToBytes for [T] {
    fn as_bytes(&self) -> &[u8] {
        let byte_count = self.len() * std::mem::size_of::<T>();
        unsafe {
            std::slice::from_raw_parts(self.as_ptr() as *const u8, byte_count)
        }
    }
}

unsafe impl<T: FromBytes + Sized> FromBytes for [T] {
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        let byte_count = self.len() * std::mem::size_of::<T>();
        unsafe {
            std::slice::from_raw_parts_mut(self.as_mut_ptr() as *mut u8, byte_count)
        }
    }

    fn try_from_bytes(bytes: &[u8]) -> Option<&Self> {
        let count = bytes.len() / std::mem::size_of::<T>();
        if count * std::mem::size_of::<T>() != bytes.len() {
            return None; // byte size is not multiple of T size
        }
        if !is_well_aligned::<T>(bytes) {
            return None;
        }

        unsafe {
            Some(std::slice::from_raw_parts(bytes.as_ptr() as *const T, count))
        }
    }

    fn try_from_bytes_mut(bytes: &mut [u8]) -> Option<&mut Self> {
        let count = bytes.len() / std::mem::size_of::<T>();
        if count * std::mem::size_of::<T>() != bytes.len() {
            return None; // byte size is not multiple of T size
        }
        if !is_well_aligned::<T>(bytes) {
            return None;
        }

        unsafe {
            Some(std::slice::from_raw_parts_mut(bytes.as_mut_ptr() as *mut T, count))
        }
    }
}

/// Returns true if the bytes slice is well aligned to be used as memory for the type T
///
/// This function only validates alignment and not the size of the slice.
pub fn is_well_aligned<T: Sized>(bytes: &[u8]) -> bool {
    (bytes.as_ptr() as usize) % std::mem::size_of::<T>() == 0
}

unsafe impl ToBytes for u8 {}
unsafe impl ToBytes for i8 {}
unsafe impl ToBytes for u16 {}
unsafe impl ToBytes for i16 {}
unsafe impl ToBytes for u32 {}
unsafe impl ToBytes for i32 {}
unsafe impl ToBytes for u64 {}
unsafe impl ToBytes for i64 {}
unsafe impl ToBytes for u128 {}
unsafe impl ToBytes for i128 {}
unsafe impl ToBytes for f32 {}
unsafe impl ToBytes for f64 {}
unsafe impl ToBytes for usize {}
unsafe impl ToBytes for isize {}
unsafe impl ToBytes for char {}

unsafe impl FromBytes for u8 {}
unsafe impl FromBytes for i8 {}
unsafe impl FromBytes for u16 {}
unsafe impl FromBytes for i16 {}
unsafe impl FromBytes for u32 {}
unsafe impl FromBytes for i32 {}
unsafe impl FromBytes for u64 {}
unsafe impl FromBytes for i64 {}
unsafe impl FromBytes for u128 {}
unsafe impl FromBytes for i128 {}
unsafe impl FromBytes for f32 {}
unsafe impl FromBytes for f64 {}
unsafe impl FromBytes for usize {}
unsafe impl FromBytes for isize {}
unsafe impl FromBytes for char {}