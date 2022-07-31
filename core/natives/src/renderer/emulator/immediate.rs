use std::collections::VecDeque;
use std::panic::RefUnwindSafe;
use std::ptr::NonNull;
use std::sync::{Arc, Condvar, Mutex};

use ash::vk;
use crate::allocator::{Allocation, HostAccess};

use crate::util::alloc::next_aligned;

use crate::prelude::*;

pub(super) struct ImmediatePool {
    buffer_queue: Mutex<VecDeque<Box<ImmediateBuffer>>>,
    ready_condvar: Condvar,
}

impl ImmediatePool {
    pub(super) fn new(device: Arc<DeviceContext>) -> Self {
        let mut buffer_queue = VecDeque::with_capacity(2);
        for _ in 0..2 {
            buffer_queue.push_back(Box::new(ImmediateBuffer::new(device.clone())));
        }

        Self {
            buffer_queue: Mutex::new(buffer_queue),
            ready_condvar: Condvar::new(),
        }
    }

    pub(super) fn get_next_buffer(&self) -> Box<ImmediateBuffer> {
        let mut guard = self.buffer_queue.lock().unwrap_or_else(|_| {
            log::error!("Poisoned queue mutex in ImmediatePool::get_next_buffer");
            panic!()
        });
        loop {
            if let Some(next) = guard.pop_front() {
                return next;
            }

            let (new_guard, timeout) = self.ready_condvar.wait_timeout(guard, std::time::Duration::from_secs(1)).unwrap_or_else(|_| {
                log::error!("Poisoned queue mutex in ImmediatePool::get_next_buffer after waiting for condvar");
                panic!()
            });
            guard = new_guard;

            if timeout.timed_out() {
                log::warn!("1s timeout hit while waiting for new buffer in ImmediatePool::get_next_buffer");
            }
        }
    }

    pub(super) fn return_buffer(&self, mut buffer: Box<ImmediateBuffer>) {
        buffer.reset();

        let mut guard = self.buffer_queue.lock().unwrap_or_else(|_| {
            log::error!("Poisoned queue mutex in ImmediatePool::return_buffer");
            panic!()
        });

        guard.push_back(buffer);
        self.ready_condvar.notify_one();
    }
}

impl RefUnwindSafe for ImmediatePool {} // Condvar is not RefUnwindSafe

pub(super) struct ImmediateBuffer {
    device: Arc<DeviceContext>,
    current_buffer: Buffer,
    old_buffers: Vec<Buffer>,
}

impl ImmediateBuffer {
    const MIN_BUFFER_SIZE: vk::DeviceSize = 2u64.pow(24); // 16MB
    const OVER_ALLOCATION: u8 = 77; // 30%

    fn new(device: Arc<DeviceContext>) -> Self {
        let current_buffer = Buffer::new(device.clone(), Self::MIN_BUFFER_SIZE);

        Self {
            device,
            current_buffer,
            old_buffers: Vec::new(),
        }
    }

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

    pub(super) fn allocate(&mut self, data: &[u8], alignment: vk::DeviceSize) -> (vk::Buffer, vk::DeviceSize) {
        if let Some(info) = self.current_buffer.allocate(data, alignment) {
            info
        } else {
            let usage = self.get_current_usage();
            let alloc_size = usage + (usage * (Self::OVER_ALLOCATION as u64) / (u8::MAX as u64));
            let alloc_size = std::cmp::max(alloc_size, data.len() as u64);
            let alloc_size = std::cmp::max(alloc_size, Self::MIN_BUFFER_SIZE);

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

    main_allocation: Allocation,
    staging: Option<(vk::Buffer, Allocation)>,
}

impl Buffer {
    fn new(device: Arc<DeviceContext>, size: vk::DeviceSize) -> Self {
        let (main_buffer, main_allocation, main_mapped) = Self::create_main_buffer(&device, size);

        let (staging, mapped_memory) = if let Some(mapped) = main_mapped {
            log::info!("Immediate buffer uses mapped memory");
            (None, mapped)
        } else {
            log::info!("Immediate buffer uses staging memory");
            let (staging_buffer, staging_allocation, staging_mapped) = Self::create_staging_buffer(&device, size);
            (Some((staging_buffer, staging_allocation)), staging_mapped)
        };

        Self {
            device,
            main_buffer,
            mapped_memory,
            size,
            current_offset: 0,
            main_allocation,
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
        if aligned + (bytes.len() as vk::DeviceSize) > self.size {
            return None;
        }

        self.current_offset = aligned + (bytes.len() as vk::DeviceSize);

        let start = aligned as usize;
        let end = self.current_offset as usize;
        let dst = &mut unsafe { std::slice::from_raw_parts_mut(self.mapped_memory.as_ptr(), self.size as usize) }[start..end];

        dst.copy_from_slice(bytes);

        Some((self.main_buffer, aligned))
    }

    fn get_current_used_bytes(&self) -> vk::DeviceSize {
        self.current_offset
    }

    fn create_main_buffer(device: &DeviceContext, size: vk::DeviceSize) -> (vk::Buffer, Allocation, Option<NonNull<u8>>) {
        let info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let (buffer, allocation, mapped) = unsafe {
            device.get_allocator().create_buffer(&info, HostAccess::RandomOptional, &format_args!("ImmediateMainBuffer"))
        }.unwrap_or_else(|| {
            log::error!("Failed to create main buffer.");
            panic!()
        });

        (buffer, allocation, mapped)
    }

    fn create_staging_buffer(device: &DeviceContext, size: vk::DeviceSize) -> (vk::Buffer, Allocation, NonNull<u8>) {
        let info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let (buffer, allocation, mapped) = unsafe {
            device.get_allocator().create_buffer(&info, HostAccess::Random, &format_args!("ImmediateStagingBuffer"))
        }.unwrap_or_else(|| {
            log::error!("Failed to create staging buffer.");
            panic!()
        });

        (buffer, allocation, mapped.unwrap())
    }
}

unsafe impl Send for Buffer { // Needed because of NonNull<u8>
}

unsafe impl Sync for Buffer { // Needed because of NonNull<u8>
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.get_allocator().destroy_buffer(self.main_buffer, self.main_allocation);
            if let Some((buffer, alloc)) = self.staging.take() {
                self.device.get_allocator().destroy_buffer(buffer, alloc)
            }
        }
    }
}