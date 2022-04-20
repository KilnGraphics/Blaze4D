use ash::vk;
use crate::objects::SynchronizationGroup;

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
    pub format: &'static crate::objects::Format,
    pub range: BufferRange,
}

impl BufferViewDescription {
    pub fn new_simple(range: BufferRange, format: &'static crate::objects::Format) -> Self {
        Self { range, format }
    }
}

pub struct BufferInstanceData {
    handle: vk::Buffer,
    synchronization_group: SynchronizationGroup,
}

impl BufferInstanceData {
    pub fn new(handle: vk::Buffer, synchronization_group: SynchronizationGroup) -> Self {
        Self {
            handle,
            synchronization_group,
        }
    }

    pub fn get_synchronization_group(&self) -> &SynchronizationGroup {
        &self.synchronization_group
    }

    pub unsafe fn get_handle(&self) -> vk::Buffer {
        self.handle
    }
}

pub struct BufferViewInstanceData {
    handle: vk::BufferView,
    synchronization_group: SynchronizationGroup,
}

impl BufferViewInstanceData {
    pub fn new(handle: vk::BufferView, synchronization_group: SynchronizationGroup) -> Self {
        Self {
            handle,
            synchronization_group,
        }
    }

    pub fn get_synchronization_group(&self) -> &SynchronizationGroup {
        &self.synchronization_group
    }

    pub unsafe fn get_handle(&self) -> vk::BufferView {
        self.handle
    }
}