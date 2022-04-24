use std::ffi::c_void;
use std::ptr::NonNull;
use ash::vk;
use crate::vk::objects::allocator::MappedMemory;
use crate::vk::objects::Format;
use crate::vk::objects::types::BufferId;

#[derive(Copy, Clone, Debug)]
pub struct BufferSpec {
    pub size: u64,
}

impl BufferSpec {
    pub const fn new(size: u64) -> Self {
        BufferSpec { size }
    }

    pub const fn get_size(&self) -> u64 {
        self.size
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BufferRange {
    pub offset: u64,
    pub length: u64,
}

/// Contains a description for a vulkan buffer.
///
/// This only contains static information relevant to vulkan (i.e. size or supported usage flags).
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BufferDescription {
    pub size: u64,
    pub usage_flags: vk::BufferUsageFlags,
}

impl BufferDescription {
    pub fn new_simple(size: u64, usage_flags: vk::BufferUsageFlags) -> Self {
        BufferDescription { size, usage_flags }
    }
}

/// Contains a description for a vulkan buffer.
///
/// This only contains static information relevant to vulkan (i.e. range or format, however not the
/// source buffer as buffer views with different sources may have the same description).
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BufferViewDescription {
    pub format: &'static Format,
    pub range: BufferRange,
}

impl BufferViewDescription {
    pub fn new_simple(range: BufferRange, format: &'static Format) -> Self {
        Self { range, format }
    }
}

pub struct BufferInstanceData {
    handle: vk::Buffer,
    mapped_memory: Option<MappedMemory>,
}

impl BufferInstanceData {
    pub fn new(handle: vk::Buffer, mapped_memory: Option<MappedMemory>) -> Self {
        Self {
            handle,
            mapped_memory,
        }
    }

    pub fn get_mapped_memory(&self) -> Option<&MappedMemory> {
        self.mapped_memory.as_ref()
    }

    pub unsafe fn get_handle(&self) -> vk::Buffer {
        self.handle
    }
}

pub struct BufferViewInstanceData {
    handle: vk::BufferView,
    source_buffer: BufferId,
}

impl BufferViewInstanceData {
    pub fn new(handle: vk::BufferView, source_buffer: BufferId) -> Self {
        Self {
            handle,
            source_buffer,
        }
    }

    pub fn get_source_buffer(&self) -> BufferId {
        self.source_buffer
    }

    pub unsafe fn get_handle(&self) -> vk::BufferView {
        self.handle
    }
}