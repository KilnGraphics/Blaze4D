use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use ash::vk;
use ash::vk::Handle;
use crate::objects::id::{BufferId, ObjectId};

use crate::vk::objects::allocator::MappedMemory;
use crate::vk::objects::Format;

use crate::prelude::*;

#[derive(Copy, Clone)]
pub struct Buffer {
    id: BufferId,
    handle: vk::Buffer,
}

impl Buffer {
    pub fn new(handle: vk::Buffer) -> Self {
        Self {
            id: BufferId::new(),
            handle,
        }
    }

    pub fn from_raw(id: BufferId, handle: vk::Buffer) -> Self {
        Self {
            id,
            handle
        }
    }

    pub fn get_id(&self) -> BufferId {
        self.id
    }

    pub fn get_handle(&self) -> vk::Buffer {
        self.handle
    }
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for Buffer {
}

impl PartialOrd for Buffer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Buffer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Hash for Buffer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("vkBuffer(UUID: {:#016X}, Handle: {:#016X})", self.id.get_raw(), self.handle.as_raw()))
    }
}

impl From<Buffer> for BufferId {
    fn from(buffer: Buffer) -> Self {
        buffer.get_id()
    }
}

impl From<Buffer> for UUID {
    fn from(buffer: Buffer) -> UUID {
        buffer.get_id().as_uuid()
    }
}


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
    source_buffer: crate::vk::objects::types::BufferId,
}

impl BufferViewInstanceData {
    pub fn new(handle: vk::BufferView, source_buffer: crate::vk::objects::types::BufferId) -> Self {
        Self {
            handle,
            source_buffer,
        }
    }

    pub fn get_source_buffer(&self) -> crate::vk::objects::types::BufferId {
        self.source_buffer
    }

    pub unsafe fn get_handle(&self) -> vk::BufferView {
        self.handle
    }
}