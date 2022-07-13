use std::ffi::CString;
use std::fmt;
use std::ptr::NonNull;
use std::sync::Arc;

use ash::vk;

use crate::prelude::*;

mod vma;

pub struct Allocator {
    vma_allocator: vma::Allocator,

    debug: bool,
    functions: Arc<DeviceFunctions>,
}

impl Allocator {
    pub fn new(functions: Arc<DeviceFunctions>) -> Result<Self, vk::Result> {
        let vma_allocator = vma::Allocator::new(&functions, vma::AllocatorCreateFlags::empty())?;

        Ok(Self {
            vma_allocator,
            debug: true,
            functions
        })
    }

    /// Allocates vulkan memory for some requirements.
    ///
    /// Returns the allocation and a [`AllocationBindingInfo`] containing information necessary to
    /// bind and use the memory. If allocation fails [`None`] is returned.
    ///
    /// # Safety
    ///
    /// `requirements` must be a valid [`vk::MemoryRequirements`] instance.
    pub unsafe fn allocate_memory(&self, requirements: &vk::MemoryRequirements, host_access: HostAccess, name: &fmt::Arguments) -> Option<(Allocation, AllocationBindingInfo)> {
        let create_info = Self::make_default_info(host_access);
        let mut allocation_info = vma::AllocationInfo::default();
        match self.vma_allocator.allocate_memory(requirements, &create_info, Some(&mut allocation_info)) {
            Ok(allocation) => {
                if self.debug {
                    self.set_allocation_name(allocation, name);
                }
                let binding_info = AllocationBindingInfo::new(&allocation_info);
                Some((Allocation::new(allocation), binding_info))
            }
            Err(err) => {
                log::warn!("Failed to allocate vulkan memory for {:?}. {:?}", name, err);
                None
            }
        }
    }

    /// Allocates multiple pages of vulkan memory for some requirements.
    ///
    /// Returns the allocations and [`AllocationBindingInfo`] containing information necessary to
    /// bind and use the memory. If allocation fails [`None`] is returned.
    ///
    /// # Safety
    ///
    /// Every entry in `requirements` must be a valid [`vk::MemoryRequirements`] instance.
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

    /// Frees previously allocated memory.
    ///
    /// # Safety
    ///
    /// The allocation must have been previously allocated from this allocator and not yet freed.
    pub unsafe fn free_memory(&self, allocation: Allocation) {
        self.vma_allocator.free_memory(allocation.vma_allocation)
    }

    /// Frees multiple previously allocated memory pages.
    ///
    /// # Safety
    ///
    /// All allocations must have been previously allocated from this allocator and not yet freed.
    pub unsafe fn free_memory_pages(&self, allocations: &[Allocation]) {
        let mapped: Box<_> = allocations.iter().map(|a| a.vma_allocation).collect();
        self.vma_allocator.free_memory_pages(mapped.as_ref())
    }

    /// Creates a gpu only buffer and binds memory to it.
    ///
    /// If creation, allocation or binding fails [`None`] is returned.
    ///
    /// # Safety
    ///
    /// `create_info` must be a valid [`vk::BufferCreateInfo`] instance.
    pub unsafe fn create_gpu_buffer(&self, create_info: &vk::BufferCreateInfo, name: &fmt::Arguments) -> Option<(vk::Buffer, Allocation)> {
        let allocation_create_info = Self::make_default_info(HostAccess::None);
        match self.vma_allocator.create_buffer(create_info, &allocation_create_info, None) {
            Ok((buffer, allocation)) => {
                if self.debug {
                    self.set_allocation_name(allocation, name);
                }
                Some((buffer, Allocation::new(allocation)))
            },
            Err(err) => {
                log::warn!("Failed to create gpu vulkan buffer {:?}. {:?}", name, err);
                None
            }
        }
    }

    /// Creates a buffer and binds memory to it.
    ///
    /// The allocator may select host visible memory even if it was not requested. In that case a
    /// pointer to the mapped memory will always be returned.
    ///
    /// Returns the buffer, allocation and if host visible memory is selected a pointer to the
    /// mapped memory. If creation, allocation or binding fails [`None`] is returned.
    ///
    /// # Safety
    ///
    /// `create_info` must be a valid [`vk::BufferCreateInfo`] instance.
    pub unsafe fn create_buffer(&self, create_info: &vk::BufferCreateInfo, host_access: HostAccess, name: &fmt::Arguments) -> Option<(vk::Buffer, Allocation, Option<NonNull<u8>>)> {
        let allocation_create_info = Self::make_default_info(host_access);
        let mut allocation_info = vma::AllocationInfo::default();
        match self.vma_allocator.create_buffer(create_info, &allocation_create_info, Some(&mut allocation_info)) {
            Ok((buffer, allocation)) => {
                if self.debug {
                    self.set_allocation_name(allocation, name);
                }
                Some((buffer, Allocation::new(allocation), NonNull::new(allocation_info.p_mapped_data as *mut u8)))
            },
            Err(err) => {
                log::warn!("Failed to create vulkan buffer {:?}. {:?}", name, err);
                None
            }
        }
    }

    /// Creates a gpu only image and binds memory to it.
    ///
    /// If creation, allocation or binding fails [`None`] is returned.
    ///
    /// # Safety
    ///
    /// `create_info` must be a valid [`vk::ImageCreateInfo`] instance.
    pub unsafe fn create_gpu_image(&self, create_info: &vk::ImageCreateInfo, name: &fmt::Arguments) -> Option<(vk::Image, Allocation)> {
        let allocation_create_info = Self::make_default_info(HostAccess::None);
        match self.vma_allocator.create_image(create_info, &allocation_create_info, None) {
            Ok((image, allocation)) => {
                if self.debug {
                    self.set_allocation_name(allocation, name);
                }
                Some((image, Allocation::new(allocation)))
            },
            Err(err) => {
                log::warn!("Failed to create gpu vulkan image {:?}. {:?}", name, err);
                None
            }
        }
    }

    /// Creates a image and binds memory to it.
    ///
    /// The allocator may select host visible memory even if it was not requested. In that case a
    /// pointer to the mapped memory will always be returned.
    ///
    /// Returns the image, allocation and if host visible memory is selected a pointer to the
    /// mapped memory. If creation, allocation or binding fails [`None`] is returned.
    ///
    /// # Safety
    ///
    /// `create_info` must be a valid [`vk::ImageCreateInfo`] instance.
    pub unsafe fn create_image(&self, create_info: &vk::ImageCreateInfo, host_access: HostAccess, name: &fmt::Arguments) -> Option<(vk::Image, Allocation, Option<NonNull<u8>>)> {
        let allocation_create_info = Self::make_default_info(HostAccess::None);
        let mut allocation_info = vma::AllocationInfo::default();
        match self.vma_allocator.create_image(create_info, &allocation_create_info, Some(&mut allocation_info)) {
            Ok((image, allocation)) => {
                if self.debug {
                    self.set_allocation_name(allocation, name);
                }
                Some((image, Allocation::new(allocation), NonNull::new(allocation_info.p_mapped_data as *mut u8)))
            },
            Err(err) => {
                log::warn!("Failed to create vulkan image {:?}. {:?}", name, err);
                None
            }
        }
    }

    /// Destroys a previously created buffer and allocation
    ///
    /// # Safety
    ///
    /// `buffer` must be a valid [`vk::Buffer`] handle created on the same device that this
    /// allocator uses.
    /// `allocation` must have been previously allocated from this allocator and not yet freed.
    pub unsafe fn destroy_buffer(&self, buffer: vk::Buffer, allocation: Allocation) {
        self.vma_allocator.destroy_buffer(buffer, allocation.vma_allocation)
    }

    /// Destroys a previously created image and allocation
    ///
    /// # Safety
    ///
    /// `image` must be a valid [`vk::Image`] handle created on the same device that this
    /// allocator uses.
    /// `allocation` must have been previously allocated from this allocator and not yet freed.
    pub unsafe fn destroy_image(&self, image: vk::Image, allocation: Allocation) {
        self.vma_allocator.destroy_image(image, allocation.vma_allocation)
    }

    unsafe fn set_allocation_name(&self, allocation: vma::Allocation, name: &fmt::Arguments) {
        if let Some(str) = name.as_str() {
            self.vma_allocator.set_allocation_name(allocation, CString::new(str).unwrap().as_c_str())
        } else {
            self.vma_allocator.set_allocation_name(allocation, CString::new(name.to_string()).unwrap().as_c_str())
        }
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