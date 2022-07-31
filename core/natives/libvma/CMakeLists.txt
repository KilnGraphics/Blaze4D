project( b4d_core_vma )
cmake_minimum_required( VERSION 3.13 )
include(FetchContent)

FetchContent_Declare(
    vulkanMemoryAllocator
    GIT_REPOSITORY https://github.com/GPUOpen-LibrariesAndSDKs/VulkanMemoryAllocator.git
    GIT_TAG "v3.0.1"
)

set(CMAKE_DEBUG_POSTFIX "")

set(VMA_STATIC_VULKAN_FUNCTIONS OFF CACHE BOOL "")
set(VMA_DYNAMIC_VULKAN_FUNCTIONS ON CACHE BOOL "")
FetchContent_MakeAvailable(vulkanMemoryAllocator)

install(TARGETS VulkanMemoryAllocator DESTINATION .)