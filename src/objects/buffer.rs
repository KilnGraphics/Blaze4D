use ash::vk;

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
pub struct BufferMeta {

}

#[non_exhaustive]
pub struct BufferCreateDesc {
    pub size: u64,
    pub usage_flags: vk::BufferUsageFlags,
}

impl BufferCreateDesc {
    pub fn new_simple(size: u64, usage_flags: vk::BufferUsageFlags) -> Self {
        BufferCreateDesc { size, usage_flags }
    }
}

#[non_exhaustive]
pub struct BufferViewCreateDesc {
    pub format: &'static crate::objects::Format,
    pub range: BufferRange,
}

impl BufferViewCreateDesc {
    pub fn new_simple(range: BufferRange, format: &'static crate::objects::Format) -> Self {
        Self { range, format }
    }
}