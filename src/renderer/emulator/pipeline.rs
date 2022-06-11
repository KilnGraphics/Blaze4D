use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::{Arc, Weak};
use ash::prelude::VkResult;

use ash::vk;
use bumpalo::Bump;
use crate::device::device::Queue;
use crate::device::device_utils::BlitPass;
use crate::device::surface::{AcquiredImageInfo, SurfaceSwapchain};

use crate::vk::DeviceEnvironment;
use crate::vk::objects::buffer::Buffer;

use crate::prelude::*;
use crate::renderer::emulator::mc_shaders::ShaderId;

pub use super::worker::SubmitRecorder;
pub use super::worker::PooledObjectProvider;

/// A [`EmulatorPipeline`] performs the actual rendering inside a pass.
///
/// To define how objects should be rendered a pipeline can define multiple types. The meaning of
/// each type is pipeline dependant however the [`EmulatorRenderer`] requires some information about
/// the type which must be provided through [`EmulatorPipeline::get_type_info`].
pub trait EmulatorPipeline: Send + Sync + UnwindSafe + RefUnwindSafe {

    /// Called internally by the emulator renderer when a pass is started. All rendering will be
    /// performed using the returned object.
    ///
    /// This function must be called on thread with [`EmulatorRenderer::start_pass`] and thus may be
    /// used to block execution. For example to prevent an infinite build up of un-submitted passes
    /// if the user submits tasks faster than the gpu can process them.
    fn start_pass(&self) -> Box<dyn EmulatorPipelinePass + Send>;

    /// Returns the size and a list of image views which can be used as source images for samplers
    /// for the output of the pipeline.
    ///
    /// **This is a temporary api and needs a rework to improve flexibility and elegance**
    fn get_output(&self) -> (Vec2u32, &[vk::ImageView]);

    /// Called internally by the emulator renderer when pass uses a shader for the first time.
    /// A corresponding call to [`dec_shader_used`] will be performed after the corresponding pass
    /// has been dropped.
    ///
    /// It is guaranteed that this function will be called before the corresponding pass receives
    /// any task using this shader.
    ///
    /// This can be used to keep track of used shaders globally to manage vulkan pipelines.
    fn inc_shader_used(&self, shader: ShaderId);

    /// Called internally by the emulator renderer after a pass is dropped for each shader the pass
    /// used. Every call to this function must have had a earlier call to [`inc_shader_used`].
    ///
    /// This can be used to keep track of used shaders globally to manage vulkan pipelines.
    fn dec_shader_used(&self, shader: ShaderId);
}

/// Represents one execution of a [`EmulatorPipeline`].
///
/// A pass is processed in 3 stages.
/// 1. Uninitialized: The pass has just been created by calling [`EmulatorPipeline::start_pass`].
/// 2. Recording: The pass is currently recording tasks.
/// 3. Submitted: All command buffers have been submitted for execution.
///
/// Any instance of this struct will not be dropped until all submitted command buffers have
/// finished execution. If it is dropped it may assume that all used resources are safe to be
/// reused. A pass may be aborted at any moment for any reason.
pub trait EmulatorPipelinePass {

    /// Called to initialize internal state.
    ///
    /// This transitions the pass from the uninitialized state to the recording state.
    ///
    /// The queue which will be used to submit command buffers is provided. All resources (i.e.
    /// buffers, images etc.) passed to this pass will be owned by this queue family.
    fn init(&mut self, queue: &Queue, obj: &mut PooledObjectProvider);

    /// Called to process a task.
    ///
    /// Must only be called while the pass is in the recording state.
    fn process_task(&mut self, task: &PipelineTask, obj: &mut PooledObjectProvider);

    /// Called to record any necessary command buffer submissions for the execution of the pass.
    /// The recorded submits will be submitted by the calling code.
    ///
    /// This transitions the pass from the recording state to the submitted state.
    fn record<'a>(&mut self, obj: &mut PooledObjectProvider, submits: &mut SubmitRecorder<'a>, alloc: &'a Bump);

    /// Returns the index into the image view list returned by [`EmulatorPipeline::get_output`]
    /// determining which image view should be used to access the output of the pass.
    ///
    /// **This is a temporary api and needs a rework to improve flexibility and elegance**
    fn get_output_index(&self) -> usize;

    /// Called to retrieve a list of fences used to wait for internally submitted commands.
    ///
    /// In order to guarantee that any submissions made by the pass internally have completed
    /// execution this function returns a list of fences such that waiting on all fences implies
    /// that all submissions are done executing and resources can be safely reused.
    ///
    /// If any other function (except [`drop`]) of this pass are called after this function, the
    /// list of returned fences becomes invalid and a new call to this function must be made.
    ///
    /// TODO this is currently not used by the worker
    fn get_internal_fences(&self, fences: &mut Vec<vk::Fence>);
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PipelineTask {
    UpdateDevUniform(ShaderId, vk::Buffer, vk::DeviceSize),
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
    pub shader: ShaderId,
    pub primitive_topology: vk::PrimitiveTopology,
}

/// Used to process the output of a [`EmulatorPipelinePass`].
///
/// Any instance of this struct will not be dropped until all submitted command buffers have
/// finished execution. If it is dropped it may assume that all used resources are safe to be
/// reused.
///
/// **This is a temporary api and needs a rework to improve flexibility and elegance**
pub trait EmulatorOutput {
    /// Initializes the output to use the specified [`EmulatorPipelinePass`].
    fn init(&mut self, pass: &dyn EmulatorPipelinePass, obj: &mut PooledObjectProvider);

    /// Records any necessary submissions.
    /// The recorded submits will be submitted by the calling code.
    fn record<'a>(&mut self, obj: &mut PooledObjectProvider, submits: &mut SubmitRecorder<'a>, alloc: &'a Bump);

    /// Called after the submits recorded by [`EmulatorOutput::record`] have been submitted for
    /// execution. This is particularly useful to perform any queue present operations.
    fn on_post_submit(&mut self, queue: &Queue);
}

/// A utility struct providing a [`BlitPass`] for the output of a [`EmulatorPipeline`].
pub struct OutputUtil {
    #[allow(unused)] // We just need to keep the pipeline alive
    pipeline: Arc<dyn EmulatorPipeline>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Box<[vk::DescriptorSet]>,
    blit_pass: BlitPass,
}

impl OutputUtil {
    pub fn new(device: &DeviceEnvironment, pipeline: Arc<dyn EmulatorPipeline>, format: vk::Format, final_layout: vk::ImageLayout) -> Self {
        let (_, sampler_views) = pipeline.get_output();

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

    /// Creates a framebuffer which can be used as a draw target for the blit pass.
    ///
    /// The returned framebuffer is fully owned by the calling code and must be destroyed before
    /// this struct is dropped.
    pub fn create_framebuffer(&self, image_view: vk::ImageView, size: Vec2u32) -> VkResult<vk::Framebuffer> {
        self.blit_pass.create_framebuffer(image_view, size)
    }

    /// Records one execution of the blit pass.
    ///
    /// The pipeline index is the index returned by [`EmulatorPipelinePass::get_output_index`].
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

/// A [`EmulatorOutput`] implementation which copes the output image to a swapchain image and
/// presents it.
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

    /// Attempts to acquire a new image from the swapchain blocking until it does.
    ///
    /// Returns [`None`] if the swapchain is out of date.
    ///
    /// If it successfully acquires a image returns a [`EmulatorOutput`] instance for the image as
    /// well as a boolean flag set to true if the swapchain is suboptimal.
    pub fn next_image(&self) -> Option<(Box<dyn EmulatorOutput + Send>, bool)> {
        loop {
            let arc = self.weak.upgrade().unwrap();
            match self.swapchain.acquire_next_image(1000000000, None) {
                Ok((info, suboptimal)) =>
                    return Some((Box::new(SwapchainOutputInstance::new(arc, info)), suboptimal)),
                Err(vk::Result::TIMEOUT) =>
                    log::warn!("1s timeout reached while waiting for next swapchain image in SwapchainOutput::next_image"),
                Err(err) => {
                    log::error!("vkAcquireNextImageKHR returned {:?} in SwapchainOutput::next_image", err);
                    panic!()
                }
            }
        }
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

    fn on_post_submit(&mut self, queue: &Queue) {
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