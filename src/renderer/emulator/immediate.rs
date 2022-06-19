use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::Arc;
use ash::vk;

use crate::objects::id::BufferId;
use crate::objects::sync::SemaphoreOp;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};
use crate::vk::objects::buffer::Buffer;
use crate::util::alloc::next_aligned;

use crate::prelude::*;

pub(super) struct ImmediateBuffer {
    device: Arc<DeviceContext>,
    current_buffer: Buffer,
    old_buffers: Vec<Buffer>,
}

impl ImmediateBuffer {
    const MIN_BUFFER_SIZE: vk::DeviceSize = 2u64.pow(24); // 16MB
    const OVER_ALLOCATION: u8 = 77; // 30%

    pub(super) fn generate_copy_commands(&self, cmd: vk::CommandBuffer) {
        self.current_buffer.generate_copy_commands(cmd);
        for old_buffer in &self.old_buffers {
            old_buffer.generate_copy_commands(cmd);
        }
    }

    pub(super) fn reset(&mut self) {
        self.current_buffer.reset();
        self.old_buffers.clear();
    }

    pub(super) fn allocate<T: ToBytes>(&mut self, data: &T, alignment: vk::DeviceSize) -> (vk::Buffer, vk::DeviceSize) {
        let data = data.as_bytes();

        if let Some(info) = self.current_buffer.allocate(data, alignment) {
            info
        } else {
            let usage = self.get_current_usage();
            let alloc_size = usage + (usage * (Self::OVER_ALLOCATION as u64) / (u8::MAX as u64));
            let alloc_size = std::cmp::max(data.len() as u64, alloc_size);

            let new_buffer = Buffer::new(self.device.clone(), alloc_size);
            self.old_buffers.push(std::mem::replace(&mut self.current_buffer, new_buffer));

            self.current_buffer.allocate(data, alignment).unwrap()
        }
    }

    fn get_current_usage(&self) -> vk::DeviceSize {
        let mut usage = self.current_buffer.get_current_used_bytes();
        for old_buffer in &self.old_buffers {
            usage += old_buffer.get_current_used_bytes();
        }

        usage
    }
}

struct Buffer {
    device: Arc<DeviceContext>,

    main_buffer: vk::Buffer,
    mapped_memory: NonNull<u8>,
    size: vk::DeviceSize,
    current_offset: vk::DeviceSize,

    main_allocation: Option<Allocation>,
    staging: Option<(vk::Buffer, Allocation)>,
}

impl Buffer {
    fn new(device: Arc<DeviceContext>, size: vk::DeviceSize) -> Self {
        let (main_buffer, main_allocation) = Self::create_main_buffer(&device, size);

        let (staging, mapped_memory) = if let Some(mapped) = main_allocation.mapped_ptr() {
            (None, mapped.into())
        } else {
            let (staging_buffer, staging_allocation) = Self::create_staging_buffer(&device, size);
            let mapped = staging_allocation.mapped_ptr().unwrap();
            (Some((staging_buffer, staging_allocation)), mapped.into())
        };

        Self {
            device,
            main_buffer,
            mapped_memory,
            size,
            current_offset: 0,
            main_allocation: Some(main_allocation),
            staging
        }
    }

    fn generate_copy_commands(&self, cmd: vk::CommandBuffer) {
        if let Some((staging_buffer, _)) = &self.staging {
            if self.current_offset != 0 {
                unsafe {
                    self.device.vk().cmd_copy_buffer(
                        cmd,
                        *staging_buffer,
                        self.main_buffer,
                        &[vk::BufferCopy {
                            src_offset: 0,
                            dst_offset: 0,
                            size: self.current_offset
                        }]
                    )
                }
            }
        }
    }

    fn reset(&mut self) {
        self.current_offset = 0;
    }

    fn allocate(&mut self, bytes: &[u8], alignment: vk::DeviceSize) -> Option<(vk::Buffer, vk::DeviceSize)> {
        let aligned = next_aligned(self.current_offset, alignment);
        if aligned + bytes.len() > self.size {
            return None;
        }

        self.current_offset = aligned + bytes.len();

        let start = aligned as usize;
        let end = self.current_offset as usize;
        let dst = &mut unsafe { std::slice::from_raw_parts_mut(self.mapped_memory.as_ptr(), self.size as usize) }[start..end];

        dst.copy_from_slice(bytes);

        Some((self.main_buffer, aligned))
    }

    fn get_current_used_bytes(&self) -> vk::DeviceSize {
        self.current_offset
    }

    fn create_main_buffer(device: &DeviceContext, size: vk::DeviceSize) -> (vk::Buffer, Allocation) {
        let info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device.vk().create_buffer(&info, None)
        }.unwrap_or_else(|err| {
            log::error!("Failed to create main buffer {:?}", err);
            panic!()
        });

        let allocation = device.get_allocator().allocate_buffer_memory(buffer, &AllocationStrategy::AutoGpuOnly).unwrap_or_else(|err| {
            log::error!("Failed to allocate main buffer memory {:?}", err);
            unsafe { device.vk().destroy_buffer(buffer, None) };
            panic!()
        });

        if let Err(err) = unsafe {
            device.vk().bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
        } {
            log::error!("Failed to bind buffer memory {:?}", err);
            unsafe { device.vk().destroy_buffer(buffer, None) };
            device.get_allocator().free(allocation);
            panic!();
        }

        (buffer, allocation)
    }

    fn create_staging_buffer(device: &DeviceContext, size: vk::DeviceSize) -> (vk::Buffer, Allocation) {
        let info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device.vk().create_buffer(&info, None)
        }.unwrap_or_else(|err| {
            log::error!("Failed to create staging buffer {:?}", err);
            panic!();
        });

        let allocation = device.get_allocator().allocate_buffer_memory(buffer, &AllocationStrategy::AutoGpuCpu).unwrap_or_else(|err| {
            log::error!("Failed to allocate staging memory {:?}", err);
            unsafe { device.vk().destroy_buffer(buffer, None) };
            panic!();
        });

        if let Err(err) = unsafe {
            device.vk().bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
        } {
            log::error!("Failed to bind staging memory {:?}", err);
            unsafe { device.vk().destroy_buffer(buffer, None) };
            device.get_allocator().free(allocation);
            panic!();
        }

        (buffer, allocation)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_buffer(self.main_buffer, None);
        }
        self.device.get_allocator().free(self.main_allocation.take().unwrap());
        if let Some((buffer, alloc)) = self.staging.take() {
            unsafe {
                self.device.vk().destroy_buffer(buffer, None)
            };
            self.device.get_allocator().free(alloc);
        }
    }
}