mod swapchain_manager;
pub mod emulator;

use ash::prelude::VkResult;
use ash::vk;
use ash::vk::{CommandPool, PipelineStageFlags};

use crate::debug::{ApplyTarget, DebugOverlay, Target};
use crate::debug::text::CharacterVertexData;
use crate::prelude::Vec2u32;
use crate::device::device::VkQueue;
use crate::vk::DeviceEnvironment;
use crate::vk::objects::{Format, ImageSubresourceRange, ImageViewDescription, ObjectSet, SwapchainObjectSet};
use crate::vk::objects::surface::SurfaceId;
use crate::vk::objects::swapchain::{SwapchainCreateDesc, SwapchainImageSpec};
use crate::vk::objects::types::{ImageViewId};

pub struct B4DRenderWorker {
    device: DeviceEnvironment,
    surface: SurfaceId,
}

impl B4DRenderWorker {
    pub fn new(device: DeviceEnvironment, surface: SurfaceId) -> Self {
        Self {
            device,
            surface
        }
    }

    pub fn run(&self) {
        let debug_overlay = DebugOverlay::new(self.device.clone());
        let overlay_target = debug_overlay.create_target(Vec2u32::new(1, 1));

        log::error!("SIZE: {:?}", std::mem::size_of::<CharacterVertexData>());

        let (swapchain, view_ids) = self.build_swapchain(&overlay_target);
        let set: &SwapchainObjectSet = swapchain.as_any().downcast_ref().unwrap();

        let image_ids = set.get_image_ids();

        let main_queue = self.device.get_device().get_main_queue();
        let transfer_queue = self.device.get_device().get_transfer_queue();

        log::error!("Main Queue: {:?} Transfer: {:?}", main_queue.get_queue_family_index(), transfer_queue.get_queue_family_index());

        let command_pool = self.create_command_pool(&main_queue).unwrap();

        let buffers = self.record_buffers(&swapchain, command_pool, view_ids, main_queue.get_queue_family_index());

        let sync = self.create_sync(2);
        let mut next_sync = 0;

        let swapchain_khr = self.device.get_device().swapchain_khr().unwrap();
        let swapchain_handle = unsafe { swapchain.get_data(set.get_swapchain_id()).get_handle() };
        loop {
            let (sem1, sem2, sem3, fence) = sync.get(next_sync).unwrap();
            next_sync = (next_sync + 1) % sync.len();

            unsafe { self.device.vk().wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX) }.unwrap();
            unsafe { self.device.vk().reset_fences(std::slice::from_ref(&fence)) }.unwrap();

            let (index, _) = unsafe { swapchain_khr.acquire_next_image(swapchain_handle, u64::MAX, *sem1, vk::Fence::null()) }.unwrap();

            let buffer = buffers.get(index as usize).unwrap();

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(std::slice::from_ref(sem1))
                .wait_dst_stage_mask(std::slice::from_ref(&PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT))
                .command_buffers(std::slice::from_ref(buffer))
                .signal_semaphores(std::slice::from_ref(sem2));

            unsafe { main_queue.submit(std::slice::from_ref(&submit_info), Some(*fence)) };

            overlay_target.apply_overlay(index as usize, Some(*sem2), Some(*sem3));

            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(std::slice::from_ref(sem3))
                .swapchains(std::slice::from_ref(&swapchain_handle))
                .image_indices(std::slice::from_ref(&index));

            unsafe { main_queue.present(&present_info) }.unwrap();
        }
    }

    fn build_swapchain(&self, target: &Target) -> (ObjectSet, Box<[ImageViewId]>) {
        let capabilities = self.device.get_device().get_surface_capabilities(self.surface).unwrap();
        log::error!("Capabilities: {:?}", capabilities);
        let formats = unsafe { self.device.get_instance().surface_khr().unwrap().get_physical_device_surface_formats(*self.device.get_device().get_physical_device(), self.device.get_device().get_surface(self.surface).unwrap().0).unwrap() };
        log::error!("Formats: {:?}", formats);

        let desc = SwapchainCreateDesc {
            min_image_count: capabilities.min_image_count,
            image_spec: SwapchainImageSpec {
                format: &Format::B8G8R8A8_SRGB,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
                extent: vk::Extent2D::builder().width(800).height(600).build(),
                array_layers: 1
            },
            usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
            pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: vk::PresentModeKHR::MAILBOX,
            clipped: true
        };

        let set = SwapchainObjectSet::new(self.device.clone(), self.surface, &desc).unwrap();

        let view_desc = ImageViewDescription::make_full(
            vk::ImageViewType::TYPE_2D,
            &Format::B8G8R8A8_SRGB,
            vk::ImageAspectFlags::COLOR
        );

        let image_ids = Vec::from(set.get_image_ids()).into_boxed_slice();
        let view_ids = set.add_image_views(&view_desc);

        let set = ObjectSet::new(set);

        let mut targets = Vec::with_capacity(image_ids.len());
        for (index, image) in image_ids.iter().enumerate() {
            targets.push(ApplyTarget {
                image: unsafe { set.get_data(*image).get_handle() },
                format: vk::Format::B8G8R8A8_SRGB,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1
                },
                src_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                dst_layout: vk::ImageLayout::PRESENT_SRC_KHR
            });
        }
        target.resize(Vec2u32::new(800, 600));
        target.prepare_targets(targets.as_slice()).unwrap();

        (set, view_ids)
    }

    fn create_command_pool(&self, queue: &VkQueue) -> VkResult<vk::CommandPool> {
        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue.get_queue_family_index());

        unsafe { self.device.vk().create_command_pool(&info, None) }
    }

    fn record_buffers(&self, set: &ObjectSet, pool: CommandPool, view_ids: Box<[ImageViewId]>, src_family: u32) -> Box<[vk::CommandBuffer]> {
        let swapchain: &SwapchainObjectSet = set.as_any().downcast_ref().unwrap();

        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(swapchain.get_image_ids().len() as u32);

        let buffers = unsafe { self.device.vk().allocate_command_buffers(&alloc_info) }.unwrap();

        for (index, buffer) in buffers.iter().enumerate() {
            let image = unsafe { set.get_data(*swapchain.get_image_ids().get(index).unwrap()).get_handle() };
            let image_view = unsafe { set.get_data(*view_ids.get(index).unwrap()).get_handle() };

            let begin_info = vk::CommandBufferBeginInfo::builder();

            unsafe { self.device.vk().begin_command_buffer(*buffer, &begin_info).unwrap() };

            let color_attachement = vk::RenderingAttachmentInfo::builder()
                .image_view(image_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .resolve_mode(vk::ResolveModeFlags::NONE)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(vk::ClearValue{ color: vk::ClearColorValue{ uint32: [0, 255, 0, 0] } });

            let rendering_info = vk::RenderingInfo::builder()
                .render_area(vk::Rect2D::from(vk::Extent2D::builder().width(400).height(400).build()))
                .layer_count(1)
                .color_attachments(std::slice::from_ref(&color_attachement));

            // unsafe { self.device.vk().cmd_begin_rendering(*buffer, &rendering_info) };

            // unsafe { self.device.vk().cmd_end_rendering(*buffer) };

            let image_barrier = vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::NONE)
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .image(image)
                .subresource_range(ImageSubresourceRange::full_color().as_vk_subresource_range());

            unsafe { self.device.vk().cmd_pipeline_barrier(
                *buffer,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::default(),
                &[],
                &[],
                std::slice::from_ref(&image_barrier)
            )};

            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(vk::REMAINING_MIP_LEVELS)
                .base_array_layer(0)
                .layer_count(vk::REMAINING_ARRAY_LAYERS);

            unsafe { self.device.vk().cmd_clear_color_image(
                *buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &vk::ClearColorValue{ float32: [0.0, 255.0, 0.0, 0.0] },
                std::slice::from_ref(&subresource_range)
            )};

            let image_barrier = vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::NONE)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .image(image)
                .subresource_range(ImageSubresourceRange::full_color().as_vk_subresource_range());

            unsafe { self.device.vk().cmd_pipeline_barrier(
                *buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::DependencyFlags::default(),
                &[],
                &[],
                std::slice::from_ref(&image_barrier)
            )};

            unsafe { self.device.vk().end_command_buffer(*buffer) }.unwrap();
        }

        buffers.into_boxed_slice()
    }

    fn create_sync(&self, count: usize) -> Box<[(vk::Semaphore, vk::Semaphore, vk::Semaphore, vk::Fence)]> {
        let mut result = Vec::with_capacity(count);

        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..count {
            let semaphore1 = unsafe { self.device.vk().create_semaphore(&semaphore_info, None) }.unwrap();
            let semaphore2 = unsafe { self.device.vk().create_semaphore(&semaphore_info, None) }.unwrap();
            let semaphore3 = unsafe { self.device.vk().create_semaphore(&semaphore_info, None) }.unwrap();
            let fence = unsafe { self.device.vk().create_fence(&fence_info, None) }.unwrap();

            result.push((semaphore1, semaphore2, semaphore3, fence));
        }

        result.into_boxed_slice()
    }
}