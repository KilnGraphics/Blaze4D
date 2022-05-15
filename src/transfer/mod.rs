mod resource_state;
mod worker;

use std::collections::{VecDeque};
use std::sync::{Arc, Condvar, Mutex};

use ash::vk;

use crate::prelude::*;
use crate::device::device::VkQueue;
use crate::vk::DeviceEnvironment;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};
use crate::vk::objects::buffer::{Buffer, BufferId};

use worker::*;
use crate::vk::objects::image::{Image, ImageId};
use crate::vk::objects::semaphore::{SemaphoreOp, SemaphoreOps};

#[derive(Clone)]
pub struct Transfer(Arc<Share>);

impl Transfer {
    pub fn new(device: DeviceEnvironment) -> Self {
        let share = Arc::new(Share::new(device.clone()));

        let queue = device.get_device().get_transfer_queue();
        let share2 = share.clone();
        std::thread::spawn(move || {
            run_worker(share2, device, queue);
        });

        Self(share)
    }

    pub fn get_transfer_queue_family(&self) -> u32 {
        self.0.queue_family
    }

    pub fn wait_for(&self, id: u64) {
        let info = vk::SemaphoreWaitInfo::builder()
            .semaphores(std::slice::from_ref(&self.0.semaphore))
            .values(std::slice::from_ref(&id));

        unsafe {
            self.0.device.vk().wait_semaphores(&info, u64::MAX)
        }.unwrap();
    }

    pub fn get_wait_op(&self, id: u64) -> SemaphoreOp {
        SemaphoreOp::new_timeline(self.0.semaphore, id)
    }

    /// Makes a buffer available for transfer operations.
    ///
    /// If the `src_queue` parameter differs from the transfer queue a queue family acquire
    /// operation will be generated for the specified buffer ranges. It is the callers responsibility
    /// to ensure a matching queue family release operation is submitted on the source queue.
    pub fn make_buffer_available(&self, op: BufferAvailabilityOp) {
        self.push_task(TaskInfo::BufferAcquire(op));
    }

    /// Revokes availability of a buffer from transfer operations.
    ///
    /// If the `dst_queue` parameter differs from the transfer queue a queue family release
    /// operation will be generated for the specified buffer ranges. It is the callers responsibility
    /// to ensure a matching queue family acquire operation is submitted on the destination queue.
    ///
    /// The returned value is a transfer id which can be used to wait on the host or as a semaphore
    /// to wait for completion of the operation.
    pub fn release_buffer(&self, op: BufferAvailabilityOp) -> u64 {
        self.push_task(TaskInfo::BufferRelease(op))
    }

    pub fn make_image_available(&self, op: ImageAvailabilityOp) {
        self.push_task(TaskInfo::ImageAcquire(op));
    }

    pub fn release_image(&self, op: ImageAvailabilityOp) -> u64 {
        self.push_task(TaskInfo::ImageRelease(op))
    }

    pub fn request_staging_memory(&self, capacity: usize) -> StagingMemory {
        let info = vk::BufferCreateInfo::builder()
            .size(capacity as vk::DeviceSize)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { self.0.device.vk().create_buffer(&info, None) }.unwrap();

        let allocation = self.0.device.get_allocator().allocate_buffer_memory(buffer, &AllocationStrategy::AutoGpuCpu).unwrap();

        unsafe {
            self.0.device.vk().bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
        }.unwrap();

        let memory = unsafe {
            std::slice::from_raw_parts_mut(allocation.mapped_ptr().unwrap().as_ptr() as *mut u8, capacity)
        };

        let buffer = Buffer::new(buffer);

        self.push_task(TaskInfo::AcquireStagingMemory(buffer));

        StagingMemory {
            transfer: self,
            memory,
            last_transfer: 0,
            buffer,
            buffer_offset: 0,
            allocation: Some(allocation),
        }
    }

    pub fn flush(&self) -> u64 {
        let mut guard = self.0.channel.lock().unwrap();

        let id = guard.current_task_id;

        guard.task_queue.push_back(Task{ id, info: TaskInfo::Flush });
        drop(guard);

        self.0.condvar.notify_one();

        id
    }

    fn push_task(&self, task: TaskInfo) -> u64 {
        let mut guard = self.0.channel.lock().unwrap();

        guard.current_task_id += 1;
        let id = guard.current_task_id;

        guard.task_queue.push_back(Task{ id, info: task });
        drop(guard);

        self.0.condvar.notify_one();

        id
    }
}

pub struct StagingMemory<'a> {
    transfer: &'a Transfer,
    memory: &'a mut [u8],
    last_transfer: u64,
    buffer: Buffer,
    buffer_offset: vk::DeviceSize,
    allocation: Option<Allocation>,
}

impl<'a> StagingMemory<'a> {
    /// Returns a slice to the staging memory range
    pub fn get_memory(&mut self) -> &mut [u8] {
        &mut self.memory
    }

    /// Writes the data stored in the slice to the memory and returns the number of bytes written.
    /// If the data does not fit into the available memory range [`None`] is returned.
    pub fn write<T: Copy>(&mut self, data: &[T]) -> Option<usize> {
        self.write_offset(data, 0)
    }

    /// Writes the data stored in the slice to the memory at the specified offset and returns the
    /// number of bytes written.
    /// If the data does not fit into the available memory range [`None`] is returned.
    pub fn write_offset<T: Copy>(&mut self, data: &[T], offset: usize) -> Option<usize> {
        let byte_count = data.len() * std::mem::size_of::<T>();
        if (offset + byte_count) < self.memory.len() {
            return None;
        }

        let src = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, byte_count)
        };
        let dst = &mut self.memory[offset..byte_count];
        dst.copy_from_slice(src);

        Some(byte_count)
    }

    pub fn read<T: Copy>(&self, data: &mut [T]) -> Result<(), ()> {
        self.read_offset(data, 0)
    }

    pub fn read_offset<T: Copy>(&self, data: &mut [T], offset: usize) -> Result<(), ()> {
        let byte_count = data.len() * std::mem::size_of::<T>();
        if (offset + byte_count) < self.memory.len() {
            return Err(());
        }

        let src = &self.memory[offset..byte_count];
        let dst = unsafe {
            std::slice::from_raw_parts_mut(data.as_ptr() as *mut u8, byte_count)
        };
        dst.copy_from_slice(src);

        Ok(())
    }

    pub fn copy_to_buffer<T: Into<BufferId>>(&mut self, dst_buffer: T, mut ranges: BufferTransferRanges) -> u64 {
        ranges.add_src_offset(self.buffer_offset);
        let task = TaskInfo::BufferTransfer(BufferTransfer {
            src_buffer: self.buffer.into(),
            dst_buffer: dst_buffer.into(),
            ranges
        });
        let id = self.transfer.push_task(task);
        self.last_transfer = id;
        id
    }

    pub fn copy_from_buffer<T: Into<BufferId>>(&mut self, src_buffer: T, mut ranges: BufferTransferRanges) -> u64 {
        ranges.add_dst_offset(self.buffer_offset);
        let task = TaskInfo::BufferTransfer(BufferTransfer {
            src_buffer: src_buffer.into(),
            dst_buffer: self.buffer.into(),
            ranges
        });
        let id = self.transfer.push_task(task);
        self.last_transfer = id;
        id
    }

    pub fn copy_to_image<T: Into<ImageId>>(&mut self, dst_image: T, mut ranges: BufferImageTransferRanges) -> u64 {
        ranges.add_buffer_offset(self.buffer_offset);
        let task = TaskInfo::BufferToImageTransfer(BufferToImageTransfer {
            src_buffer: self.buffer.get_id(),
            dst_image: dst_image.into(),
            ranges
        });
        let id = self.transfer.push_task(task);
        self.last_transfer = id;
        id
    }
}

impl<'a> Drop for StagingMemory<'a> {
    fn drop(&mut self) {
        self.transfer.push_task(TaskInfo::FreeStagingMemory(self.buffer, self.allocation.take().unwrap()));
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BufferAvailabilityOp {
    buffer: Buffer,
    queue: u32,
    semaphore_ops: SemaphoreOps,
}

impl BufferAvailabilityOp {
    pub fn new(buffer: Buffer, queue: u32, semaphore_ops: SemaphoreOps) -> Self {
        Self {
            buffer,
            queue,
            semaphore_ops,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BufferTransferRange {
    pub src_offset: vk::DeviceSize,
    pub dst_offset: vk::DeviceSize,
    pub size: vk::DeviceSize,
}

impl BufferTransferRange {
    pub fn new(src_offset: vk::DeviceSize, dst_offset: vk::DeviceSize, size: vk::DeviceSize) -> Self {
        Self {
            src_offset,
            dst_offset,
            size
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BufferTransferRanges {
    One(BufferTransferRange),
    Multiple(Box<[BufferTransferRange]>),
}

impl BufferTransferRanges {
    pub fn new_single(src_offset: vk::DeviceSize, dst_offset: vk::DeviceSize, size: vk::DeviceSize) -> Self {
        Self::One(BufferTransferRange::new(src_offset, dst_offset, size))
    }

    pub fn add_src_offset(&mut self, src_offset: vk::DeviceSize) {
        match self {
            BufferTransferRanges::One(range) => range.src_offset += src_offset,
            BufferTransferRanges::Multiple(ranges) => {
                for range in ranges.as_mut() {
                    range.src_offset += src_offset;
                }
            }
        }
    }

    pub fn add_dst_offset(&mut self, dst_offset: vk::DeviceSize) {
        match self {
            BufferTransferRanges::One(range) => range.dst_offset += dst_offset,
            BufferTransferRanges::Multiple(ranges) => {
                for range in ranges.as_mut() {
                    range.dst_offset += dst_offset;
                }
            }
        }
    }

    pub fn as_slice(&self) -> &[BufferTransferRange] {
        match self {
            BufferTransferRanges::One(range) => std::slice::from_ref(range),
            BufferTransferRanges::Multiple(ranges) => ranges.as_ref(),
        }
    }
}

#[derive(Debug)]
pub struct BufferTransfer {
    pub src_buffer: BufferId,
    pub dst_buffer: BufferId,
    pub ranges: BufferTransferRanges,
}

impl BufferTransfer {
    pub fn new_single_range<S: Into<BufferId>, D: Into<BufferId>>(
        src_buffer: S,
        src_offset: vk::DeviceSize,
        dst_buffer: D,
        dst_offset: vk::DeviceSize,
        size: vk::DeviceSize
    ) -> Self {
        Self {
            src_buffer: src_buffer.into(),
            dst_buffer: dst_buffer.into(),
            ranges: BufferTransferRanges::new_single(src_offset, dst_offset, size)
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ImageAvailabilityOp {
    image: Image,
    aspect_mask: vk::ImageAspectFlags,
    queue: u32,
    layout: vk::ImageLayout,
    semaphore_ops: SemaphoreOps,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BufferImageTransferRange {
    pub buffer_offset: vk::DeviceSize,
    pub buffer_row_length: u32,
    pub buffer_image_height: u32,
    pub image_aspect_mask: vk::ImageAspectFlags,
    pub image_mip_level: u32,
    pub image_base_array_layer: u32,
    pub image_layer_count: u32,
    pub image_offset: Vec3i32,
    pub image_extent: Vec3u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BufferImageTransferRanges {
    One(BufferImageTransferRange),
    Multiple(Box<[BufferImageTransferRange]>),
}

impl BufferImageTransferRanges {
    pub fn as_slice(&self) -> &[BufferImageTransferRange] {
        match self {
            Self::One(range) => std::slice::from_ref(range),
            Self::Multiple(ranges) => ranges.as_ref(),
        }
    }

    pub fn add_buffer_offset(&mut self, offset: vk::DeviceSize) {
        match self {
            BufferImageTransferRanges::One(range) => range.buffer_offset += offset,
            BufferImageTransferRanges::Multiple(ranges) => {
                for range in ranges.as_mut() {
                    range.buffer_offset += offset;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct BufferToImageTransfer {
    src_buffer: BufferId,
    dst_image: ImageId,
    ranges: BufferImageTransferRanges,
}

#[derive(Debug)]
pub struct ImageToBufferTransfer {
    src_image: ImageId,
    dst_buffer: BufferId,
    ranges: BufferImageTransferRanges,
}

#[cfg(test)]
mod tests {
    use crate::vk::test::make_headless_instance_device;
    use super::*;

    fn create_test_buffer(device: &DeviceEnvironment, size: usize) -> Buffer {
        let info = vk::BufferCreateInfo::builder()
            .size(size as vk::DeviceSize)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device.vk().create_buffer(&info, None)
        }.unwrap();

        let allocation = device.get_allocator().allocate_buffer_memory(buffer, &AllocationStrategy::AutoGpuOnly).unwrap();

        unsafe {
            device.vk().bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
        }.unwrap();

        Buffer::new(buffer)
    }

    #[test]
    fn test_buffer_copy() {
        env_logger::init();

        let (_, device) = make_headless_instance_device();

        let buffer = create_test_buffer(&device, 1024);
        let transfer = Transfer::new(device);

        let data: Vec<_> = (0u32..16u32).collect();
        let byte_size = data.len() * std::mem::size_of::<u32>();

        let queue = transfer.get_transfer_queue_family();
        transfer.make_buffer_available(BufferAvailabilityOp::new(buffer, queue, SemaphoreOps::None));

        let mut write_mem = transfer.request_staging_memory(byte_size);
        write_mem.write(data.as_slice());
        write_mem.copy_to_buffer(buffer, BufferTransferRanges::new_single(0, 0, byte_size as vk::DeviceSize));

        let mut dst_data = Vec::new();
        dst_data.resize(data.len(), 0u32);

        let mut read_mem = transfer.request_staging_memory(byte_size);
        read_mem.copy_from_buffer(buffer, BufferTransferRanges::new_single(0, 0, byte_size as vk::DeviceSize));

        transfer.release_buffer(BufferAvailabilityOp::new(buffer, queue, SemaphoreOps::None));
        let id = transfer.flush();

        transfer.wait_for(id);
        read_mem.read(dst_data.as_mut_slice()).unwrap();

        assert_eq!(data, dst_data);
    }
}