use std::mem::ManuallyDrop;
use std::sync::Mutex;

use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{AllocationCreateDesc, AllocatorCreateDesc};

use crate::device::DeviceContext;

#[derive(Debug)]
pub enum AllocationError {
    GpuAllocator(gpu_allocator::AllocationError),
}

impl From<gpu_allocator::AllocationError> for AllocationError {
    fn from(err: gpu_allocator::AllocationError) -> Self {
        Self::GpuAllocator(err)
    }
}

pub enum AllocationStrategy {
    /// Automatically select memory that is only used by the gpu
    AutoGpuOnly,

    /// Automatically select memory that is used by both gpu and cpu
    AutoGpuCpu,
}

/// Manages memory allocation for vulkan object
///
/// Currently just uses the [`gpu_allocator::vulkan::Allocator`] struct.
pub(super) struct Allocator {
    device: DeviceContext,

    // We need to ensure the allocator is dropped before the instance and device are
    allocator: ManuallyDrop<Mutex<gpu_allocator::vulkan::Allocator>>
}

impl Allocator {
    pub fn new(device: DeviceContext) -> Self {
        let allocator = gpu_allocator::vulkan::Allocator::new(&AllocatorCreateDesc{
            instance: device.get_instance().vk().clone(),
            device: device.vk().clone(),
            physical_device: device.get_physical_device().clone(),
            debug_settings: Default::default(),
            buffer_device_address: false
        }).unwrap();

        Self {
            device,
            allocator: ManuallyDrop::new(Mutex::new(allocator)),
        }
    }

    pub fn allocate_buffer_memory(&self, buffer: vk::Buffer, strategy: &AllocationStrategy) -> Result<Allocation, AllocationError> {
        let location = match strategy {
            AllocationStrategy::AutoGpuOnly => MemoryLocation::GpuOnly,
            AllocationStrategy::AutoGpuCpu => MemoryLocation::CpuToGpu,
        };

        let requirements = unsafe {
            self.device.vk().get_buffer_memory_requirements(buffer)
        };

        let alloc_desc = AllocationCreateDesc{
            name: "",
            requirements,
            location,
            linear: true
        };

        let alloc = self.allocator.lock().unwrap().allocate(&alloc_desc)?;

        Ok(Allocation::new(alloc))
    }

    pub fn allocate_image_memory(&self, image: vk::Image, strategy: &AllocationStrategy) -> Result<Allocation, AllocationError> {
        let location = match strategy {
            AllocationStrategy::AutoGpuOnly => MemoryLocation::GpuOnly,
            AllocationStrategy::AutoGpuCpu => MemoryLocation::CpuToGpu,
        };

        let requirements = unsafe {
            self.device.vk().get_image_memory_requirements(image)
        };

        let alloc_desc = AllocationCreateDesc{
            name: "",
            requirements,
            location,
            // If image is accessed by the cpu it has to be linear
            linear: location == MemoryLocation::CpuToGpu,
        };

        let alloc = self.allocator.lock().unwrap().allocate(&alloc_desc)?;

        Ok(Allocation::new(alloc))
    }

    pub fn free(&self, allocation: Allocation) {
        self.allocator.lock().unwrap().free(allocation.alloc).unwrap()
    }
}

impl Drop for Allocator {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.allocator) };
    }
}

pub struct Allocation {
    alloc: gpu_allocator::vulkan::Allocation,
}

impl Allocation {
    fn new(alloc: gpu_allocator::vulkan::Allocation) -> Self {
        Self {
            alloc,
        }
    }

    pub fn memory(&self) -> vk::DeviceMemory {
        unsafe { self.alloc.memory() }
    }

    pub fn offset(&self) -> vk::DeviceSize {
        self.alloc.offset()
    }
}