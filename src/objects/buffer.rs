use std::sync::Arc;
use ash::vk;
use crate::objects::{id, SynchronizationGroup};

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

#[derive(Copy, Clone, Debug)]
pub struct BufferRange {
    pub offset: u64,
    pub length: u64,
}

#[non_exhaustive]
pub struct BufferDescription {
    pub size: u64,
    pub usage_flags: vk::BufferUsageFlags,
}

impl BufferDescription {
    pub fn new_simple(size: u64, usage_flags: vk::BufferUsageFlags) -> Self {
        BufferDescription { size, usage_flags }
    }
}

/// Contains information about a vulkan buffer object
pub struct BufferInfo {
    desc: BufferDescription,
    group: SynchronizationGroup,
}

impl BufferInfo {
    pub fn new(desc: BufferDescription, group: SynchronizationGroup) -> Self {
        Self {
            desc,
            group
        }
    }

    pub fn get_description(&self) -> &BufferDescription {
        &self.desc
    }

    pub fn get_synchronization_group(&self) -> &SynchronizationGroup {
        &self.group
    }
}

#[non_exhaustive]
pub struct BufferViewDescription {
    pub format: &'static crate::objects::Format,
    pub range: BufferRange,
}

impl BufferViewDescription {
    pub fn new_simple(range: BufferRange, format: &'static crate::objects::Format) -> Self {
        Self { range, format }
    }
}

/// Contains information about a vulkan buffer view object
pub struct BufferViewInfo {
    desc: BufferViewDescription,
    source_buffer_id: id::BufferId,
    source_buffer_info: Arc<BufferInfo>,
}

impl BufferViewInfo {
    pub fn new(desc: BufferViewDescription, source_buffer_id: id::BufferId, source_buffer_info: Arc<BufferInfo>) -> Self {
        Self {
            desc,
            source_buffer_id,
            source_buffer_info,
        }
    }

    pub fn get_description(&self) -> &BufferViewDescription {
        &self.desc
    }

    pub fn get_source_buffer_id(&self) -> id::BufferId {
        self.source_buffer_id
    }

    pub fn get_source_buffer_info(&self) -> &BufferInfo {
        self.source_buffer_info.as_ref()
    }

    /// Utility function to get the synchronization group for this buffer view.
    /// Is equivalent to calling `get_source_buffer_info().get_synchronization_group()`.
    pub fn get_synchronization_group(&self) -> &SynchronizationGroup {
        &self.source_buffer_info.get_synchronization_group()
    }
}