use std::ptr::NonNull;
use std::sync::Arc;

use ash::vk;

use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

use crate::prelude::*;

pub(super) struct DescriptorPool {
    device: Arc<DeviceContext>,
    uniform_buffer_pool: UniformBufferPool,
}

impl DescriptorPool {
    pub(super) fn new(device: Arc<DeviceContext>) -> Self {
        let uniform_buffer_pool = UniformBufferPool::new(&device);
        Self {
            device,
            uniform_buffer_pool,
        }
    }

    pub(super) fn allocate_uniform(&mut self, data: &[u8]) -> (vk::Buffer, vk::DeviceSize) {
        self.uniform_buffer_pool.allocate_write(data)
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        self.uniform_buffer_pool.destroy(&self.device);
    }
}

struct UniformBufferPool {
    buffer_allocation: Option<Allocation>,
    buffer: vk::Buffer,
    buffer_size: usize,
    current_offset: usize,
    mapped_ptr: NonNull<u8>,
}

impl UniformBufferPool {
    fn new(device: &DeviceContext) -> Self {
        let target_size = 2usize.pow(25); // ~32MB
        let info = vk::BufferCreateInfo::builder()
            .size(target_size as vk::DeviceSize)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device.vk().create_buffer(&info, None)
        }.unwrap();

        let allocation = device.get_allocator().allocate_buffer_memory(buffer, &AllocationStrategy::AutoGpuCpu).unwrap();

        unsafe {
            device.vk().bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
        }.unwrap();

        let ptr = allocation.mapped_ptr().unwrap().cast();

        Self {
            buffer_allocation: Some(allocation),
            buffer,
            buffer_size: target_size,
            current_offset: 0,
            mapped_ptr: ptr,
        }
    }

    fn allocate_write(&mut self, data: &[u8]) -> (vk::Buffer, vk::DeviceSize) {
        // We just allocate a new slot hoping that it isn't in use anymore. This is not that dangerous right now since we have a 32MB buffer which equates to roughly 100k slots
        // but it for sure can't become a permanent solution.
        let src = data;
        if src.len() > 1024 { // Just a sanity check all of our uniforms currently are < 256
            panic!("Wtf are you doing???");
        }

        // We align to 256bytes because that was the highest in the gpuinfo database (Yes this is entire module is very much a TODO)
        let add = 256 - (self.current_offset % 256);
        let mut base_offset = self.current_offset + add;

        if base_offset + src.len() > self.buffer_size {
            base_offset = 0;
        }
        let base_offset = base_offset;

        self.current_offset = base_offset + src.len();

        let dst = unsafe {
            std::slice::from_raw_parts_mut(self.mapped_ptr.as_ptr().offset(base_offset as isize), src.len())
        };
        dst.copy_from_slice(src);

        (self.buffer, base_offset as vk::DeviceSize)
    }

    fn destroy(&mut self, device: &DeviceContext) {
        unsafe {
            device.vk().destroy_buffer(self.buffer, None)
        };
        device.get_allocator().free(self.buffer_allocation.take().unwrap());
    }
}

unsafe impl Send for UniformBufferPool {
}