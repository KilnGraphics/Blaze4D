use std::os::raw::c_char;
use std::ffi::c_void;
use ash::vk;

use crate::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct AllocatorCreateFlags(u32);

impl AllocatorCreateFlags {
    pub const EXTERNALLY_SYNCHRONIZED: AllocatorCreateFlags = AllocatorCreateFlags(0x00000001);
    pub const DEDICATED_ALLOCATION: AllocatorCreateFlags = AllocatorCreateFlags(0x00000002);
    pub const KHR_BIND_MEMORY2: AllocatorCreateFlags = AllocatorCreateFlags(0x00000004);
    pub const EXT_MEMORY_BUDGET: AllocatorCreateFlags = AllocatorCreateFlags(0x00000008);
    pub const AMD_DEVICE_COHERENT_MEMORY: AllocatorCreateFlags = AllocatorCreateFlags(0x00000010);
    pub const BUFFER_DEVICE_ADDRESS: AllocatorCreateFlags = AllocatorCreateFlags(0x00000020);
    pub const EXT_MEMORY_PRIORITY: AllocatorCreateFlags = AllocatorCreateFlags(0x00000040);
}
ash::vk_bitflags_wrapped!(AllocatorCreateFlags, u32);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct AllocationCreateFlags(u32);

impl AllocationCreateFlags {
    pub const DEDICATED_MEMORY: AllocationCreateFlags = AllocationCreateFlags(0x00000001);
    pub const NEVER_ALLOCATE: AllocationCreateFlags = AllocationCreateFlags(0x00000002);
    pub const CREATE_MAPPED: AllocationCreateFlags = AllocationCreateFlags(0x00000004);
    pub const UPPER_ADDRESS: AllocationCreateFlags = AllocationCreateFlags(0x00000040);
    pub const DONT_BIND: AllocationCreateFlags = AllocationCreateFlags(0x00000080);
    pub const WITHIN_BUDGET: AllocationCreateFlags = AllocationCreateFlags(0x00000100);
    pub const CAN_ALIAS: AllocationCreateFlags = AllocationCreateFlags(0x00000200);
    pub const HOST_ACCESS_SEQUENTIAL_WRITE: AllocationCreateFlags = AllocationCreateFlags(0x00000400);
    pub const HOST_ACCESS_RANDOM: AllocationCreateFlags = AllocationCreateFlags(0x00000800);
    pub const HOST_ACCESS_ALLOW_TRANSFER_INSTEAD: AllocationCreateFlags = AllocationCreateFlags(0x00001000);
    pub const STRATEGY_MIN_MEMORY: AllocationCreateFlags = AllocationCreateFlags(0x00010000);
    pub const STRATEGY_MIN_TIME: AllocationCreateFlags = AllocationCreateFlags(0x00020000);
    pub const STRATEGY_MIN_OFFSET: AllocationCreateFlags = AllocationCreateFlags(0x00040000);
}
ash::vk_bitflags_wrapped!(AllocationCreateFlags, u32);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MemoryUsage(u32);

impl MemoryUsage {
    pub const UNKNOWN: MemoryUsage = MemoryUsage(0);
    pub const GPU_LAZILY_ALLOCATED: MemoryUsage = MemoryUsage(6);
    pub const AUTO: MemoryUsage = MemoryUsage(7);
    pub const AUTO_PREFER_DEVICE: MemoryUsage = MemoryUsage(8);
    pub const AUTO_PREFER_HOST: MemoryUsage = MemoryUsage(9);

    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AllocationCreateInfo {
    pub flags: AllocationCreateFlags,
    pub usage: MemoryUsage,
    pub required_flags: vk::MemoryPropertyFlags,
    pub preferred_flags: vk::MemoryPropertyFlags,
    pub memory_type_bits: u32,
    pub pool: *const u8,
    pub p_user_data: *mut c_void,
    pub priority: f32,
}
impl Default for AllocationCreateInfo {
    fn default() -> Self {
        Self {
            flags: AllocationCreateFlags::empty(),
            usage: MemoryUsage::UNKNOWN,
            required_flags: vk::MemoryPropertyFlags::empty(),
            preferred_flags: vk::MemoryPropertyFlags::empty(),
            memory_type_bits: 0,
            pool: std::ptr::null(),
            p_user_data: std::ptr::null_mut(),
            priority: 0.5
        }
    }
}
impl AllocationCreateInfo {
    pub fn builder<'a>() -> AllocationCreateInfoBuilder<'a> {
        AllocationCreateInfoBuilder {
            inner: Self::default(),
            marker: std::marker::PhantomData,
        }
    }
}

#[repr(transparent)]
pub struct AllocationCreateInfoBuilder<'a> {
    inner: AllocationCreateInfo,
    marker: std::marker::PhantomData<&'a ()>,
}
impl<'a> std::ops::Deref for AllocationCreateInfoBuilder<'a> {
    type Target = AllocationCreateInfo;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<'a> std::ops::DerefMut for AllocationCreateInfoBuilder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl<'a> AllocationCreateInfoBuilder<'a> {
    #[inline(always)]
    pub fn flags(mut self, flags: AllocationCreateFlags) -> Self {
        self.flags = flags;
        self
    }

    #[inline(always)]
    pub fn usage(mut self, usage: MemoryUsage) -> Self {
        self.usage = usage;
        self
    }

    #[inline(always)]
    pub fn required_flags(mut self, required_flags: vk::MemoryPropertyFlags) -> Self {
        self.required_flags = required_flags;
        self
    }

    #[inline(always)]
    pub fn preferred_flags(mut self, preferred_flags: vk::MemoryPropertyFlags) -> Self {
        self.preferred_flags = preferred_flags;
        self
    }

    #[inline(always)]
    pub fn memory_type_bits(mut self, memory_type_bits: u32) -> Self {
        self.memory_type_bits = memory_type_bits;
        self
    }

    #[inline(always)]
    pub fn pool(mut self, pool: *const u8) -> Self {
        self.pool = pool;
        self
    }

    #[inline(always)]
    pub fn priority(mut self, priority: f32) -> Self {
        self.priority = priority;
        self
    }

    /// Calling build will **discard** all the lifetime information. Only call this if
    /// necessary! Builders implement `Deref` targeting their corresponding vma struct,
    /// so references to builders can be passed directly to vma functions.
    #[inline(always)]
    pub fn build(self) -> AllocationCreateInfo {
        self.inner
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AllocationInfo {
    pub memory_type: u32,
    pub device_memory: vk::DeviceMemory,
    pub offset: vk::DeviceSize,
    pub size: vk::DeviceSize,
    pub p_mapped_data: *mut c_void,
    pub p_user_data: *mut c_void,
    pub p_name: *const c_char,
}
impl Default for AllocationInfo {
    fn default() -> Self {
        Self {
            memory_type: 0,
            device_memory: vk::DeviceMemory::null(),
            offset: 0,
            size: 0,
            p_mapped_data: std::ptr::null_mut(),
            p_user_data: std::ptr::null_mut(),
            p_name: std::ptr::null()
        }
    }
}

#[repr(C)]
struct VulkanFunctions {
    vk_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    vk_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    _0: *const u8,
    _1: *const u8,
    _2: *const u8,
    _3: *const u8,
    _4: *const u8,
    _5: *const u8,
    _6: *const u8,
    _7: *const u8,
    _8: *const u8,
    _9: *const u8,
    _10: *const u8,
    _11: *const u8,
    _12: *const u8,
    _13: *const u8,
    _14: *const u8,
    _15: *const u8,
    _16: *const u8,
    _17: *const u8,
    _18: *const u8,
    _19: *const u8,
    _20: *const u8,
    _21: *const u8,
    _22: *const u8,
    _23: *const u8
}

impl VulkanFunctions {
    fn new_dynamic(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        Self {
            vk_get_instance_proc_addr: entry.static_fn().get_instance_proc_addr,
            vk_get_device_proc_addr: instance.fp_v1_0().get_device_proc_addr,
            _0: std::ptr::null(),
            _1: std::ptr::null(),
            _2: std::ptr::null(),
            _3: std::ptr::null(),
            _4: std::ptr::null(),
            _5: std::ptr::null(),
            _6: std::ptr::null(),
            _7: std::ptr::null(),
            _8: std::ptr::null(),
            _9: std::ptr::null(),
            _10: std::ptr::null(),
            _11: std::ptr::null(),
            _12: std::ptr::null(),
            _13: std::ptr::null(),
            _14: std::ptr::null(),
            _15: std::ptr::null(),
            _16: std::ptr::null(),
            _17: std::ptr::null(),
            _18: std::ptr::null(),
            _19: std::ptr::null(),
            _20: std::ptr::null(),
            _21: std::ptr::null(),
            _22: std::ptr::null(),
            _23: std::ptr::null()
        }
    }
}

#[repr(C)]
struct AllocatorCreateInfo {
    flags: AllocatorCreateFlags,
    physical_device: vk::PhysicalDevice,
    device: vk::Device,
    preferred_large_heap_block_size: vk::DeviceSize,
    p_allocation_callbacks: *const vk::AllocationCallbacks,
    p_device_memory_callbacks: *const u8,
    p_heap_size_limit: *const vk::DeviceSize,
    p_vulkan_functions: *const VulkanFunctions,
    instance: vk::Instance,
    vulkan_api_version: u32,
    p_type_external_memory_handle_types: *const u8,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
struct AllocatorHandle(*const u8);

pub struct Allocator {
    handle: AllocatorHandle,
}

impl Allocator {
    pub fn new(device: &DeviceFunctions, create_flags: AllocatorCreateFlags) -> Result<Self, vk::Result> {
        let functions = VulkanFunctions::new_dynamic(device.instance.get_entry(), device.instance.vk());

        let info = AllocatorCreateInfo {
            flags: create_flags,
            physical_device: device.physical_device,
            device: device.vk.handle(),
            preferred_large_heap_block_size: 0,
            p_allocation_callbacks: std::ptr::null(),
            p_device_memory_callbacks: std::ptr::null(),
            p_heap_size_limit: std::ptr::null(),
            p_vulkan_functions: &functions,
            instance: device.instance.vk().handle(),
            vulkan_api_version: device.instance.get_version().get_raw(),
            p_type_external_memory_handle_types: std::ptr::null()
        };

        let mut handle = AllocatorHandle(std::ptr::null());
        let result = unsafe {
            sys::vmaCreateAllocator(&info, &mut handle)
        };
        if result == vk::Result::SUCCESS {
            Ok(Self { handle })
        } else {
            Err(result)
        }
    }

    pub unsafe fn allocate_memory(&self, memory_requirements: &vk::MemoryRequirements, create_info: &AllocationCreateInfo, allocation_info: Option<&mut AllocationInfo>) -> Result<Allocation, vk::Result> {
        let mut handle = Allocation::null();
        let allocation_info = allocation_info.map(|i| i as *mut AllocationInfo).unwrap_or(std::ptr::null_mut());
        let result = sys::vmaAllocateMemory(self.handle, memory_requirements, create_info, &mut handle, allocation_info);
        if result == vk::Result::SUCCESS {
            Ok(handle)
        } else {
            Err(result)
        }
    }

    pub unsafe fn allocate_memory_pages(&self, memory_requirements: &[vk::MemoryRequirements], create_info: &[AllocationCreateInfo], allocation_info: Option<&mut [AllocationInfo]>) -> Result<Vec<Allocation>, vk::Result> {
        let count = memory_requirements.len();
        assert_eq!(create_info.len(), count);
        if let Some(info) = &allocation_info {
            assert_eq!(info.len(), count);
        }

        let mut handles = Vec::new();
        handles.resize(count, Allocation::null());
        let allocation_info = allocation_info.map(|i| i.as_mut_ptr()).unwrap_or(std::ptr::null_mut());
        let result = sys::vmaAllocateMemoryPages(self.handle, memory_requirements.as_ptr(), create_info.as_ptr(), count, handles.as_mut_ptr(), allocation_info);
        if result == vk::Result::SUCCESS {
            Ok(handles)
        } else {
            Err(result)
        }
    }

    pub unsafe fn free_memory(&self, allocation: Allocation) {
        sys::vmaFreeMemory(self.handle, allocation)
    }

    pub unsafe fn free_memory_pages(&self, allocations: &[Allocation]) {
        sys::vmaFreeMemoryPages(self.handle, allocations.len(), allocations.as_ptr())
    }

    pub unsafe fn get_allocation_info(&self, allocation: Allocation, info: &mut AllocationInfo) {
        sys::vmaGetAllocationInfo(self.handle, allocation, info)
    }

    pub unsafe fn create_buffer(&self, buffer_create_info: &vk::BufferCreateInfo, allocation_create_info: &AllocationCreateInfo, allocation_info: Option<&mut AllocationInfo>) -> Result<(vk::Buffer, Allocation), vk::Result> {
        let mut buffer_handle = vk::Buffer::null();
        let mut allocation_handle = Allocation::null();
        let allocation_info = allocation_info.map(|i| i as *mut AllocationInfo).unwrap_or(std::ptr::null_mut());
        let result = sys::vmaCreateBuffer(self.handle, buffer_create_info, allocation_create_info, &mut buffer_handle, &mut allocation_handle, allocation_info);
        if result == vk::Result::SUCCESS {
            Ok((buffer_handle, allocation_handle))
        } else {
            Err(result)
        }
    }

    pub unsafe fn destroy_buffer(&self, buffer: vk::Buffer, allocation: Allocation) {
        sys::vmaDestroyBuffer(self.handle, buffer, allocation)
    }

    pub unsafe fn create_image(&self, image_create_info: &vk::ImageCreateInfo, allocation_create_info: &AllocationCreateInfo, allocation_info: Option<&mut AllocationInfo>) -> Result<(vk::Image, Allocation), vk::Result> {
        let mut image_handle = vk::Image::null();
        let mut allocation_handle = Allocation::null();
        let allocation_info = allocation_info.map(|i| i as *mut AllocationInfo).unwrap_or(std::ptr::null_mut());
        let result = sys::vmaCreateImage(self.handle, image_create_info, allocation_create_info, &mut image_handle, &mut allocation_handle, allocation_info);
        if result == vk::Result::SUCCESS {
            Ok((image_handle, allocation_handle))
        } else {
            Err(result)
        }
    }

    pub unsafe fn destroy_image(&self, image: vk::Image, allocation: Allocation) {
        sys::vmaDestroyImage(self.handle, image, allocation)
    }
}

unsafe impl Send for Allocator {}
unsafe impl Sync for Allocator {}

impl Drop for Allocator {
    fn drop(&mut self) {
        unsafe {
            sys::vmaDestroyAllocator(self.handle)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Allocation(*const u8);

impl Allocation {
    pub const fn null() -> Self {
        Allocation(std::ptr::null())
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
impl Default for Allocation {
    fn default() -> Self {
        Self::null()
    }
}

mod sys {
    use super::*;

    #[link(name = "VulkanMemoryAllocator", kind = "static")]
    extern "C" {
        pub(super) fn vmaCreateAllocator(
            p_create_info: *const AllocatorCreateInfo,
            p_allocator: *mut AllocatorHandle
        ) -> vk::Result;

        pub(super) fn vmaDestroyAllocator(
            p_allocator: AllocatorHandle
        );

        pub(super) fn vmaAllocateMemory(
            allocator: AllocatorHandle,
            p_vk_memory_requirements: *const vk::MemoryRequirements,
            p_create_info: *const AllocationCreateInfo,
            p_allocation: *mut Allocation,
            p_allocation_info: *mut AllocationInfo,
        ) -> vk::Result;

        pub(super) fn vmaAllocateMemoryPages(
            allocator: AllocatorHandle,
            p_vk_memory_requirements: *const vk::MemoryRequirements,
            p_create_info: *const AllocationCreateInfo,
            allocation_count: usize,
            p_allocations: *mut Allocation,
            p_allocation_info: *mut AllocationInfo,
        ) -> vk::Result;

        pub(super) fn vmaFreeMemory(
            allocator: AllocatorHandle,
            allocation: Allocation,
        );

        pub(super) fn vmaFreeMemoryPages(
            allocator: AllocatorHandle,
            count: usize,
            p_allocations: *const Allocation,
        );

        pub(super) fn vmaGetAllocationInfo(
            allocator: AllocatorHandle,
            allocation: Allocation,
            p_allocation_info: *mut AllocationInfo,
        );

        pub(super) fn vmaCreateBuffer(
            allocator: AllocatorHandle,
            p_buffer_create_info: *const vk::BufferCreateInfo,
            p_allocation_create_info: *const AllocationCreateInfo,
            p_buffer: *mut vk::Buffer,
            p_allocation: *mut Allocation,
            p_allocation_info: *mut AllocationInfo,
        ) -> vk::Result;

        pub(super) fn vmaDestroyBuffer(
            allocator: AllocatorHandle,
            buffer: vk::Buffer,
            allocation: Allocation,
        );

        pub(super) fn vmaCreateImage(
            allocator: AllocatorHandle,
            p_image_create_info: *const vk::ImageCreateInfo,
            p_allocation_create_info: *const AllocationCreateInfo,
            p_image: *mut vk::Image,
            p_allocation: *mut Allocation,
            p_allocation_info: *mut AllocationInfo,
        ) -> vk::Result;

        pub(super) fn vmaDestroyImage(
            allocator: AllocatorHandle,
            image: vk::Image,
            allocation: Allocation,
        );
    }
}