use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::Mutex;

use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{AllocationCreateDesc, AllocatorCreateDesc};

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
pub struct Allocator {
    device: ash::Device,
    allocator: Mutex<gpu_allocator::vulkan::Allocator>
}

impl Allocator {
    pub fn new(instance: ash::Instance, device: ash::Device, physical_device: vk::PhysicalDevice) -> Self {
        let allocator = gpu_allocator::vulkan::Allocator::new(&AllocatorCreateDesc{
            instance,
            device: device.clone(),
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: false
        }).unwrap();

        Self {
            device,
            allocator: Mutex::new(allocator),
        }
    }

    pub fn allocate_buffer_memory(&self, buffer: vk::Buffer, strategy: &AllocationStrategy) -> Result<Allocation, AllocationError> {
        let location = match strategy {
            AllocationStrategy::AutoGpuOnly => MemoryLocation::GpuOnly,
            AllocationStrategy::AutoGpuCpu => MemoryLocation::CpuToGpu,
        };

        let requirements = unsafe {
            self.device.get_buffer_memory_requirements(buffer)
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
            self.device.get_image_memory_requirements(image)
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

pub struct Allocation {
    alloc: gpu_allocator::vulkan::Allocation,
}

impl Allocation {
    fn new(alloc: gpu_allocator::vulkan::Allocation) -> Self {
        Self {
            alloc,
        }
    }

    pub fn mapped_ptr(&self) -> Option<std::ptr::NonNull<c_void>> {
        self.alloc.mapped_ptr()
    }

    pub fn memory(&self) -> vk::DeviceMemory {
        unsafe { self.alloc.memory() }
    }

    pub fn offset(&self) -> vk::DeviceSize {
        self.alloc.offset()
    }
}

pub struct MappedMemory {
    ptr: NonNull<c_void>,
    len: usize,
}

impl MappedMemory {
    pub unsafe fn new(ptr: NonNull<c_void>, len: usize) -> Self {
        if len == 0 {
            panic!("Length of mapped memory must be greater than 0.");
        }
        Self {
            ptr,
            len
        }
    }

    pub fn as_byte_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr() as *const u8, self.len) }
    }
}