use std::hash::Hash;
use std::sync::{Arc, Weak};
use ash::prelude::VkResult;

use ash::vk;
use bumpalo::Bump;
use crate::device::device::VkQueue;
use crate::device::device_utils::BlitPass;
use crate::device::surface::{AcquiredImageInfo, SurfaceSwapchain};

use crate::vk::DeviceEnvironment;

use crate::prelude::*;

use crate::vk::objects::buffer::Buffer;

pub use super::worker::SubmitRecorder;
pub use super::worker::PooledObjectProvider;

pub trait EmulatorPipeline: Send + Sync {
    /// Starts one pass of the pipeline
    fn start_pass(&self) -> Box<dyn EmulatorPipelinePass + Send>;

    /// Returns a list of all allowed pipeline types.
    ///
    /// The index into this list must be equal to the id of the type.
    fn get_type_table(&self) -> &[PipelineTypeInfo];

    fn get_outputs(&self) -> (Vec2u32, &[vk::ImageView]);
}

pub struct PipelineTypeInfo {
    /// The stride used when accessing the vertex data
    pub vertex_stride: u32,
}

/// Represents one execution of a [`EmulatorPipeline`]
pub trait EmulatorPipelinePass {
    fn init(&mut self, queue: &VkQueue, obj: &mut PooledObjectProvider);

    fn process_task(&mut self, task: PipelineTask, obj: &mut PooledObjectProvider);

    /// Records tasks for submission
    fn record<'a>(&mut self, obj: &mut PooledObjectProvider, submits: &mut SubmitRecorder<'a>, alloc: &'a Bump);

    fn get_output_index(&self) -> usize;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PipelineTask {
    SetModelViewMatrix(Mat4f32),
    SetProjectionMatrix(Mat4f32),
    Draw(DrawTask),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct DrawTask {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub vertex_offset: i32,
    pub first_index: u32,
    pub index_type: vk::IndexType,
    pub index_count: u32,
    pub type_id: u32,
}

pub trait EmulatorOutput {
    fn init(&mut self, pass: &dyn EmulatorPipelinePass, obj: &mut PooledObjectProvider);

    fn record<'a>(&mut self, obj: &mut PooledObjectProvider, submits: &mut SubmitRecorder<'a>, alloc: &'a Bump);

    fn on_post_submit(&mut self, queue: &VkQueue);
}

pub struct OutputUtil {
    pipeline: Arc<dyn EmulatorPipeline>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Box<[vk::DescriptorSet]>,
    blit_pass: BlitPass,
}

impl OutputUtil {
    pub fn new(device: &DeviceEnvironment, pipeline: Arc<dyn EmulatorPipeline>, format: vk::Format, final_layout: vk::ImageLayout) -> Self {
        let (_, sampler_views) = pipeline.get_outputs();

        let blit_pass = device.get_utils().blit_utils().create_blit_pass(format, vk::AttachmentLoadOp::DONT_CARE, vk::ImageLayout::UNDEFINED, final_layout);

        let descriptor_pool = Self::create_descriptor_pool(device, sampler_views.len());
        let descriptor_sets = blit_pass.create_descriptor_sets(descriptor_pool, sampler_views).unwrap().into_boxed_slice();

        Self {
            pipeline,
            descriptor_pool,
            descriptor_sets,
            blit_pass
        }
    }

    pub fn create_framebuffer(&self, image_view: vk::ImageView, size: Vec2u32) -> VkResult<vk::Framebuffer> {
        self.blit_pass.create_framebuffer(image_view, size)
    }

    pub fn record(&self, command_buffer: vk::CommandBuffer, output_framebuffer: vk::Framebuffer, output_size: Vec2u32, pipeline_index: usize) {
        self.blit_pass.record_blit(
            command_buffer,
            self.descriptor_sets[pipeline_index],
            output_framebuffer,
            output_size,
            None
        )
    }

    fn create_descriptor_pool(device: &DeviceEnvironment, sampler_count: usize) -> vk::DescriptorPool {
        let sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: sampler_count as u32,
            }
        ];

        let info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(sampler_count as u32)
            .pool_sizes(&sizes);

        unsafe {
            device.vk().create_descriptor_pool(&info, None)
        }.unwrap()
    }
}

impl Drop for OutputUtil {
    fn drop(&mut self) {
        unsafe {
            self.blit_pass.get_device().vk().destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}

pub struct SwapchainOutput {
    weak: Weak<Self>,
    swapchain: Arc<SurfaceSwapchain>,
    util: OutputUtil,
    framebuffers: Box<[vk::Framebuffer]>,
}

impl SwapchainOutput {
    pub fn new(device: &DeviceEnvironment, pipeline: Arc<dyn EmulatorPipeline>, swapchain: Arc<SurfaceSwapchain>) -> Arc<Self> {
        let util = OutputUtil::new(device, pipeline, swapchain.get_image_format().format, vk::ImageLayout::PRESENT_SRC_KHR);

        let framebuffers = swapchain.get_images().iter().map(|image| {
            util.create_framebuffer(image.get_framebuffer_view(), swapchain.get_image_size()).unwrap()
        }).collect();

        Arc::new_cyclic(|weak| Self {
            weak: weak.clone(),
            swapchain,
            util,
            framebuffers
        })
    }

    pub fn next_image(&self) -> Option<(Box<dyn EmulatorOutput + Send>, bool)> {
        let arc = self.weak.upgrade().unwrap();
        let (info, suboptimal) = self.swapchain.acquire_next_image(u64::MAX, None).ok()?;

        Some((Box::new(SwapchainOutputInstance::new(arc, info)), suboptimal))
    }
}

impl Drop for SwapchainOutput {
    fn drop(&mut self) {
        let device = self.swapchain.get_device();
        unsafe {
            for framebuffer in self.framebuffers.iter() {
                device.vk().destroy_framebuffer(*framebuffer, None);
            }
        }
    }
}

struct SwapchainOutputInstance {
    output: Arc<SwapchainOutput>,
    image_info: AcquiredImageInfo,
    pipeline_index: Option<usize>,
}

impl SwapchainOutputInstance {
    fn new(output: Arc<SwapchainOutput>, image_info: AcquiredImageInfo) -> Self {
        Self {
            output,
            image_info,
            pipeline_index: None,
        }
    }
}

impl EmulatorOutput for SwapchainOutputInstance {
    fn init(&mut self, pass: &dyn EmulatorPipelinePass, _: &mut PooledObjectProvider) {
        self.pipeline_index = Some(pass.get_output_index());
    }

    fn record<'a>(&mut self, obj: &mut PooledObjectProvider, submits: &mut SubmitRecorder<'a>, alloc: &'a Bump) {
        let cmd = obj.get_begin_command_buffer().unwrap();

        self.output.util.record(cmd, self.output.framebuffers[self.image_info.image_index as usize], self.output.swapchain.get_image_size(), self.pipeline_index.unwrap());

        unsafe {
            self.output.swapchain.get_device().vk().end_command_buffer(cmd)
        }.unwrap();

        let waits = alloc.alloc([
            vk::SemaphoreSubmitInfo::builder()
                .semaphore(self.image_info.acquire_semaphore.semaphore.get_handle())
                .value(self.image_info.acquire_semaphore.value.unwrap_or(0))
                .build()
        ]);

        let signals = alloc.alloc([
            vk::SemaphoreSubmitInfo::builder()
                .semaphore(self.image_info.acquire_ready_semaphore.semaphore.get_handle())
                .value(self.image_info.acquire_ready_semaphore.value.unwrap_or(0))
                .build(),
            vk::SemaphoreSubmitInfo::builder()
                .semaphore(self.output.swapchain.get_images()[self.image_info.image_index as usize].get_present_semaphore().get_handle())
                .build()
        ]);

        let commands = alloc.alloc([
            vk::CommandBufferSubmitInfo::builder()
                .command_buffer(cmd)
                .build()
        ]);

        submits.push(vk::SubmitInfo2::builder()
            .wait_semaphore_infos(waits)
            .command_buffer_infos(commands)
            .signal_semaphore_infos(signals)
        );
    }

    fn on_post_submit(&mut self, queue: &VkQueue) {
        let present_semaphore = self.output.swapchain.get_images()[self.image_info.image_index as usize].get_present_semaphore().get_handle();

        let guard = self.output.swapchain.get_swapchain().lock().unwrap();

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(std::slice::from_ref(&present_semaphore))
            .swapchains(std::slice::from_ref(&*guard))
            .image_indices(std::slice::from_ref(&self.image_info.image_index));

        unsafe {
            queue.present(&present_info)
        }.unwrap();
    }
}