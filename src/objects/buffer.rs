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
pub struct BufferCreateInfo {
    pub size: u64,
    pub usage_flags: vk::BufferUsageFlags,
}

impl BufferCreateInfo {
    pub fn new_simple(size: u64, usage_flags: vk::BufferUsageFlags) -> Self {
        BufferCreateInfo { size, usage_flags }
    }
}

#[non_exhaustive]
pub struct BufferViewCreateInfo {
    pub format: vk::Format,
    pub range: BufferRange,
}