use std::ptr::NonNull;
use std::sync::Arc;

use ash::vk;
use ash::vk::{BufferCreateInfo};

use crate::prelude::*;

mod vma;

pub struct Allocator {
    vma_allocator: vma::Allocator,

    functions: Arc<DeviceFunctions>,
}

impl Allocator {
    pub fn new(functions: Arc<DeviceFunctions>) -> Result<Self, vk::Result> {
        let vma_allocator = vma::Allocator::new(&functions, vma::AllocatorCreateFlags::empty())?;

        Ok(Self {
            vma_allocator,
            functions
        })
    }

    pub unsafe fn allocate_memory(&self, requirements: &vk::MemoryRequirements, host_access: HostAccess) -> Option<(Allocation, AllocationBindingInfo)> {
        let create_info = Self::make_default_info(host_access);
        let mut allocation_info = vma::AllocationInfo::default();
        match self.vma_allocator.allocate_memory(requirements, &create_info, Some(&mut allocation_info)) {
            Ok(allocation) => {
                let binding_info = AllocationBindingInfo::new(&allocation_info);
                Some((Allocation::new(allocation), binding_info))
            }
            Err(err) => {
                log::warn!("Failed to allocate vulkan memory {:?}", err);
                None
            }
        }
    }

    pub unsafe fn allocate_memory_pages(&self, requirements: &[vk::MemoryRequirements], host_access: HostAccess) -> Option<Vec<(Allocation, AllocationBindingInfo)>> {
        let create_info: Box<_> = std::iter::repeat(Self::make_default_info(host_access).build()).take(requirements.len()).collect();
        let mut allocation_info = Vec::new();
        allocation_info.resize(requirements.len(), vma::AllocationInfo::default());
        match self.vma_allocator.allocate_memory_pages(requirements, create_info.as_ref(), Some(&mut allocation_info)) {
            Ok(allocations) => {
                debug_assert_eq!(allocations.len(), allocation_info.len());
                Some(allocations.into_iter().map(Allocation::new).zip(allocation_info.iter().map(AllocationBindingInfo::new)).collect())
            }
            Err(err) => {
                log::warn!("Failed to allocate vulkan memory pages {:?}", err);
                None
            }
        }
    }

    pub unsafe fn free_memory(&self, allocation: Allocation) {
        self.vma_allocator.free_memory(allocation.vma_allocation)
    }

    pub unsafe fn free_memory_pages(&self, allocations: &[Allocation]) {
        let mapped: Box<_> = allocations.iter().map(|a| a.vma_allocation).collect();
        self.vma_allocator.free_memory_pages(mapped.as_ref())
    }

    pub unsafe fn create_gpu_buffer(&self, create_info: &BufferCreateInfo) -> Option<(vk::Buffer, Allocation)> {
        let allocation_create_info = Self::make_default_info(HostAccess::None);
        match self.vma_allocator.create_buffer(create_info, &allocation_create_info, None) {
            Ok((buffer, allocation)) => Some((buffer, Allocation::new(allocation))),
            Err(err) => {
                log::warn!("Failed to create gpu vulkan buffer {:?}", err);
                None
            }
        }
    }

    pub unsafe fn create_buffer(&self, create_info: &BufferCreateInfo, host_access: HostAccess) -> Option<(vk::Buffer, Allocation, Option<NonNull<u8>>)> {
        let allocation_create_info = Self::make_default_info(host_access);
        let mut allocation_info = vma::AllocationInfo::default();
        match self.vma_allocator.create_buffer(create_info, &allocation_create_info, Some(&mut allocation_info)) {
            Ok((buffer, allocation)) => Some((buffer, Allocation::new(allocation), NonNull::new(allocation_info.p_mapped_data as *mut u8))),
            Err(err) => {
                log::warn!("Failed to create vulkan buffer {:?}", err);
                None
            }
        }
    }

    pub unsafe fn destroy_buffer(&self, buffer: vk::Buffer, allocation: Allocation) {
        self.vma_allocator.destroy_buffer(buffer, allocation.vma_allocation)
    }

    fn make_default_info<'a>(host_access: HostAccess) -> vma::AllocationCreateInfoBuilder<'a> {
        vma::AllocationCreateInfo::builder()
            .flags(host_access.to_vma_flags() | vma::AllocationCreateFlags::CREATE_MAPPED)
            .usage(vma::MemoryUsage::AUTO)
            .required_flags(vk::MemoryPropertyFlags::empty())
            .preferred_flags(vk::MemoryPropertyFlags::empty())
            .memory_type_bits(0)
            .priority(0.5f32)
    }
}

pub struct Allocation {
    vma_allocation: vma::Allocation,
}

impl Allocation {
    fn new(vma_allocation: vma::Allocation) -> Self {
        Self {
            vma_allocation
        }
    }
}

#[derive(Copy, Clone)]
pub struct AllocationBindingInfo {
    device_memory: vk::DeviceMemory,
    offset: vk::DeviceSize,
    size: vk::DeviceSize,
    mapped_data: Option<NonNull<u8>>,
}

impl AllocationBindingInfo {
    fn new(info: &vma::AllocationInfo) -> Self {
        Self {
            device_memory: info.device_memory,
            offset: info.offset,
            size: info.size,
            mapped_data: NonNull::new(info.p_mapped_data as *mut u8)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum HostAccess {
    /// Host access is not required or preferred
    None,

    /// Host will read or write randomly from the memory
    Random,

    /// Host will read or write randomly from the memory. The allocator can select a non host
    /// visible memory if necessary or better.
    RandomOptional,

    /// Host will only write sequentially to the memory.
    SequentialWrite,

    /// Host will only write sequentially to the memory. The allocator can select a non host
    /// visible memory if necessary or better.
    SequentialWriteOptional,
}

impl HostAccess {
    fn to_vma_flags(&self) -> vma::AllocationCreateFlags {
        match self {
            HostAccess::None => vma::AllocationCreateFlags::empty(),
            HostAccess::Random => vma::AllocationCreateFlags::HOST_ACCESS_RANDOM,
            HostAccess::RandomOptional => vma::AllocationCreateFlags::HOST_ACCESS_RANDOM | vma::AllocationCreateFlags::HOST_ACCESS_ALLOW_TRANSFER_INSTEAD,
            HostAccess::SequentialWrite => vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
            HostAccess::SequentialWriteOptional => vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE | vma::AllocationCreateFlags::HOST_ACCESS_ALLOW_TRANSFER_INSTEAD,
        }
    }
}