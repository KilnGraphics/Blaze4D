use std::sync::Arc;
use ash::vk;
use crate::device::surface::SurfaceSwapchain;
use crate::renderer::emulator::{ExportHandle, ExportSet};

use crate::prelude::*;

pub struct EmulatorSwapchainExport {
    device: Arc<DeviceFunctions>,
    swapchain: Arc<SurfaceSwapchain>,
    queue: Arc<Queue>,

    command_pool: vk::CommandPool,

    next_cmd_buffer: vk::CommandBuffer,
    next_cmd_fence: vk::Fence,

    reserve_cmd_buffer: vk::CommandBuffer,
    reserve_cmd_fence: vk::Fence,
    reserve_cmd_wait: bool,
}

impl EmulatorSwapchainExport {
    pub fn new(swapchain: Arc<SurfaceSwapchain>, queue: Arc<Queue>) -> Result<Self, vk::Result> {
        let device = swapchain.get_device().clone();

        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::TRANSIENT | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue.get_queue_family_index());

        let command_pool = unsafe {
            device.vk.create_command_pool(&info, None)
        }?;

        let info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(2);

        let command_buffers = unsafe {
            device.vk.allocate_command_buffers(&info)
        }.map_err(|err| {
            unsafe { device.vk.destroy_command_pool(command_pool, None); }
            err
        })?;

        let info = vk::FenceCreateInfo::builder();
        let next_cmd_fence = unsafe {
            device.vk.create_fence(&info, None)
        }.map_err(|err| {
            unsafe { device.vk.destroy_command_pool(command_pool, None); }
            err
        })?;
        let reserve_cmd_fence = unsafe {
            device.vk.create_fence(&info, None)
        }.map_err(|err| {
            unsafe { device.vk.destroy_command_pool(command_pool, None); }
            unsafe { device.vk.destroy_fence(next_cmd_fence, None); }
            err
        })?;

        Ok(Self {
            device,
            swapchain,
            queue,
            command_pool,
            next_cmd_buffer: command_buffers[0],
            next_cmd_fence,
            reserve_cmd_buffer: command_buffers[1],
            reserve_cmd_fence,
            reserve_cmd_wait: false
        })
    }

    pub fn present_export(&mut self, export: &ExportSet) -> Result<bool, vk::Result> {
        let (acquired, suboptimal) = unsafe { self.swapchain.acquire_next_image(1000000000)? };
        let signal_semaphore = acquired.acquire_ready_semaphore.semaphore.get_handle();
        let signal_value = acquired.acquire_ready_semaphore.value.unwrap();

        unsafe {
            self.device.vk.reset_command_buffer(self.next_cmd_buffer, vk::CommandBufferResetFlags::empty())?;
        }

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device.vk.begin_command_buffer(self.next_cmd_buffer, &info)?;
        }

        todo!()
    }
}