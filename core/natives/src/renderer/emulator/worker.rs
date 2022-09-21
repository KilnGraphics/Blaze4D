use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::collections::hash_map::RandomState;
use std::default;
use std::ffi::CString;
use std::hash::{BuildHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::process::exit;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ash::prelude::VkResult;
use ash::vk;
use bumpalo::Bump;
use ordered_float::NotNan;
use rspirv::spirv::CLOp::s_clamp;

use crate::device::device::Queue;

use crate::renderer::emulator::pass::PassId;
use crate::renderer::emulator::immediate::ImmediateBuffer;
use crate::renderer::emulator::pipeline::{EmulatorOutput, EmulatorPipeline, EmulatorPipelinePass, PipelineTask};

use super::share::{NextTaskResult, Share, Share2};

use crate::prelude::*;
use crate::renderer::emulator::global_objects::{GlobalImage, GlobalMesh};
use crate::renderer::emulator::{PipelineState, Image, FramebufferState, GraphicsPipeline, PipelineInputAttribute, PipelineColorBlendState, EmulatorTask, ImageId};
use crate::renderer::emulator::mc_shaders::ShaderId;
use crate::renderer::emulator::objects::{Buffer, BufferId, BufferInfo, GraphicsPipelineId};
use crate::renderer::emulator::staging::{StagingAllocationId2, StagingAllocationId};

mod task {
    use std::any::Any;
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::pin::Pin;
    use std::sync::Arc;

    use bumpalo::Bump;
    use crate::renderer::emulator::EmulatorTask;
    use crate::renderer::emulator::share::Share2;
    use crate::renderer::emulator::staging::StagingAllocationId2;

    pub(in crate::renderer::emulator)
    enum WorkerTask3 {
        Emulator(u64, EmulatorTaskContainer),
        Flush,
        Shutdown,
    }

    pub(in crate::renderer::emulator)
    struct EmulatorTaskContainer(TaskContainerPayload);

    impl EmulatorTaskContainer {
        /// # Safety
        /// - The alloc parameter must have been used to create all objects of the 'a lifetime of the
        /// task.
        /// - The objects parameter must have been used to create all objects of the 'o lifetime of the
        /// task.
        pub(in crate::renderer::emulator)
        unsafe fn new(alloc: Bump, task: EmulatorTask<'static>) -> Self {
            Self(TaskContainerPayload(Some((alloc, task))))
        }

        pub(super)
        fn as_ref<'s>(&'s self) -> &'s EmulatorTask<'s> {
            unsafe {
                std::mem::transmute::<&'s EmulatorTask<'static>, &'s EmulatorTask<'s>>(&(self.0.0.as_ref().unwrap().1))
            }
        }

        pub(super)
        fn unwrap<'a, 'o: 'a>(mut self, allocation_cache: &'a AllocationCache) -> EmulatorTask<'a> {
            let (alloc, task) = self.0.0.take().unwrap();
            allocation_cache.allocations.borrow_mut().push(alloc);

            // We moved the backing objects into the corresponding caches so this is safe
            unsafe {
                std::mem::transmute::<EmulatorTask<'static>, EmulatorTask<'a>>(task)
            }
        }
    }

    struct TaskContainerPayload(Option<(Bump, EmulatorTask<'static>)>);

    impl Drop for TaskContainerPayload {
        fn drop(&mut self) {
            if let Some((alloc, task)) = self.0.take() {
                // Let rust figure out the drop order
                let task = unsafe { bind_lifetimes(&alloc, task) };
                drop(task);
            }
        }
    }

    unsafe fn bind_lifetimes<'a>(_: &'a Bump, task: EmulatorTask<'static>) -> EmulatorTask<'a> {
        std::mem::transmute(task)
    }

    pub(super) struct ObjectCache {
        share: Arc<Share2>,
        objects: Vec<Arc<dyn Any + Send + Sync>>,
        staging_allocations: Vec<StagingAllocationId2>,
    }

    impl ObjectCache {
        pub(super) fn new(share: Arc<Share2>) -> Self {
            Self {
                share,
                objects: Vec::new(),
                staging_allocations: Vec::new(),
            }
        }

        pub(super) fn push_object(&mut self, object: Arc<dyn Any + Send + Sync>) {
            self.objects.push(object);
        }

        // TODO should this be unsafe?
        pub(super) fn push_staging(&mut self, alloc: StagingAllocationId2) {
            self.staging_allocations.push(alloc);
        }
    }

    impl Drop for ObjectCache {
        fn drop(&mut self) {
            unsafe {
                self.share.free_staging(std::mem::replace(&mut self.staging_allocations, Vec::new()).into_iter())
            }
        }
    }

    pub(super) struct AllocationCache {
        allocations: RefCell<Vec<Bump>>,
    }

    impl AllocationCache {
        pub(super) fn new() -> Self {
            Self {
                allocations: RefCell::new(Vec::new()),
            }
        }

        pub(super) fn reset(&mut self) {
            self.allocations.borrow_mut().clear();
        }
    }
}

pub(super) use task::{EmulatorTaskContainer, WorkerTask3};
use task::{AllocationCache, ObjectCache};

pub(super) enum WorkerTask2 {
    CopyStagingToBuffer(CopyStagingToBufferTask),
    CopyBufferToStaging(CopyBufferToStagingTask),
    CopyStagingToImage(CopyStagingToImageTask),
    CopyImageToStaging(CopyImageToStagingTask),
    Draw(DrawTask),
    Flush,
    Shutdown,
}

pub(super) struct CopyStagingToBufferTask {
    pub staging_allocation: StagingAllocationId2,
    pub staging_buffer: vk::Buffer,
    pub staging_buffer_offset: vk::DeviceSize,
    pub dst_buffer: Arc<Buffer>,
    pub dst_offset: vk::DeviceSize,
    pub copy_size: vk::DeviceSize,
}

pub(super) struct CopyBufferToStagingTask {
    pub staging_buffer: vk::Buffer,
    pub staging_buffer_offset: vk::DeviceSize,
    pub src_buffer: Arc<Buffer>,
    pub src_offset: vk::DeviceSize,
    pub copy_size: vk::DeviceSize,
}

pub(super) struct CopyStagingToImageTask {
    pub staging_allocation: StagingAllocationId2,
    pub staging_buffer: vk::Buffer,
    pub dst_image: Arc<Image>,
    pub copy_regions: Box<[vk::BufferImageCopy]>,
}

pub(super) struct CopyImageToStagingTask {
    pub staging_buffer: vk::Buffer,
    pub src_image: Arc<Image>,
    pub copy_regions: Box<[vk::BufferImageCopy]>,
}

pub(super) struct DrawTask {
    pub pipeline: Arc<GraphicsPipeline>,
    pub input_attributes: Box<[PipelineInputAttribute]>,
    pub pipeline_state: PipelineState,
    pub framebuffer_state: FramebufferState,
    pub draw_count: u32,
}

pub(super) fn run_worker2(share: Arc<Share2>) {
    let mut object_pool = RefCell::new(ObjectPool2::new(share.clone()).unwrap());
    let mut artifacts = VecDeque::with_capacity(3);

    let mut recorder = None;

    let mut last_sync = 0u64;
    let mut next_sync = 0u64;
    let mut last_update = Instant::now();
    loop {
        if let Some(task) = share.pop_task(Duration::from_millis(33)) {
            match task {
                WorkerTask3::Emulator(id, task) => {
                    if recorder.is_none() {
                        recorder = Some(Recorder::new(&object_pool));
                    }
                    recorder.as_mut().unwrap().push_task(task);
                    next_sync = id;
                }
                WorkerTask3::Flush |
                WorkerTask3::Shutdown => {
                    if let Some(recorder) = recorder.take() {
                        artifacts.push_back(recorder.submit(last_sync, next_sync));
                        last_sync = next_sync;
                    }
                    if last_sync != next_sync {
                        todo!()
                    }
                    if let WorkerTask3::Shutdown = task {
                        break;
                    }
                }
            }

            while let Some(artifact) = artifacts.front() {
                if artifact.is_done() {
                    artifacts.pop_front();
                } else {
                    break;
                }
            }

            let now = Instant::now();
            if now.duration_since(last_update) >= Duration::from_secs(10) {
                share.update();
                last_update = now;
            }
        }
    }

    let semaphore = share.get_semaphore();
    loop {
        let info = vk::SemaphoreWaitInfo::builder()
            .semaphores(std::slice::from_ref(&semaphore))
            .values(std::slice::from_ref(&last_sync));

        match unsafe {
            share.get_device().timeline_semaphore_khr().wait_semaphores(&info, 1000000000)
        } {
            Ok(()) => break,
            Err(vk::Result::TIMEOUT) => log::warn!("Hit timeout while waiting for vkWaitSemaphores"),
            Err(err) => panic!("vkWaitSemaphores returned {:?}", err),
        }
    }
}

/// Provides a pool of vulkan objects to allow for object reuse.
struct ObjectPool2 {
    share: Arc<Share2>,
    device: Arc<DeviceContext>,

    command_pool: vk::CommandPool,
    available_buffers: Vec<vk::CommandBuffer>,
}

impl ObjectPool2 {
    fn new(share: Arc<Share2>) -> Result<Self, vk::Result> {
        let device = share.get_device().clone();

        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(share.get_queue().get_queue_family_index());

        let command_pool = unsafe {
            device.vk().create_command_pool(&info, None)
        }?;

        Ok(Self {
            share,
            device,
            command_pool,
            available_buffers: Vec::new(),
        })
    }

    fn get_share(&self) -> &Arc<Share2> {
        &self.share
    }

    fn get_device(&self) -> &Arc<DeviceContext> {
        &self.device
    }

    /// Retrieves a new command buffer from the pool, creating a new one if necessary.
    ///
    /// The returned command buffer is guaranteed to be in the initial state.
    fn get_command_buffer(&mut self) -> vk::CommandBuffer {
        if let Some(cmd) = self.available_buffers.pop() {
            cmd
        } else {
            let info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(8);

            let mut buffers = unsafe {
                self.device.vk().allocate_command_buffers(&info)
            }.expect("Failed to allocate command buffers"); // Maybe recover from out of device memory?

            let cmd = buffers.pop().unwrap();
            self.available_buffers.extend(buffers);

            cmd
        }
    }

    /// Retrieves a command buffer from the pool, creating a new one if necessary and begins
    /// recording for a one time submission.
    fn get_begin_command_buffer(&mut self) -> vk::CommandBuffer {
        let cmd = self.get_command_buffer();

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device.vk().begin_command_buffer(cmd, &info)
        }.expect("Failed to begin command buffer recording"); // Maybe recover from out of device memory?

        cmd
    }

    /// Returns a set of command buffer to the pool. The command buffers will be reset by this
    /// function.
    ///
    /// # Safety
    /// The command buffers must be safe to be reset or destroyed and must have previously been
    /// retrieved from a call to this instance.
    unsafe fn return_command_buffers<I: IntoIterator<Item=vk::CommandBuffer>>(&mut self, iter: I) {
        for cmd in iter.into_iter() {
            unsafe {
                self.device.vk().reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
            }.expect("Failed to reset command buffers"); // Maybe recover from out of device memory?

            self.available_buffers.push(cmd);
        }
    }
}

impl Drop for ObjectPool2 {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_command_pool(self.command_pool, None);
        }
    }
}

mod recorder {
    use std::borrow::BorrowMut;
    use std::cell::{Ref, RefCell, RefMut};
    use std::collections::HashMap;
    use std::ops::Deref;
    use std::rc::Rc;
    use std::sync::Arc;

    use ash::vk;
    use bumpalo::Bump;
    use ouroboros::self_referencing;

    use crate::renderer::emulator::{Buffer, EmulatorTask, Image, ImageId};
    use super::{AllocationCache, EmulatorTaskContainer, ObjectCache, ObjectPool2};

    use crate::prelude::*;
    use crate::renderer::emulator::share::Share2;

    type BBox<'a, T> = bumpalo::boxed::Box<'a, T>;

    /// Container to keep track of submission state and keep used objects alive.
    ///
    /// # Safety
    /// This struct drops any object references on drop so [`SubmissionArtifact::is_done()`] must
    /// return true before drop otherwise objects which are still in use may be destroyed.
    pub(super) struct SubmissionArtifact<'a> {
        object_pool: &'a RefCell<ObjectPool2>,
        cmd: vk::CommandBuffer,
        object_cache: ObjectCache,
        wait_value: u64,
    }

    impl<'a> SubmissionArtifact<'a> {
        /// Returns true if the submission has finished execution.
        pub(super) fn is_done(&self) -> bool {
            let pool = self.object_pool.borrow();

            unsafe {
                pool.get_device().timeline_semaphore_khr().get_semaphore_counter_value(pool.share.get_semaphore())
            }.unwrap() >= self.wait_value
        }
    }

    impl<'a> Drop for SubmissionArtifact<'a> {
        fn drop(&mut self) {
            unsafe {
                self.object_pool.borrow_mut().return_command_buffers(std::iter::once(self.cmd))
            }
        }
    }

    /// Struct used to record command buffers.
    ///
    /// [`EmulatorTask`]s can be added by calling [`Recorder::push_task`]. The recorded work can
    /// then be submitted to the main queue by calling [`Recorder::submit`].
    pub(super) struct Recorder<'a> {
        recorder: Option<PassRecorder<'a>>,
    }

    impl<'a> Recorder<'a> {
        pub(super) fn new(object_pool: &'a RefCell<ObjectPool2>) -> Self {
            Self {
                recorder: Some(PassRecorder::new(object_pool))
            }
        }

        pub(super) fn push_task(&mut self, task: EmulatorTaskContainer) {
            self.recorder.as_mut().unwrap().push_task(task);
        }

        pub(super) fn submit(mut self, wait_value: u64, signal_value: u64) -> SubmissionArtifact<'a> {
            self.recorder.take().unwrap().submit(wait_value, signal_value)
        }
    }

    /// Implementation of the [`Recorder`] functions. Needed because this struct has to implement
    /// [`Drop`] which would make it impossible to define functions consuming self like
    /// [`Recorder::submit`].
    ///
    /// # Task reordering
    /// The recording process is structured into batches. One batch of tasks is represented by a
    /// [`RecorderContainer`]. The goal is to batch tasks in such a way to minimize pipeline
    /// barriers and render pass restarts.
    ///
    /// To prevent render pass restarts tasks can be reordered. This is implemented by having 2
    /// active [`RecorderContainer`]s. A main one and one for reordered tasks. If a task which
    /// cannot be recorded into a render pass is pushed while a render pass is active in the main
    /// recorder it checks if it is possible to reorder the task before the render pass. If so
    /// the task will be pushed into the reorder recorder. If not the render pass will be ended and
    /// recording happens as normal.
    ///
    /// If the main recorder cannot record a task and it is not possible to reorder the task the
    /// current reorder recorder will be completed and record its task into the command buffer. The
    /// current main recorder will then become the new reorder recorder and a new main recorder will
    /// be created.
    ///
    /// If a reordered task cannot be recorded into the reorder recorder the reorder recorder will
    /// be completed and a new one will be created without moving the main recorder.
    ///
    /// # Synchronization state tracking
    /// Object state is tracked for synchronization purposes inside the [`PassRecorder`]. It is not
    /// necessary to track state outside of a pass because semaphore wait/signal operations
    /// implicitly create a global memory barrier. The only exception to this are image layouts
    /// which is tracked inside [`Image`]s.
    struct PassRecorder<'a> {
        object_pool: &'a RefCell<ObjectPool2>,

        device: Arc<DeviceContext>,
        cmd: Option<vk::CommandBuffer>,

        object_cache: Option<ObjectCache>,

        pre_state: Option<SyncState>,
        reorder_recorder: Option<RecorderContainer>,
        main_recorder: Option<RecorderContainer>,
    }

    impl<'a> PassRecorder<'a> {
        fn new(object_pool: &'a RefCell<ObjectPool2>) -> Self {
            let mut pool = object_pool.borrow_mut();
            let share = pool.get_share().clone();
            let device = pool.get_device().clone();

            let cmd = pool.get_begin_command_buffer();

            Self {
                object_pool,

                device,
                cmd: Some(cmd),

                object_cache: Some(ObjectCache::new(share)),

                pre_state: None,
                reorder_recorder: None,
                main_recorder: None
            }
        }

        fn push_task(&mut self, task: EmulatorTaskContainer) {
            if self.main_recorder.is_none() {
                self.main_recorder = Some(Self::new_recorder_container())
            }
            if let Err((task, reorder)) = self.main_recorder.as_mut().unwrap().with_internal_mut(|internal| {
                internal.push_task(task, self.object_cache.as_mut().unwrap())
            }) {
                if reorder {
                    self.ensure_reorder_recorder_exists();
                    if let Err((task, _)) = self.reorder_recorder.as_mut().unwrap().with_internal_mut(|internal| {
                        internal.push_task(task, self.object_cache.as_mut().unwrap())
                    }) {
                        self.finish_reorder_recorder();
                        self.ensure_reorder_recorder_exists();
                        if let Err(_) = self.reorder_recorder.as_mut().unwrap().with_internal_mut(|internal| {
                            internal.push_task(task, self.object_cache.as_mut().unwrap())
                        }) {
                            panic!("Failed to record task into newly created reorder recorder!");
                        }
                    }
                } else {
                    self.reorder_main_recorder();
                    self.ensure_main_recorder_exists();
                    if let Err(_) = self.main_recorder.as_mut().unwrap().with_internal_mut(|internal| {
                        internal.push_task(task, self.object_cache.as_mut().unwrap())
                    }) {
                        panic!("Failed to record task into newly created main recorder!");
                    }
                }
            }
        }

        fn submit(&mut self, wait_value: u64, signal_value: u64) -> SubmissionArtifact<'a> {
            self.reorder_main_recorder();
            self.finish_reorder_recorder();

            debug_assert!(self.reorder_recorder.is_none());
            debug_assert!(self.main_recorder.is_none());

            let queue = self.device.get_main_queue();
            let semaphore = self.object_pool.borrow().get_share().get_semaphore();

            let post_state = self.pre_state.take().unwrap();
            for (image, state) in post_state.images {
                match state {
                    ImageState::ReadUniform(layout, _, _) |
                    ImageState::ReadWriteUniform(layout, _, _) => {
                        unsafe { image.set_current_layout(layout) };
                    }
                }
            }

            unsafe {
                self.device.vk().end_command_buffer(self.cmd.unwrap())
            }.unwrap();

            let wait_info = vk::SemaphoreSubmitInfo::builder()
                .semaphore(semaphore)
                .value(wait_value)
                .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS);

            let signal_info = vk::SemaphoreSubmitInfo::builder()
                .semaphore(semaphore)
                .value(signal_value)
                .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS);

            let cmd_info = vk::CommandBufferSubmitInfo::builder()
                .command_buffer(self.cmd.unwrap());

            let info = vk::SubmitInfo2::builder()
                .wait_semaphore_infos(std::slice::from_ref(&wait_info))
                .command_buffer_infos(std::slice::from_ref(&cmd_info))
                .signal_semaphore_infos(std::slice::from_ref(&signal_info));

            unsafe {
                queue.submit_2(std::slice::from_ref(&info), None)
            }.unwrap();

            SubmissionArtifact {
                object_pool: self.object_pool,
                cmd: self.cmd.take().unwrap(),
                object_cache: self.object_cache.take().unwrap(),
                wait_value: signal_value
            }
        }

        fn ensure_reorder_recorder_exists(&mut self) {
            if self.reorder_recorder.is_none() {
                self.reorder_recorder = Some(Self::new_recorder_container());
            }
        }

        fn finish_reorder_recorder(&mut self) {
            if let Some(mut reorder_recorder) = self.reorder_recorder.take() {
                self.pre_state = reorder_recorder.with_internal_mut(|internal| {
                    Some(internal.record(&self.device, self.cmd.unwrap(), self.pre_state.take()))
                });
            }
        }

        fn ensure_main_recorder_exists(&mut self) {
            if self.main_recorder.is_none() {
                self.main_recorder = Some(Self::new_recorder_container());
            }
        }

        fn reorder_main_recorder(&mut self) {
            self.finish_reorder_recorder();
            self.reorder_recorder = self.main_recorder.take();
        }

        fn new_recorder_container() -> RecorderContainer {
            RecorderContainer::new(
                AllocationCache::new(),
                |allocation_cache| {
                    RecorderInternal::new(allocation_cache)
                }
            )
        }
    }

    impl<'a> Drop for PassRecorder<'a> {
        fn drop(&mut self) {
            if let Some(cmd) = self.cmd.take() {
                unsafe {
                    self.object_pool.borrow_mut().return_command_buffers(std::iter::once(cmd));
                }
            }
        }
    }

    #[self_referencing]
    struct RecorderContainer {
        allocation_cache: AllocationCache,
        #[covariant]
        #[borrows(allocation_cache)]
        internal: RecorderInternal<'this>,
    }

    struct RecorderInternal<'a> {
        allocation_cache: &'a AllocationCache,
        state: SyncState,
        recordable: Vec<Recordable<'a>>,
    }

    impl<'a> RecorderInternal<'a> {
        pub(super) fn new(allocation_cache: &'a AllocationCache) -> Self {
            Self {
                allocation_cache,
                state: SyncState::new(),
                recordable: Vec::new(),
            }
        }

        pub(super) fn push_task(&mut self, task: EmulatorTaskContainer, object_cache: &mut ObjectCache) -> Result<(), (EmulatorTaskContainer, bool)> {
            match task.as_ref() {
                EmulatorTask::CopyStagingToBuffer(task_r) => {
                    let mut new_state = BufferState::ReadWriteUniform(vk::PipelineStageFlags2::TRANSFER, vk::AccessFlags2::TRANSFER_WRITE);
                    if let Some(state) = self.state.buffers.get(&task_r.dst_buffer) {
                        // Check that we can extend without needing a barrier
                        match state.try_extend(&new_state) {
                            Some(new_state2) => new_state = new_state2,
                            None => return Err((task, false))
                        }
                    }

                    // Checks completed
                    let task = task.unwrap(self.allocation_cache);
                    let task = match task {
                        EmulatorTask::CopyStagingToBuffer(task) => BBox::into_inner(task),
                        _ => panic!()
                    };
                    object_cache.push_object(task.dst_buffer.clone());
                    object_cache.push_staging(task.staging_allocation);

                    self.recordable.push(Recordable::BufferCopy {
                        src_buffer: task.staging_buffer,
                        dst_buffer: task.dst_buffer.get_handle(),
                        regions: task.copy_regions,
                    });
                    self.state.buffers.insert(task.dst_buffer, new_state);
                }
                EmulatorTask::CopyBufferToStaging(task_r) => {
                    let mut new_state = BufferState::ReadUniform(vk::PipelineStageFlags2::TRANSFER, vk::AccessFlags2::TRANSFER_READ);
                    if let Some(state) = self.state.buffers.get(&task_r.src_buffer) {
                        // Check that we can extend without needing a barrier
                        match state.try_extend(&new_state) {
                            Some(new_state2) => new_state = new_state2,
                            None => return Err((task, false))
                        }
                    }

                    // Checks completed
                    let task = task.unwrap(self.allocation_cache);
                    let task = match task {
                        EmulatorTask::CopyBufferToStaging(task) => BBox::into_inner(task),
                        _ => panic!()
                    };
                    object_cache.push_object(task.src_buffer.clone());

                    self.recordable.push(Recordable::BufferCopy {
                        src_buffer: task.src_buffer.get_handle(),
                        dst_buffer: task.staging_buffer,
                        regions: task.copy_regions,
                    });
                    self.state.buffers.insert(task.src_buffer, new_state);
                },
                EmulatorTask::CopyStagingToImage(task_r) => {
                    let mut new_state = ImageState::ReadWriteUniform(vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::PipelineStageFlags2::TRANSFER, vk::AccessFlags2::TRANSFER_WRITE);
                    if let Some(state) = self.state.images.get(&task_r.dst_image) {
                        // Check that we can extend without needing a barrier
                        match state.try_extend(&new_state) {
                            Some(new_state2) => new_state = new_state2,
                            None => return Err((task, false)),
                        }
                    }

                    // Checks completed
                    let task = task.unwrap(self.allocation_cache);
                    let task = match task {
                        EmulatorTask::CopyStagingToImage(task) => BBox::into_inner(task),
                        _ => panic!(),
                    };
                    object_cache.push_object(task.dst_image.clone());
                    object_cache.push_staging(task.staging_allocation);

                    self.recordable.push(Recordable::BufferToImageCopy {
                        src_buffer: task.staging_buffer,
                        dst_image: task.dst_image.get_handle(),
                        regions: task.copy_regions,
                    });
                    self.state.images.insert(task.dst_image, new_state);
                },
                EmulatorTask::CopyImageToStaging(task_r) => {
                    let mut new_state = ImageState::ReadUniform(vk::ImageLayout::TRANSFER_SRC_OPTIMAL, vk::PipelineStageFlags2::TRANSFER, vk::AccessFlags2::TRANSFER_READ);
                    if let Some(state) = self.state.images.get(&task_r.src_image) {
                        match state.try_extend(&new_state) {
                            Some(new_state2) => new_state = new_state2,
                            None => return Err((task, false)),
                        }
                    }

                    // Checks completed
                    let task = task.unwrap(self.allocation_cache);
                    let task = match task {
                        EmulatorTask::CopyImageToStaging(task) => BBox::into_inner(task),
                        _ => panic!(),
                    };
                    object_cache.push_object(task.src_image.clone());

                    self.recordable.push(Recordable::ImageToBufferCopy {
                        src_image: task.src_image.get_handle(),
                        dst_buffer: task.staging_buffer,
                        regions: task.copy_regions,
                    });
                    self.state.images.insert(task.src_image, new_state);
                },
                EmulatorTask::CopyBuffer(_) => todo!(),
                EmulatorTask::CopyBufferToImage(_) => todo!(),
                EmulatorTask::CopyImageToBuffer(_) => todo!(),
            }
            Ok(())
        }

        pub(super) fn record(&mut self, device: &DeviceContext, cmd: vk::CommandBuffer, mut pre_state: Option<SyncState>) -> SyncState {
            let mut buffer_barriers = Vec::new();
            let mut image_barriers = Vec::new();

            for (image, state) in &self.state.images {
                if pre_state.as_mut().map(|pre_state| pre_state.images.remove(image).map(|pre_state|
                    pre_state.gen_barriers(state, image, &mut image_barriers))
                ).flatten().is_none() {
                    let pre_layout = unsafe { image.get_current_layout() };
                    let pre_state = ImageState::ReadUniform(pre_layout, vk::PipelineStageFlags2::NONE, vk::AccessFlags2::NONE);
                    pre_state.gen_barriers(state, image, &mut image_barriers);
                }
            }

            if let Some(mut pre_state) = pre_state {
                for (buffer, state) in &self.state.buffers {
                    if let Some(pre_state) = pre_state.buffers.remove(buffer) {
                        pre_state.gen_barriers(state, buffer, &mut buffer_barriers);
                    }
                }
                for (buffer, state) in pre_state.buffers {
                    // Only those buffers which were not modified in this pass are left
                    if !self.state.buffers.insert(buffer, state).is_none() {
                        panic!("What?")
                    }
                }

                for (image, state) in pre_state.images {
                    // Only those images which were not modified in this pass are left
                    if !self.state.images.insert(image, state).is_none() {
                        panic!("What?")
                    }
                }
            }

            if !buffer_barriers.is_empty() || !image_barriers.is_empty() {
                let dependency_info = vk::DependencyInfo::builder()
                    .buffer_memory_barriers(&buffer_barriers)
                    .image_memory_barriers(&image_barriers);

                unsafe {
                    device.synchronization_2_khr().cmd_pipeline_barrier2(cmd, &dependency_info);
                }
            }

            for recordable in &self.recordable {
                recordable.record(device, cmd);
            }

            std::mem::replace(&mut self.state, SyncState::new())
        }
    }

    #[derive(Copy, Clone, Debug)]
    enum BufferState {
        ReadUniform(vk::PipelineStageFlags2, vk::AccessFlags2),
        ReadWriteUniform(vk::PipelineStageFlags2, vk::AccessFlags2),
    }

    impl BufferState {
        fn try_extend(&self, new: &BufferState) -> Option<BufferState> {
            // We can merge 2 accesses if they are both read only
            if let (BufferState::ReadUniform(stage_a, access_a), BufferState::ReadUniform(stage_b, access_b)) = (self, new) {
                Some(BufferState::ReadUniform(*stage_a | *stage_b, *access_a | *access_b))
            } else {
                None
            }
        }

        fn gen_barriers(&self, next: &BufferState, buffer: &Buffer, barriers: &mut Vec<vk::BufferMemoryBarrier2>) {
            match (self, next) {
                (BufferState::ReadUniform(stage_a, access_a), BufferState::ReadUniform(stage_b, access_b)) |
                (BufferState::ReadUniform(stage_a, access_a), BufferState::ReadWriteUniform(stage_b, access_b)) => {
                    if access_a != access_b {
                        barriers.push(vk::BufferMemoryBarrier2::builder()
                            .src_stage_mask(*stage_a)
                            .src_access_mask(vk::AccessFlags2::NONE)
                            .dst_stage_mask(*stage_b)
                            .dst_access_mask(*access_b)
                            .buffer(buffer.get_handle())
                            .offset(0)
                            .size(vk::WHOLE_SIZE)
                            .build());
                    }
                },
                (BufferState::ReadWriteUniform(stage_a, access_a), BufferState::ReadUniform(stage_b, access_b)) |
                (BufferState::ReadWriteUniform(stage_a, access_a), BufferState::ReadWriteUniform(stage_b, access_b)) => {
                    barriers.push(vk::BufferMemoryBarrier2::builder()
                        .src_stage_mask(*stage_a)
                        .src_access_mask(*access_a)
                        .dst_stage_mask(*stage_b)
                        .dst_access_mask(*access_b)
                        .buffer(buffer.get_handle())
                        .offset(0)
                        .size(vk::WHOLE_SIZE)
                        .build());
                }
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    enum ImageState {
        ReadUniform(vk::ImageLayout, vk::PipelineStageFlags2, vk::AccessFlags2),
        ReadWriteUniform(vk::ImageLayout, vk::PipelineStageFlags2, vk::AccessFlags2),
    }

    impl ImageState {
        fn try_extend(&self, new: &ImageState) -> Option<ImageState> {
            match (self, new) {
                (ImageState::ReadUniform(src_layout, src_stage, src_access), ImageState::ReadUniform(dst_layout, dst_stage, dst_access)) => {
                    if src_layout == dst_layout {
                        Some(ImageState::ReadUniform(*src_layout, *src_stage | *dst_stage, *src_access | *dst_access))
                    } else {
                        None
                    }
                },
                _ => None,
            }
        }

        fn gen_barriers(&self, next: &ImageState, image: &Image, barriers: &mut Vec<vk::ImageMemoryBarrier2>) {
            match (self, next) {
                (ImageState::ReadUniform(src_layout, src_stage, src_access), ImageState::ReadUniform(dst_layout, dst_stage, dst_access)) => {
                    if src_layout != dst_layout || !src_access.contains(*dst_access) {
                        barriers.push(Self::gen_uniform_barrier(image, *src_layout, *src_stage, vk::AccessFlags2::NONE, *dst_layout, *dst_stage, *dst_access));
                    }
                }
                (ImageState::ReadUniform(src_layout, src_stage, src_access), ImageState::ReadWriteUniform(dst_layout, dst_stage, dst_access)) |
                (ImageState::ReadWriteUniform(src_layout, src_stage, src_access), ImageState::ReadUniform(dst_layout, dst_stage, dst_access)) |
                (ImageState::ReadWriteUniform(src_layout, src_stage, src_access), ImageState::ReadWriteUniform(dst_layout, dst_stage, dst_access)) => {
                    barriers.push(Self::gen_uniform_barrier(image, *src_layout, *src_stage, *src_access, *dst_layout, *dst_stage, *dst_access));
                }
            }
        }

        fn gen_uniform_barrier(image: &Image, src_layout: vk::ImageLayout, src_stage: vk::PipelineStageFlags2, src_access: vk::AccessFlags2, dst_layout: vk::ImageLayout, dst_stage: vk::PipelineStageFlags2, dst_access: vk::AccessFlags2) -> vk::ImageMemoryBarrier2 {
            vk::ImageMemoryBarrier2::builder()
                .src_stage_mask(src_stage)
                .src_access_mask(src_access)
                .dst_stage_mask(dst_stage)
                .dst_access_mask(dst_access)
                .old_layout(src_layout)
                .new_layout(dst_layout)
                .image(image.get_handle())
                .subresource_range(image.get_info().get_full_subresource_range())
                .build()
        }
    }

    pub(super) struct SyncState {
        buffers: HashMap<Arc<Buffer>, BufferState>,
        images: HashMap<Arc<Image>, ImageState>,
    }

    impl SyncState {
        fn new() -> Self {
            Self {
                buffers: HashMap::new(),
                images: HashMap::new(),
            }
        }
    }

    enum Recordable<'a> {
        BufferCopy {
            src_buffer: vk::Buffer,
            dst_buffer: vk::Buffer,
            regions: BBox<'a, [vk::BufferCopy]>,
        },
        BufferToImageCopy {
            src_buffer: vk::Buffer,
            dst_image: vk::Image,
            regions: BBox<'a, [vk::BufferImageCopy]>,
        },
        ImageToBufferCopy {
            src_image: vk::Image,
            dst_buffer: vk::Buffer,
            regions: BBox<'a, [vk::BufferImageCopy]>,
        }
    }

    impl<'a> Recordable<'a> {
        fn record(&self, device: &DeviceContext, cmd: vk::CommandBuffer) {
            match self {
                Recordable::BufferCopy { src_buffer, dst_buffer, regions } => {
                    unsafe {
                        device.vk().cmd_copy_buffer(cmd, *src_buffer, *dst_buffer, &regions)
                    }
                }
                Recordable::BufferToImageCopy { src_buffer, dst_image, regions } => {
                    unsafe {
                        device.vk().cmd_copy_buffer_to_image(cmd, *src_buffer, *dst_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &regions)
                    }
                }
                Recordable::ImageToBufferCopy { src_image, dst_buffer, regions } => {
                    unsafe {
                        device.vk().cmd_copy_image_to_buffer(cmd, *src_image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, *dst_buffer, &regions)
                    }
                }
            }
        }
    }
}

use recorder::{Recorder, SubmissionArtifact};

struct RenderPassState {
    device: Arc<DeviceContext>,
    framebuffer_state: FramebufferState,
    current_pipeline: Option<(u64, GraphicsPipelineId)>,
    pipeline_state: PipelineStateTracker,
}

impl RenderPassState {
    fn new(device: Arc<DeviceContext>, cmd: vk::CommandBuffer, framebuffer_state: FramebufferState) -> Self {
        let mut info = vk::RenderingInfo::builder()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: framebuffer_state.render_area.0.x as i32, y: framebuffer_state.render_area.0.y as i32 },
                extent: vk::Extent2D { width: framebuffer_state.render_area.1.x, height: framebuffer_state.render_area.1.y }
            })
            .layer_count(1)
            .view_mask(0);

        let attachments: Box<[_]> = framebuffer_state.color_attachments.iter().map(|a| {
            vk::RenderingAttachmentInfo::builder()
                .image_view(a.get_default_view_handle())
                .image_layout(vk::ImageLayout::GENERAL)
                .resolve_mode(vk::ResolveModeFlags::NONE)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)
                .build()
        }).collect();

        info = info.color_attachments(&attachments);

        let depth_attachment;
        if let Some(depth_image) = &framebuffer_state.depth_attachment {
            depth_attachment = vk::RenderingAttachmentInfo::builder()
                .image_view(depth_image.get_default_view_handle())
                .image_layout(vk::ImageLayout::GENERAL)
                .resolve_mode(vk::ResolveModeFlags::NONE)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE);

            info = info.depth_attachment(&depth_attachment);
        }

        let stencil_attachment;
        if let Some(stencil_image) = &framebuffer_state.stencil_attachment {
            stencil_attachment = vk::RenderingAttachmentInfo::builder()
                .image_view(stencil_image.get_default_view_handle())
                .image_layout(vk::ImageLayout::GENERAL)
                .resolve_mode(vk::ResolveModeFlags::NONE)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE);

            info = info.stencil_attachment(&stencil_attachment);
        }

        unsafe {
            device.dynamic_rendering_khr().unwrap().cmd_begin_rendering(cmd, &info);
        }

        Self {
            device,
            framebuffer_state,
            current_pipeline: None,
            pipeline_state: PipelineStateTracker::new(),
        }
    }

    fn is_compatible(&self, framebuffer_state: &FramebufferState) -> bool {
        &self.framebuffer_state == framebuffer_state
    }

    fn setup_pipeline(&mut self, cmd: vk::CommandBuffer, pipeline: (u64, GraphicsPipelineId), handle: vk::Pipeline, input_attributes: &[PipelineInputAttribute], pipeline_state: &PipelineState, capabilities: &DynamicStateCapabilities) {
        if self.current_pipeline != Some(pipeline) {
            unsafe {
                self.device.vk().cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, handle);
            }
            self.current_pipeline = Some(pipeline);
        }

        let mut buffers = Vec::with_capacity(input_attributes.len());
        let mut offsets = Vec::with_capacity(input_attributes.len());
        let mut strides = Vec::with_capacity(input_attributes.len());
        for attribute in input_attributes {
            buffers.push(attribute.buffer.get_handle());
            offsets.push(attribute.offset);
            strides.push(attribute.stride as vk::DeviceSize);
        }

        if capabilities.extended_dynamic_state {
            unsafe {
                self.device.extended_dynamic_state_ext().unwrap().cmd_bind_vertex_buffers2(cmd, 0, &buffers, &offsets, None, Some(&strides));
            }
        } else {
            unsafe {
                self.device.vk().cmd_bind_vertex_buffers(cmd, 0, &buffers, &offsets);
            }
        }

        self.pipeline_state.update_dynamic_state(&self.device, cmd, pipeline_state, capabilities);
    }

    fn complete(self, cmd: vk::CommandBuffer) {
        unsafe {
            self.device.dynamic_rendering_khr().unwrap().cmd_end_rendering(cmd);
        }
    }
}

struct DynamicStateCapabilities {
    extended_dynamic_state: bool,
    extended_dynamic_state_2: bool,
}

struct PipelineStateTracker {
    primitive_topology: Option<vk::PrimitiveTopology>,
    primitive_restart_enable: Option<bool>,
    cull_mode: Option<vk::CullModeFlags>,
    front_face: Option<vk::FrontFace>,
    line_width: Option<NotNan<f32>>,
    depth_test_enable: Option<bool>,
    depth_write_enable: Option<bool>,
    depth_compare_op: Option<vk::CompareOp>,
    stencil_test_enable: Option<bool>,
    stencil_op: Option<(PipelineDynamicStateStencilOp, PipelineDynamicStateStencilOp)>,
    stencil_compare_mask: Option<(u32, u32)>,
    stencil_write_mask: Option<(u32, u32)>,
    stencil_reference: Option<(u32, u32)>,
    blend_constants: Option<[NotNan<f32>; 4]>,
}

impl PipelineStateTracker {
    fn new() -> Self {
        Self {
            primitive_topology: None,
            primitive_restart_enable: None,
            cull_mode: None,
            front_face: None,
            line_width: None,
            depth_test_enable: None,
            depth_write_enable: None,
            depth_compare_op: None,
            stencil_test_enable: None,
            stencil_op: None,
            stencil_compare_mask: None,
            stencil_write_mask: None,
            stencil_reference: None,
            blend_constants: None
        }
    }

    fn update_dynamic_state(&mut self, device: &DeviceContext, cmd: vk::CommandBuffer, pipeline_state: &PipelineState, capabilities: &DynamicStateCapabilities) {
        let vk = device.vk();

        let viewport = vk::Viewport {
            x: pipeline_state.viewport.0.x,
            y: pipeline_state.viewport.0.y,
            width: pipeline_state.viewport.1.x,
            height: pipeline_state.viewport.1.y,
            min_depth: 0.0,
            max_depth: 1.0
        };
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: pipeline_state.scissor.0.x as i32, y: pipeline_state.scissor.0.y as i32 },
            extent: vk::Extent2D { width: pipeline_state.scissor.1.x, height: pipeline_state.scissor.1.y }
        };
        unsafe {
            vk.cmd_set_viewport(cmd, 0, std::slice::from_ref(&viewport));
            vk.cmd_set_scissor(cmd, 0, std::slice::from_ref(&scissor));
        }
        let line_width = NotNan::new(pipeline_state.line_width).unwrap();
        if self.line_width != Some(line_width) {
            unsafe {
                vk.cmd_set_line_width(cmd, pipeline_state.line_width);
            }
            self.line_width = Some(line_width);
        }

        let blend_constants = [NotNan::new(1f32).unwrap(); 4];
        if self.blend_constants != Some(blend_constants) {
            unsafe {
                // Safe because NotNan is repr(transparent)
                let blend_constants: [f32; 4] = std::mem::transmute(blend_constants);
                vk.cmd_set_blend_constants(cmd, &blend_constants);
            }
            self.blend_constants = Some(blend_constants);
        }

        if let Some((front, back)) = &pipeline_state.stencil_test {
            if self.stencil_compare_mask != Some((front.compare_mask, back.compare_mask)) {
                unsafe {
                    vk.cmd_set_stencil_compare_mask(cmd, vk::StencilFaceFlags::FRONT, front.compare_mask);
                    vk.cmd_set_stencil_compare_mask(cmd, vk::StencilFaceFlags::BACK, back.compare_mask);
                }
                self.stencil_compare_mask = Some((front.compare_mask, back.compare_mask));
            }
            if self.stencil_write_mask != Some((front.write_mask, back.write_mask)) {
                unsafe {
                    vk.cmd_set_stencil_write_mask(cmd, vk::StencilFaceFlags::FRONT, front.write_mask);
                    vk.cmd_set_stencil_write_mask(cmd, vk::StencilFaceFlags::BACK, back.write_mask);
                }
                self.stencil_write_mask = Some((front.write_mask, back.write_mask));
            }
            if self.stencil_reference != Some((front.reference, back.reference)) {
                unsafe {
                    vk.cmd_set_stencil_reference(cmd, vk::StencilFaceFlags::FRONT, front.reference);
                    vk.cmd_set_stencil_reference(cmd, vk::StencilFaceFlags::BACK, back.reference);
                }
                self.stencil_reference = Some((front.reference, back.reference));
            }
        }

        if capabilities.extended_dynamic_state {
            let vk = device.get_functions().extended_dynamic_state_ext.as_ref().unwrap();

            if self.primitive_topology != Some(pipeline_state.primitive_topology) {
                unsafe {
                    vk.cmd_set_primitive_topology(cmd, pipeline_state.primitive_topology);
                }
                self.primitive_topology = Some(pipeline_state.primitive_topology);
            }

            if self.cull_mode != Some(pipeline_state.cull_mode) {
                unsafe {
                    vk.cmd_set_cull_mode(cmd, pipeline_state.cull_mode);
                }
                self.cull_mode = Some(pipeline_state.cull_mode);
            }

            if self.front_face != Some(pipeline_state.front_face) {
                unsafe {
                    vk.cmd_set_front_face(cmd, pipeline_state.front_face);
                }
                self.front_face = Some(pipeline_state.front_face);
            }

            let depth_test_enable = pipeline_state.depth_test.is_some();
            if self.depth_test_enable != Some(depth_test_enable) {
                unsafe {
                    vk.cmd_set_depth_test_enable(cmd, depth_test_enable)
                }
                self.depth_test_enable = Some(depth_test_enable);
            }

            if let Some(depth_compare_op) = &pipeline_state.depth_test {
                if self.depth_compare_op != Some(*depth_compare_op) {
                    unsafe {
                        vk.cmd_set_depth_compare_op(cmd, *depth_compare_op);
                    }
                    self.depth_compare_op = Some(*depth_compare_op);
                }
            }

            if self.depth_write_enable != Some(pipeline_state.depth_write_enable) {
                unsafe {
                    vk.cmd_set_depth_write_enable(cmd, pipeline_state.depth_write_enable);
                }
                self.depth_write_enable = Some(pipeline_state.depth_write_enable);
            }

            let stencil_test_enable = pipeline_state.stencil_test.is_some();
            if self.stencil_test_enable != Some(stencil_test_enable) {
                unsafe {
                    vk.cmd_set_stencil_test_enable(cmd, stencil_test_enable);
                }
                self.stencil_test_enable = Some(stencil_test_enable);
            }

            if let Some((front, back)) = &pipeline_state.stencil_test {
                let front = PipelineDynamicStateStencilOp {
                    fail_op: front.fail_op,
                    pass_op: front.pass_op,
                    depth_fail_op: front.depth_fail_op,
                    compare_op: front.compare_op
                };
                let back = PipelineDynamicStateStencilOp {
                    fail_op: back.fail_op,
                    pass_op: back.pass_op,
                    depth_fail_op: back.depth_fail_op,
                    compare_op: back.compare_op
                };
                if self.stencil_op != Some((front, back)) {
                    unsafe {
                        vk.cmd_set_stencil_op(cmd, vk::StencilFaceFlags::FRONT, front.fail_op, front.pass_op, front.depth_fail_op, front.compare_op);
                        vk.cmd_set_stencil_op(cmd, vk::StencilFaceFlags::BACK, back.fail_op, back.pass_op, back.depth_fail_op, back.compare_op);
                    }
                    self.stencil_op = Some((front, back));
                }
            }
        } else {
            self.cull_mode = None;
            self.depth_test_enable = None;
            self.depth_write_enable = None;
            self.front_face = None;
            self.primitive_topology = None;
            self.stencil_op = None;
            self.stencil_test_enable = None;
        }

        if capabilities.extended_dynamic_state_2 {
            let vk = device.get_functions().extended_dynamic_state_2_ext.as_ref().unwrap();

            if self.primitive_restart_enable != Some(false) {
                unsafe {
                    vk.cmd_set_primitive_restart_enable(cmd, false);
                }
                self.primitive_restart_enable = Some(false);
            }
        } else {
            self.primitive_restart_enable = None;
        }
    }
}

struct PipelineCache {
    share: Arc<Share2>,
    pipelines: HashMap<GraphicsPipelineId, PipelineInstanceCache>,
    hash_builder: RandomState,
    bump: Bump,
}

/// Utility function that returns [`None`] if b is true and [`Some`]`(value)` otherwise.
fn false_then<T>(b: bool, value: T) -> Option<T> {
    if b {
        None
    } else {
        Some(value)
    }
}

impl PipelineCache {
    fn new(share: Arc<Share2>) -> Self {
        Self {
            share,
            pipelines: HashMap::new(),
            hash_builder: RandomState::new(),
            bump: Bump::new()
        }
    }

    fn get_or_create_instance_from_capabilities(&mut self, pipeline: &GraphicsPipeline, input_assembly: &[PipelineInputAttribute], pipeline_state: &PipelineState, capabilities: &DynamicStateCapabilities) -> (u64, vk::Pipeline) {
        let input_attributes: Box<[_]> = input_assembly.iter().enumerate().map(|(index, input)| {
            (index as u32, input.format, input.input_rate)
        }).collect();

        let input_binding_strides: Option<Box<[_]>> = if capabilities.extended_dynamic_state {
            Some(input_assembly.iter().map(|input| input.stride).collect())
        } else {
            None
        };

        let blend_attachments: Box<[_]> = pipeline_state.color_blend.iter().map(|color_blend| {
            (color_blend.clone(), vk::ColorComponentFlags::RGBA)
        }).collect();

        let (stencil_front, stencil_back) = pipeline_state.stencil_test.unwrap_or_default();
        let stencil_op = (
            PipelineDynamicStateStencilOp {
                fail_op: stencil_front.fail_op,
                pass_op: stencil_front.pass_op,
                depth_fail_op: stencil_front.depth_fail_op,
                compare_op: stencil_front.compare_op
            }, PipelineDynamicStateStencilOp {
                fail_op: stencil_back.fail_op,
                pass_op: stencil_back.pass_op,
                depth_fail_op: stencil_back.depth_fail_op,
                compare_op: stencil_back.compare_op
        });
        // let stencil_compare_mask = (stencil_front.compare_mask, stencil_back.compare_mask);
        // let stencil_write_mask = (stencil_front.write_mask, stencil_back.write_mask);
        // let stencil_reference = (stencil_front.reference, stencil_back.reference);

        let state = PipelineDynamicState {
            input_attributes: input_attributes.as_ref(),
            input_binding_strides: input_binding_strides.as_ref().map(|b| b.as_ref()),
            primitive_topology: false_then(capabilities.extended_dynamic_state, pipeline_state.primitive_topology),
            primitive_restart_enable: false_then(capabilities.extended_dynamic_state_2, false),
            depth_clamp_enable: false,
            rasterizer_discard_enable: false,
            polygon_mode: pipeline_state.polygon_mode,
            cull_mode: false_then(capabilities.extended_dynamic_state, pipeline_state.cull_mode),
            front_face: false_then(capabilities.extended_dynamic_state, pipeline_state.front_face),
            line_width: Some(NotNan::new(pipeline_state.line_width).unwrap()),
            depth_test_enable: false_then(capabilities.extended_dynamic_state, pipeline_state.depth_test.is_some()),
            depth_write_enable: false_then(capabilities.extended_dynamic_state, pipeline_state.depth_write_enable),
            depth_compare_op: false_then(capabilities.extended_dynamic_state, pipeline_state.depth_test.unwrap_or_default()),
            stencil_test_enable: false_then(capabilities.extended_dynamic_state, pipeline_state.stencil_test.is_some()),
            stencil_op: false_then(capabilities.extended_dynamic_state, stencil_op),
            stencil_compare_mask: None,
            stencil_write_mask: None,
            stencil_reference: None,
            logic_op: Some(vk::LogicOp::default()),
            blend_attachments: blend_attachments.as_ref(),
            blend_constants: None,
            render_pass: None
        };

        self.get_or_create_instance(pipeline, &state)
    }

    fn get_or_create_instance(&mut self, pipeline: &GraphicsPipeline, state: &PipelineDynamicState) -> (u64, vk::Pipeline) {
        let result = if let Some(cache) = self.pipelines.get_mut(&pipeline.get_id()) {
            cache.get_or_create_instance(&mut self.bump, pipeline, state, &mut self.hash_builder)
        } else {
            let mut instance = PipelineInstanceCache::new(self.share.get_device().clone());
            let result = instance.get_or_create_instance(&mut self.bump, pipeline, state, &mut self.hash_builder);
            self.pipelines.insert(pipeline.get_id(), instance);
            result
        };
        self.bump.reset();
        result
    }

    /// Destroys all instances for a specific pipeline
    fn clear_instances(&mut self, id: GraphicsPipelineId) {
        self.pipelines.remove(&id);
    }
}

mod object_state {
    use std::collections::HashMap;
    use std::sync::Arc;
    use ash::vk;
    use crate::prelude::*;
    use crate::renderer::emulator::{Buffer, FramebufferState, GraphicsPipeline, Image, PipelineInputAttribute, PipelineState};

    pub(super) struct RenderPassRecorder {
        framebuffer: FramebufferState,
        draws: Vec<DrawEntry>,
        buffer_state: HashMap<Arc<Buffer>, (Option<BufferState>, BufferState)>,
        image_state: HashMap<Arc<Image>, (Option<ImageState>, ImageState)>,
    }

    impl RenderPassRecorder {
        pub(super) fn new(framebuffer: FramebufferState) -> Self {
            Self {
                framebuffer,
                draws: Vec::new(),
                buffer_state: HashMap::new(),
                image_state: HashMap::new(),
            }
        }

        pub(super) fn draw(&mut self, pipeline: Arc<GraphicsPipeline>, handle: vk::Pipeline, input_attributes: &[PipelineInputAttribute], state: PipelineState, draw_count: u32) {
            let (binding_buffers, binding_offsets, binding_strides) = self.process_input_attributes(input_attributes);

            self.draws.push(DrawEntry::Draw {
                pipeline,
                pipeline_handle: handle,
                binding_buffers,
                binding_offsets,
                binding_strides,
                state,
                draw_count
            });
        }

        pub(super) fn draw_indexed(&mut self) {
            todo!()
        }

        pub(super) fn end(self, device: &DeviceContext, cmd: vk::CommandBuffer) {
            todo!()
        }

        fn process_input_attributes(&mut self, input_attributes: &[PipelineInputAttribute]) -> (Box<[vk::Buffer]>, Box<[vk::DeviceSize]>, Box<[vk::DeviceSize]>) {
            let mut buffers = Vec::with_capacity(input_attributes.len());
            let mut offsets = Vec::with_capacity(input_attributes.len());
            let mut strides = Vec::with_capacity(input_attributes.len());
            for attr in input_attributes {
                buffers.push(attr.buffer.get_handle());
                offsets.push(attr.offset);
                strides.push(attr.stride as vk::DeviceSize);

                if let Some((_, state)) = self.buffer_state.get_mut(&attr.buffer) {
                    if state.has_write() {
                        panic!("Buffer used as vertx input attribute source must not have pending writes within a render pass");
                    }
                    state.stage_mask |= vk::PipelineStageFlags2::VERTEX_ATTRIBUTE_INPUT;
                    state.access_mask |= vk::AccessFlags2::VERTEX_ATTRIBUTE_READ;
                } else {
                    self.buffer_state.insert(attr.buffer.clone(), (None, BufferState {
                        stage_mask: vk::PipelineStageFlags2::VERTEX_ATTRIBUTE_INPUT,
                        access_mask: vk::AccessFlags2::VERTEX_ATTRIBUTE_READ,
                    }));
                }
            }

            (buffers.into_boxed_slice(), offsets.into_boxed_slice(), strides.into_boxed_slice())
        }
    }

    enum DrawEntry {
        Draw {
            pipeline: Arc<GraphicsPipeline>,
            pipeline_handle: vk::Pipeline,
            binding_buffers: Box<[vk::Buffer]>,
            binding_offsets: Box<[vk::DeviceSize]>,
            binding_strides: Box<[vk::DeviceSize]>,
            state: PipelineState,
            draw_count: u32,
        },
    }

    struct BufferState {
        stage_mask: vk::PipelineStageFlags2,
        access_mask: vk::AccessFlags2,
    }

    impl BufferState {
        fn has_write(&self) -> bool {
            todo!()
        }
    }

    struct ImageState {
        stage_mask: vk::PipelineStageFlags2,
        access_mask: vk::AccessFlags2,
        layout: vk::ImageLayout,
    }


    pub(super) struct BarrierCache {
        buffer_barriers: Vec<vk::BufferMemoryBarrier2>,
        image_barriers: Vec<vk::ImageMemoryBarrier2>,
    }

    impl BarrierCache {
        /// The maximum number of barriers of each type that can be submitted per call to
        /// VkCmdPipelineBarrier2. (Yes during testing we did hit crashes because we submitted too
        /// many barriers at once. For that specific case it was ~4000 buffer barriers).
        const MAX_BARRIERS_PER_SUBMIT: usize = 256;

        pub(super) fn new() -> Self {
            BarrierCache {
                buffer_barriers: Vec::new(),
                image_barriers: Vec::new(),
            }
        }

        pub(super) fn push_buffer_barrier(&mut self, barrier: vk::BufferMemoryBarrier2) {
            self.buffer_barriers.push(barrier);
        }

        pub(super) fn push_image_barrier(&mut self, barrier: vk::ImageMemoryBarrier2) {
            self.image_barriers.push(barrier);
        }

        pub(super) fn clear(&mut self) {
            self.buffer_barriers.clear();
            self.image_barriers.clear();
        }

        pub(super) fn submit_and_clear(&mut self, device: &DeviceContext, cmd: vk::CommandBuffer) {
            // Make sure we never submit more than MAX_BARRIERS_PER_SUBMIT at a time
            let mut offset = 0usize;
            while (offset < self.buffer_barriers.len()) || (offset < self.image_barriers.len()) {
                let mut info = vk::DependencyInfo::builder();
                if offset < self.buffer_barriers.len() {
                    let end = offset + std::cmp::min(self.buffer_barriers.len() - offset, Self::MAX_BARRIERS_PER_SUBMIT);
                    info = info.buffer_memory_barriers(&self.buffer_barriers[offset..end]);
                }
                if offset < self.image_barriers.len() {
                    let end = offset + std::cmp::min(self.image_barriers.len() - offset, Self::MAX_BARRIERS_PER_SUBMIT);
                    info = info.image_memory_barriers(&self.image_barriers[offset..end]);
                }
                unsafe {
                    device.synchronization_2_khr().cmd_pipeline_barrier2(cmd, &info)
                }
                offset += Self::MAX_BARRIERS_PER_SUBMIT;
            }
            self.clear();
        }
    }
}




















pub(super) enum WorkerTask {
    StartPass(PassId, Arc<dyn EmulatorPipeline>, Box<dyn EmulatorPipelinePass + Send>, Arc<GlobalImage>, vk::Sampler),
    EndPass(Box<ImmediateBuffer>),
    UseGlobalMesh(Arc<GlobalMesh>),
    UseGlobalImage(Arc<GlobalImage>),
    UseShader(ShaderId),
    UseOutput(Box<dyn EmulatorOutput + Send>),
    PipelineTask(PipelineTask),
    WriteGlobalMesh(GlobalMeshWrite, bool),
    ClearGlobalImage(GlobalImageClear, bool),
    WriteGlobalImage(GlobalImageWrite),
    GenerateGlobalImageMipmaps(Arc<GlobalImage>, PassId),
}

pub(super) struct GlobalMeshWrite {
    pub(super) after_pass: PassId,
    pub(super) staging_allocation: StagingAllocationId,
    pub(super) staging_range: (vk::DeviceSize, vk::DeviceSize),
    pub(super) staging_buffer: vk::Buffer,
    pub(super) dst_mesh: Arc<GlobalMesh>,
    pub(super) regions: Box<[vk::BufferCopy]>,
}

pub(super) struct GlobalImageWrite {
    pub(super) after_pass: PassId,
    pub(super) staging_allocation: StagingAllocationId,
    pub(super) staging_range: (vk::DeviceSize, vk::DeviceSize),
    pub(super) staging_buffer: vk::Buffer,
    pub(super) dst_image: Arc<GlobalImage>,
    pub(super) regions: Box<[vk::BufferImageCopy]>,
}

pub(super) struct GlobalImageClear {
    pub(super) after_pass: PassId,
    pub(super) clear_value: vk::ClearColorValue,
    pub(super) dst_image: Arc<GlobalImage>,
}

pub(super) fn run_worker(device: Arc<DeviceContext>, share: Arc<Share>) {
    let queue = device.get_main_queue();

    let pool = Rc::new(RefCell::new(WorkerObjectPool::new(device.clone(), queue.get_queue_family_index())));
    let mut current_pass: Option<PassState> = None;
    let mut old_frames = Vec::new();

    // A global objects recorder submitted before the current frame.
    // If no active pass exits this **must** be [`None`].
    let mut current_global_recorder: Option<GlobalObjectsRecorder> = None;
    // A global objects recorder submitted before the next frame.
    // When a pass is started this object is moved to `current_global_recorder`.
    let mut next_global_recorder: Option<GlobalObjectsRecorder> = None;

    let queue = device.get_main_queue();

    loop {
        old_frames.retain(|old: &PassState| {
            !old.is_complete()
        });

        let task = match share.try_get_next_task_timeout(Duration::from_micros(500)) {
            NextTaskResult::Ok(task) => task,
            NextTaskResult::Timeout => continue,
        };

        match task {
            WorkerTask::StartPass(id, pipeline, pass, placeholder_image, placeholder_sampler) => {
                if current_pass.is_some() {
                    log::error!("Worker received WorkerTask::StartPass when a pass is already running");
                    panic!()
                }
                let state = PassState::new(id, pipeline, pass, device.clone(), &queue, share.clone(), pool.clone(), placeholder_image, placeholder_sampler);
                current_pass = Some(state);
                current_global_recorder = next_global_recorder.take();
            }

            WorkerTask::EndPass(immediate_buffer) => {
                if let Some(mut pass) = current_pass.take() {
                    pass.use_immediate_buffer(immediate_buffer);
                    pass.submit(&queue, current_global_recorder.take());
                    old_frames.push(pass);
                } else {
                    log::error!("Worker received WorkerTask::EndPass when no active pass exists");
                    panic!()
                }
            }

            WorkerTask::UseGlobalMesh(mesh) => {
                if let Some(pass) = &mut current_pass {
                    pass.global_meshes.push(mesh)
                } else {
                    log::error!("Worker received WorkerTask::UseStaticMesh when no active pass exists");
                    panic!()
                }
            }

            WorkerTask::UseGlobalImage(image) => {
                if let Some(pass) = &mut current_pass {
                    pass.global_images.push(image);
                } else {
                    log::error!("Worker received WorkerTask::UseStaticImage when no active pass exits");
                    panic!()
                }
            }

            WorkerTask::UseShader(shader) => {
                if let Some(pass) = &mut current_pass {
                    pass.shaders.push(shader);
                } else {
                    log::error!("Worker received WorkerTask::UseShader when no active pass exists");
                    panic!()
                }
            }

            WorkerTask::UseOutput(output) => {
                if let Some(pass) = &mut current_pass {
                    pass.use_output(output);
                } else {
                    log::error!("Worker received WorkerTask::UseOutput when no active pass exists");
                    panic!()
                }
            }

            WorkerTask::PipelineTask(task) => {
                if let Some(pass) = &mut current_pass {
                    pass.process_task(&task)
                } else {
                    log::error!("Worker received WorkerTask::PipelineTask when no active pass exists");
                    panic!()
                }
            }

            WorkerTask::WriteGlobalMesh(write, uninit) => {
                if let Some(current_pass) = &current_pass {
                    if current_pass.pass_id > write.after_pass {
                        get_or_create_recorder(&mut current_global_recorder, &share, &pool).record_global_buffer_write(write, uninit);
                    } else {
                        get_or_create_recorder(&mut next_global_recorder, &share, &pool).record_global_buffer_write(write, uninit);
                    }
                } else {
                    get_or_create_recorder(&mut next_global_recorder, &share, &pool).record_global_buffer_write(write, uninit);
                }
            }

            WorkerTask::ClearGlobalImage(clear, uninit) => {
                if let Some(current_pass) = &current_pass {
                    if current_pass.pass_id > clear.after_pass {
                        get_or_create_recorder(&mut current_global_recorder, &share, &pool).record_global_image_clear(clear, uninit);
                    } else {
                        get_or_create_recorder(&mut next_global_recorder, &share, &pool).record_global_image_clear(clear, uninit);
                    }
                } else {
                    get_or_create_recorder(&mut next_global_recorder, &share, &pool).record_global_image_clear(clear, uninit);
                }
            }

            WorkerTask::WriteGlobalImage(write) => {
                if let Some(current_pass) = &current_pass {
                    if current_pass.pass_id > write.after_pass {
                        get_or_create_recorder(&mut current_global_recorder, &share, &pool).record_global_image_write(write, false);
                    } else {
                        get_or_create_recorder(&mut next_global_recorder, &share, &pool).record_global_image_write(write, false);
                    }
                } else {
                    get_or_create_recorder(&mut next_global_recorder, &share, &pool).record_global_image_write(write, false);
                }
            }

            WorkerTask::GenerateGlobalImageMipmaps(image, after_pass) => {
                if let Some(current_pass) = &current_pass {
                    if current_pass.pass_id > after_pass {
                        get_or_create_recorder(&mut current_global_recorder, &share, &pool).record_global_image_generate_mipmaps(image);
                    } else {
                        get_or_create_recorder(&mut next_global_recorder, &share, &pool).record_global_image_generate_mipmaps(image);
                    }
                } else {
                    get_or_create_recorder(&mut next_global_recorder, &share, &pool).record_global_image_generate_mipmaps(image);
                }
            }
        }
    }
}

fn get_or_create_recorder<'a>(recorder: &'a mut Option<GlobalObjectsRecorder>, share: &Arc<Share>, object_pool: &Rc<RefCell<WorkerObjectPool>>) -> &'a mut GlobalObjectsRecorder {
    if let Some(recorder) = recorder {
        recorder
    } else {
        *recorder = Some(GlobalObjectsRecorder::new(share.clone(), object_pool.clone()));
        recorder.as_mut().unwrap()
    }
}

struct WorkerObjectPool {
    device: Arc<DeviceContext>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    fences: Vec<vk::Fence>,
}

impl WorkerObjectPool {
    fn new(device: Arc<DeviceContext>, queue_family: u32) -> Self {
        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue_family);

        let command_pool = unsafe {
            device.vk().create_command_pool(&info, None)
        }.unwrap();

        Self {
            device,
            command_pool,
            command_buffers: Vec::new(),
            fences: Vec::new(),
        }
    }

    fn get_buffer(&mut self) -> vk::CommandBuffer {
        if self.command_buffers.is_empty() {
            let info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(8);

            let buffers = unsafe {
                self.device.vk().allocate_command_buffers(&info)
            }.unwrap();

            self.command_buffers.extend(buffers);
        }

        self.command_buffers.pop().unwrap()
    }

    fn return_buffer(&mut self, buffer: vk::CommandBuffer) {
        self.command_buffers.push(buffer)
    }

    fn return_buffers(&mut self, buffers: &[vk::CommandBuffer]) {
        self.command_buffers.extend_from_slice(buffers);
    }

    fn get_fence(&mut self) -> vk::Fence {
        if self.fences.is_empty() {
            let info = vk::FenceCreateInfo::builder();

            let fence = unsafe {
                self.device.vk().create_fence(&info, None)
            }.unwrap();

            return fence;
        }

        self.fences.pop().unwrap()
    }

    fn return_fence(&mut self, fence: vk::Fence) {
        self.fences.push(fence);
    }
}

pub struct PooledObjectProvider {
    share: Arc<Share>,
    pool: Rc<RefCell<WorkerObjectPool>>,
    used_buffers: Vec<vk::CommandBuffer>,
    used_fences: Vec<vk::Fence>,
}

impl PooledObjectProvider {
    fn new(share: Arc<Share>, pool: Rc<RefCell<WorkerObjectPool>>) -> Self {
        Self {
            share,
            pool,
            used_buffers: Vec::with_capacity(8),
            used_fences: Vec::with_capacity(4),
        }
    }

    pub fn get_command_buffer(&mut self) -> vk::CommandBuffer {
        let buffer = self.pool.borrow_mut().get_buffer();
        self.used_buffers.push(buffer);

        buffer
    }

    pub fn get_begin_command_buffer(&mut self) -> VkResult<vk::CommandBuffer> {
        let cmd = self.get_command_buffer();

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.pool.borrow().device.vk().begin_command_buffer(cmd, &info)
        }?;

        Ok(cmd)
    }

    pub fn get_fence(&mut self) -> vk::Fence {
        let fence = self.pool.borrow_mut().get_fence();
        self.used_fences.push(fence);

        fence
    }

    pub fn allocate_uniform(&mut self, data: &[u8]) -> (vk::Buffer, vk::DeviceSize) {
        self.share.allocate_uniform(data)
    }
}

impl Drop for PooledObjectProvider {
    fn drop(&mut self) {
        self.pool.borrow_mut().return_buffers(self.used_buffers.as_slice());
    }
}

pub struct SubmitRecorder<'a> {
    submits: Vec<vk::SubmitInfo2>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> SubmitRecorder<'a> {
    fn new(capacity: usize) -> Self {
        Self {
            submits: Vec::with_capacity(capacity),
            _phantom: PhantomData,
        }
    }

    pub fn push(&mut self, submit: vk::SubmitInfo2Builder<'a>) {
        self.submits.push(submit.build());
    }

    fn as_slice(&self) -> &[vk::SubmitInfo2] {
        self.submits.as_slice()
    }
}

struct PassState {
    share: Arc<Share>,
    device: Arc<DeviceContext>,
    object_pool: PooledObjectProvider,

    pass_id: PassId,

    pipeline: Arc<dyn EmulatorPipeline>,
    pass: Box<dyn EmulatorPipelinePass>,
    outputs: Vec<Box<dyn EmulatorOutput>>,

    immediate_buffer: Option<Box<ImmediateBuffer>>,
    global_meshes: Vec<Arc<GlobalMesh>>,
    global_images: Vec<Arc<GlobalImage>>,
    shaders: Vec<ShaderId>,

    pre_cmd: vk::CommandBuffer,
    post_cmd: vk::CommandBuffer,

    end_fence: Option<vk::Fence>,

    gob: Option<GlobalObjectsRecorder>,
}

impl PassState {
    fn new(
        pass_id: PassId,
        pipeline: Arc<dyn EmulatorPipeline>,
        mut pass: Box<dyn EmulatorPipelinePass>,
        device: Arc<DeviceContext>,
        queue: &Queue,
        share: Arc<Share>,
        pool: Rc<RefCell<WorkerObjectPool>>,
        placeholder_image: Arc<GlobalImage>,
        placeholder_sampler: vk::Sampler
    ) -> Self {
        let mut object_pool = PooledObjectProvider::new(share.clone(), pool);

        let pre_cmd = object_pool.get_begin_command_buffer().unwrap();
        let post_cmd = object_pool.get_begin_command_buffer().unwrap();

        pass.init(queue, &mut object_pool, placeholder_image.get_sampler_view(), placeholder_sampler);

        Self {
            share,
            device,
            object_pool,

            pass_id,

            pipeline,
            pass,
            outputs: Vec::with_capacity(8),

            immediate_buffer: None,
            global_meshes: Vec::new(),
            global_images: vec![placeholder_image],
            shaders: Vec::new(),

            pre_cmd,
            post_cmd,

            end_fence: None,
            gob: None
        }
    }

    fn use_immediate_buffer(&mut self, immediate_buffer: Box<ImmediateBuffer>) {
        if self.immediate_buffer.is_some() {
            log::error!("Called PassState::use_immediate_buffer when a immediate buffer already exists");
            panic!()
        }

        immediate_buffer.generate_copy_commands(self.pre_cmd);
        self.immediate_buffer = Some(immediate_buffer);
    }

    fn use_output(&mut self, mut output: Box<dyn EmulatorOutput>) {
        output.init(self.pass.as_ref(), &mut self.object_pool);
        self.outputs.push(output);
    }

    fn process_task(&mut self, task: &PipelineTask) {
        self.pass.process_task(task, &mut self.object_pool);
    }

    fn submit(&mut self, queue: &Queue, gob: Option<GlobalObjectsRecorder>) {
        assert!(self.end_fence.is_none());
        let end_fence = self.object_pool.get_fence();
        self.end_fence = Some(end_fence);

        unsafe {
            self.device.vk().end_command_buffer(self.pre_cmd)
        }.unwrap();

        unsafe {
            self.device.vk().end_command_buffer(self.post_cmd)
        }.unwrap();

        let submit_alloc = Bump::new();
        let mut submit_recorder = SubmitRecorder::new(32);

        if let Some(mut gob) = gob {
            gob.record(&mut submit_recorder, &submit_alloc);
            self.gob = Some(gob);
        }

        self.record_pre_submits(&mut submit_recorder, &submit_alloc);
        self.pass.record(&mut self.object_pool, &mut submit_recorder, &submit_alloc);
        for output in &mut self.outputs {
            output.record(&mut self.object_pool, &mut submit_recorder, &submit_alloc);
        }
        self.record_post_submits(&mut submit_recorder, &submit_alloc);

        unsafe {
            queue.submit_2(submit_recorder.as_slice(), Some(end_fence))
        }.unwrap();

        for output in &mut self.outputs {
            output.on_post_submit(&queue);
        }
    }

    fn is_complete(&self) -> bool {
        if let Some(fence) = self.end_fence {
            unsafe {
                self.device.vk().get_fence_status(fence)
            }.unwrap()
        } else {
            panic!("Illegal state");
        }
    }

    fn record_pre_submits<'a>(&self, recorder: &mut SubmitRecorder<'a>, alloc: &'a Bump) {
        let cmd_infos = alloc.alloc([
            vk::CommandBufferSubmitInfo::builder()
                .command_buffer(self.pre_cmd)
                .build()
        ]);

        let submit_info = vk::SubmitInfo2::builder()
            .command_buffer_infos(cmd_infos);

        recorder.push(submit_info);
    }

    fn record_post_submits<'a>(&self, _: &mut SubmitRecorder<'a>, _: &'a Bump) {
    }
}

impl Drop for PassState {
    fn drop(&mut self) {
        if let Some(immediate_buffer) = self.immediate_buffer.take() {
            self.share.return_immediate_buffer(immediate_buffer);
        }
        for shader in &self.shaders {
            self.pipeline.dec_shader_used(*shader);
        }
    }
}

struct GlobalObjectsRecorder {
    share: Arc<Share>,
    _object_pool: PooledObjectProvider,

    cmd: vk::CommandBuffer,

    staging_allocations: Vec<StagingAllocationId>,

    staging_barriers: Vec<vk::BufferMemoryBarrier2>,

    used_global_meshes: HashMap<Arc<GlobalMesh>, gob::MeshState>,
    used_global_images: HashMap<Arc<GlobalImage>, gob::ImageState>,

    /// A [`vk::ImageMemoryBarrier2`] Vec which can be used locally inside functions to avoid new
    /// allocations. It should always be cleared before use.
    tmp_image_barriers: Vec<vk::ImageMemoryBarrier2>,

    /// A [`vk::BufferMemoryBarrier2`] Vec which can be used locally inside functions to avoid new
    /// allocations. It should always be cleared before use.
    tmp_buffer_barriers: Vec<vk::BufferMemoryBarrier2>,
}

impl GlobalObjectsRecorder {
    fn new(share: Arc<Share>, object_pool: Rc<RefCell<WorkerObjectPool>>) -> Self {
        let mut object_pool = PooledObjectProvider::new(share.clone(), object_pool);

        let cmd = object_pool.get_begin_command_buffer().unwrap_or_else(|err| {
            log::error!("Failed to begin global object command buffer {:?}", err);
            panic!();
        });

        Self {
            share,
            _object_pool: object_pool,

            cmd,

            staging_allocations: Vec::new(),
            staging_barriers: Vec::new(),

            used_global_meshes: HashMap::new(),
            used_global_images: HashMap::new(),

            tmp_image_barriers: Vec::new(),
            tmp_buffer_barriers: Vec::new(),
        }
    }

    fn record_global_buffer_write(&mut self, write: GlobalMeshWrite, is_uninit: bool) {
        let dst_buffer = write.dst_mesh.get_buffer_handle();

        if !write.regions.is_empty() {
            self.transition_mesh(write.dst_mesh, gob::MeshState::TransferWrite, is_uninit);

            unsafe {
                self.share.get_device().vk().cmd_copy_buffer(
                    self.cmd,
                    write.staging_buffer,
                    dst_buffer,
                    write.regions.as_ref()
                );
            }
        }

        self.push_staging(write.staging_allocation, write.staging_buffer, write.staging_range.0, write.staging_range.1);
    }

    fn record_global_image_clear(&mut self, clear: GlobalImageClear, is_uninit: bool) {
        let dst_image = clear.dst_image.get_image_handle();

        self.transition_image(clear.dst_image, gob::ImageState::TransferWrite, is_uninit);

        unsafe {
            self.share.get_device().vk().cmd_clear_color_image(
                self.cmd,
                dst_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &clear.clear_value,
                std::slice::from_ref(&vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: vk::REMAINING_MIP_LEVELS,
                    base_array_layer: 0,
                    layer_count: vk::REMAINING_ARRAY_LAYERS
                })
            )
        }
    }

    fn record_global_image_write(&mut self, write: GlobalImageWrite, is_uninit: bool) {
        let dst_image = write.dst_image.get_image_handle();

        self.transition_image(write.dst_image, gob::ImageState::TransferWrite, is_uninit);

        if !write.regions.is_empty() {
            unsafe {
                self.share.get_device().vk().cmd_copy_buffer_to_image(
                    self.cmd,
                    write.staging_buffer,
                    dst_image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    write.regions.as_ref()
                );
            }
        }

        self.push_staging(write.staging_allocation, write.staging_buffer, write.staging_range.0, write.staging_range.1);
    }

    fn record_global_image_generate_mipmaps(&mut self, image: Arc<GlobalImage>) {
        let mip_levels = image.get_mip_levels();
        if mip_levels > 1 {
            let handle = image.get_image_handle();
            let src_size = image.get_size();
            let mut src_size = Vec2i32::new(src_size[0] as i32, src_size[1] as i32);

            self.transition_image(image, gob::ImageState::GenerateMipmaps, false);

            let device = self.share.get_device();
            for level in 1..mip_levels {
                if level > 1 {
                    let barrier = vk::ImageMemoryBarrier2::builder()
                        .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                        .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                        .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                        .dst_access_mask(vk::AccessFlags2::TRANSFER_READ)
                        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                        .image(handle)
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: level - 1,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 0
                        });

                    let info = vk::DependencyInfo::builder()
                        .image_memory_barriers(std::slice::from_ref(&barrier));

                    unsafe {
                        device.synchronization_2_khr().cmd_pipeline_barrier2(self.cmd, &info);
                    }
                }

                let dst_size = Vec2i32::new(
                    if src_size[0] > 1 { src_size[0] / 2 } else { 1 },
                    if src_size[1] > 1 { src_size[1] / 2 } else { 1 }
                );
                let blit = vk::ImageBlit::builder()
                    .src_subresource(vk::ImageSubresourceLayers {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        mip_level: level - 1,
                        base_array_layer: 0,
                        layer_count: 1
                    })
                    .src_offsets([vk::Offset3D { x: 0, y: 0, z: 0 }, vk::Offset3D { x: src_size[0], y: src_size[1], z: 1 }])
                    .dst_subresource(vk::ImageSubresourceLayers {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        mip_level: level,
                        base_array_layer: 0,
                        layer_count: 1
                    })
                    .dst_offsets([vk::Offset3D { x: 0, y: 0, z: 0 }, vk::Offset3D { x: dst_size[0], y: dst_size[1], z: 1 }]);

                unsafe {
                    device.vk().cmd_blit_image(
                        self.cmd,
                        handle,
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        handle,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        std::slice::from_ref(&blit),
                        vk::Filter::LINEAR
                    );
                }

                src_size = dst_size;
            }
        }
    }

    fn record<'a>(&mut self, recorder: &mut SubmitRecorder<'a>, bump: &'a Bump) {
        let buffer_post_barriers = self.generate_buffer_post_barriers();
        let image_post_barriers = self.generate_image_post_barriers();

        let device = self.share.get_device();

        if !buffer_post_barriers.is_empty() || !image_post_barriers.is_empty() {
            let buffer_post_barriers = buffer_post_barriers.as_slice();
            let image_post_barriers = image_post_barriers.as_slice();

            // If we have too many barriers in a single command the driver may fail to record (Yes this limit has been hit at 4000 barriers during testing in minecraft)
            const CHUNK_SIZE: usize = 256;
            let chunk_count = std::cmp::max((buffer_post_barriers.len() / CHUNK_SIZE) + 1, (image_post_barriers.len() / CHUNK_SIZE) + 1);
            for chunk in 0..chunk_count {
                let min = chunk * CHUNK_SIZE;
                let max = min + CHUNK_SIZE;
                let mut info = vk::DependencyInfo::builder();
                if min < buffer_post_barriers.len() {
                    let max = std::cmp::min(max, buffer_post_barriers.len());
                    info = info.buffer_memory_barriers(&buffer_post_barriers[min..max]);
                }
                if min < image_post_barriers.len() {
                    let max = std::cmp::min(max, image_post_barriers.len());
                    info = info.image_memory_barriers(&image_post_barriers[min..max]);
                }

                unsafe {
                    device.synchronization_2_khr().cmd_pipeline_barrier2(self.cmd, &info);
                }
            }
        }

        unsafe {
            device.vk().end_command_buffer(self.cmd)
        }.unwrap_or_else(|err| {
            log::error!("Failed to end global objects command buffer recording {:?}", err);
            panic!()
        });

        let cmd_info = bump.alloc(vk::CommandBufferSubmitInfo::builder()
            .command_buffer(self.cmd)
            .build()
        );

        recorder.push(vk::SubmitInfo2::builder()
            .command_buffer_infos(std::slice::from_ref(cmd_info))
        );
    }

    fn generate_buffer_post_barriers(&mut self) -> Vec<vk::BufferMemoryBarrier2> {
        let mut barriers = std::mem::replace(&mut self.staging_barriers, Vec::new());

        for (mesh, old_state) in &self.used_global_meshes {
            let handle = mesh.get_buffer_handle();

            gob::generate_mesh_barriers(*old_state, gob::MeshState::Ready, handle, &mut barriers);
        }

        barriers
    }

    fn generate_image_post_barriers(&mut self) -> Vec<vk::ImageMemoryBarrier2> {
        let mut barriers: Vec<vk::ImageMemoryBarrier2> = Vec::new();

        for (image, old_state) in &self.used_global_images {
            let handle = image.get_image_handle();
            let mip_levels = image.get_mip_levels();

            gob::generate_image_barriers(*old_state, gob::ImageState::Ready, handle, mip_levels, &mut barriers);
        }

        barriers
    }

    fn push_staging(&mut self, alloc: StagingAllocationId, buffer: vk::Buffer, offset: vk::DeviceSize, size: vk::DeviceSize) {
        self.staging_allocations.push(alloc);
        let barrier = vk::BufferMemoryBarrier2::builder()
            .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .src_access_mask(vk::AccessFlags2::TRANSFER_READ)
            .dst_stage_mask(vk::PipelineStageFlags2::HOST)
            .dst_access_mask(vk::AccessFlags2::HOST_WRITE)
            .buffer(buffer)
            .offset(offset)
            .size(size);

        let info = vk::DependencyInfo::builder()
            .buffer_memory_barriers(std::slice::from_ref(&barrier));

        unsafe {
            self.share.get_device().synchronization_2_khr().cmd_pipeline_barrier2(self.cmd, &info)
        };
    }

    /// Transitions a mesh to a new state and adds it to the used mesh list.
    ///
    /// If the mesh is not in the used mesh list the mesh is currently either uninitialized or
    /// ready. In that case if maybe_uninit is set the mesh is assumed to be uninitialized otherwise
    /// it is assumed to be in the ready state.
    fn transition_mesh(&mut self, mesh: Arc<GlobalMesh>, new_state: gob::MeshState, maybe_uninit: bool) {
        let handle = mesh.get_buffer_handle();

        let old_state = self.used_global_meshes.insert(mesh, new_state).unwrap_or_else(|| {
            if maybe_uninit {
                gob::MeshState::Uninitialized
            } else {
                gob::MeshState::Ready
            }
        });

        self.tmp_buffer_barriers.clear();
        gob::generate_mesh_barriers(old_state, new_state, handle, &mut self.tmp_buffer_barriers);

        if !self.tmp_buffer_barriers.is_empty() {
            let info = vk::DependencyInfo::builder()
                .buffer_memory_barriers(self.tmp_buffer_barriers.as_slice());

            unsafe {
                self.share.get_device().synchronization_2_khr().cmd_pipeline_barrier2(self.cmd, &info);
            }
        }
    }

    /// Transitions a image to a new state and adds it to the used image list.
    ///
    /// If the image is not in the used image list the image is currently either uninitialized or
    /// ready. In that case if maybe_uninit is set the image is assumed to be uninitialized otherwise
    /// it is assumed to be in the ready state.
    fn transition_image(&mut self, image: Arc<GlobalImage>, new_state: gob::ImageState, maybe_uninit: bool) {
        let handle = image.get_image_handle();
        let mip_levels = image.get_mip_levels();

        let old_state = self.used_global_images.insert(image, new_state).unwrap_or_else(|| {
            if maybe_uninit {
                gob::ImageState::Uninitialized
            } else {
                gob::ImageState::Ready
            }
        });

        self.tmp_image_barriers.clear();
        gob::generate_image_barriers(old_state, new_state, handle, mip_levels, &mut self.tmp_image_barriers);

        if !self.tmp_image_barriers.is_empty() {
            let info = vk::DependencyInfo::builder()
                .image_memory_barriers(self.tmp_image_barriers.as_slice());

            unsafe {
                self.share.get_device().synchronization_2_khr().cmd_pipeline_barrier2(self.cmd, &info);
            }
        }
    }
}

impl Drop for GlobalObjectsRecorder {
    fn drop(&mut self) {
        let mut guard = self.share.get_staging_pool().lock().unwrap_or_else(|_| {
            log::error!("Poisoned staging memory mutex in GlobalObjectsRecorder::drop");
            panic!();
        });

        for allocation in std::mem::replace(&mut self.staging_allocations, Vec::new()) {
            guard.free(allocation);
        }
    }
}

mod gob {
    //! Utility functions to create barriers for global objects

    use ash::vk;

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub(super) enum MeshState {
        /// Mesh has not been initialized yet
        Uninitialized,
        /// Mesh is ready to be used for rendering
        Ready,
        /// Mesh was previously written to
        TransferWrite,
    }

    pub(super) fn generate_mesh_barriers(old_state: MeshState, new_state: MeshState, buffer: vk::Buffer, barriers: &mut Vec<vk::BufferMemoryBarrier2>) {
        match (old_state, new_state) {
            (MeshState::Uninitialized, _) => {
            },
            (old, MeshState::Uninitialized) => {
                log::error!("Mesh cannot be transitioned into uninitialized (was {:?})", old);
                panic!();
            },
            (MeshState::Ready, MeshState::Ready) => {
                log::warn!("Transitioned mesh from ready to ready. Why?");
            }
            (old, new) => {
                let mut barrier = vk::BufferMemoryBarrier2::builder()
                    .buffer(buffer)
                    .offset(0)
                    .size(vk::WHOLE_SIZE);
                barrier = match old {
                    MeshState::Uninitialized => panic!(), // Impossible
                    MeshState::Ready => MESH_READY_INFO().write_src(barrier),
                    MeshState::TransferWrite => MESH_TRANSFER_WRITE_INFO.write_src(barrier)
                };
                barrier = match new {
                    MeshState::Uninitialized => panic!(), // Impossible
                    MeshState::Ready => MESH_READY_INFO().write_dst(barrier),
                    MeshState::TransferWrite => MESH_TRANSFER_WRITE_INFO.write_dst(barrier)
                };

                barriers.push(barrier.build());
            }
        }
    }

    // This needs to be a function because of the bitor. Waiting for const impl
    #[allow(non_snake_case)]
    fn MESH_READY_INFO() -> BufferAccessInfo {
        BufferAccessInfo::new(vk::PipelineStageFlags2::VERTEX_INPUT, vk::AccessFlags2::VERTEX_ATTRIBUTE_READ | vk::AccessFlags2::INDEX_READ)
    }
    const MESH_TRANSFER_WRITE_INFO: BufferAccessInfo = BufferAccessInfo::new(vk::PipelineStageFlags2::TRANSFER, vk::AccessFlags2::TRANSFER_WRITE);

    struct BufferAccessInfo {
        stage_mask: vk::PipelineStageFlags2,
        access_mask: vk::AccessFlags2,
    }

    impl BufferAccessInfo {
        #[inline]
        const fn new(stage_mask: vk::PipelineStageFlags2, access_mask: vk::AccessFlags2) -> Self {
            Self {
                stage_mask,
                access_mask
            }
        }

        #[inline]
        fn write_src<'a>(&self, barrier: vk::BufferMemoryBarrier2Builder<'a>) -> vk::BufferMemoryBarrier2Builder<'a> {
            barrier
                .src_stage_mask(self.stage_mask)
                .src_access_mask(self.access_mask)
        }

        #[inline]
        fn write_dst<'a>(&self, barrier: vk::BufferMemoryBarrier2Builder<'a>) -> vk::BufferMemoryBarrier2Builder<'a> {
            barrier
                .dst_stage_mask(self.stage_mask)
                .dst_access_mask(self.access_mask)
        }
    }

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub(super) enum ImageState {
        /// Image has not been initialized yet
        Uninitialized,
        /// Image is ready to be used for rendering
        Ready,
        /// Image was previously written to
        TransferWrite,
        /// Image had previously generated its mipmaps
        GenerateMipmaps,
    }

    pub(super) fn generate_image_barriers(old_state: ImageState, new_state: ImageState, image: vk::Image, mip_levels: u32, barriers: &mut Vec<vk::ImageMemoryBarrier2>) {
        match (old_state, new_state) {
            (ImageState::Uninitialized, ImageState::TransferWrite) => {
                let mut barrier = vk::ImageMemoryBarrier2::builder()
                    .image(image)
                    .subresource_range(make_full_subresource_range(vk::ImageAspectFlags::COLOR));
                barrier = IMAGE_UNINITIALIZED_INFO.write_src(barrier);
                barrier = IMAGE_TRANSFER_WRITE_INFO.write_dst(barrier);

                barriers.push(barrier.build());
            }
            (ImageState::Ready, ImageState::TransferWrite) => {
                let mut barrier = vk::ImageMemoryBarrier2::builder()
                    .image(image)
                    .subresource_range(make_full_subresource_range(vk::ImageAspectFlags::COLOR));
                barrier = IMAGE_READY_INFO.write_src(barrier);
                barrier = IMAGE_TRANSFER_WRITE_INFO.write_dst(barrier);

                barriers.push(barrier.build());
            }
            (ImageState::Ready, ImageState::GenerateMipmaps) => {
                let mut barrier0 = vk::ImageMemoryBarrier2::builder()
                    .image(image)
                    .subresource_range(make_first_mip_subresource_range(vk::ImageAspectFlags::COLOR));
                barrier0 = IMAGE_READY_INFO.write_src(barrier0);
                barrier0 = IMAGE_GENERATE_MIPMAPS_0_INFO.write_dst(barrier0);

                barriers.push(barrier0.build());

                let mut barrier1 = vk::ImageMemoryBarrier2::builder()
                    .image(image)
                    .subresource_range(make_exclude_first_mips_subresource_range(vk::ImageAspectFlags::COLOR));
                barrier1 = IMAGE_READY_INFO.write_src(barrier1);
                barrier1 = IMAGE_GENERATE_MIPMAPS_1_INFO.write_dst(barrier1);

                barriers.push(barrier1.build());
            }
            (ImageState::TransferWrite, ImageState::Ready) => {
                let mut barrier = vk::ImageMemoryBarrier2::builder()
                    .image(image)
                    .subresource_range(make_full_subresource_range(vk::ImageAspectFlags::COLOR));
                barrier = IMAGE_TRANSFER_WRITE_INFO.write_src(barrier);
                barrier = IMAGE_READY_INFO.write_dst(barrier);

                barriers.push(barrier.build());
            }
            (ImageState::TransferWrite, ImageState::TransferWrite) => {
                let mut barrier = vk::ImageMemoryBarrier2::builder()
                    .image(image)
                    .subresource_range(make_full_subresource_range(vk::ImageAspectFlags::COLOR));
                barrier = IMAGE_TRANSFER_WRITE_INFO.write_src(barrier);
                barrier = IMAGE_TRANSFER_WRITE_INFO.write_dst(barrier);

                barriers.push(barrier.build());
            }
            (ImageState::TransferWrite, ImageState::GenerateMipmaps) => {
                let mut barrier0 = vk::ImageMemoryBarrier2::builder()
                    .image(image)
                    .subresource_range(make_first_mip_subresource_range(vk::ImageAspectFlags::COLOR));
                barrier0 = IMAGE_TRANSFER_WRITE_INFO.write_src(barrier0);
                barrier0 = IMAGE_GENERATE_MIPMAPS_0_INFO.write_dst(barrier0);

                barriers.push(barrier0.build());

                let mut barrier1 = vk::ImageMemoryBarrier2::builder()
                    .image(image)
                    .subresource_range(make_exclude_first_mips_subresource_range(vk::ImageAspectFlags::COLOR));
                barrier1 = IMAGE_TRANSFER_WRITE_INFO.write_src(barrier1);
                barrier1 = IMAGE_GENERATE_MIPMAPS_1_INFO.write_dst(barrier1);

                barriers.push(barrier1.build());
            }
            (ImageState::GenerateMipmaps, ImageState::Ready) => {
                let mut barrier0 = vk::ImageMemoryBarrier2::builder()
                    .image(image)
                    .subresource_range(make_exclude_last_mips_subresource_range(vk::ImageAspectFlags::COLOR, mip_levels));
                barrier0 = IMAGE_GENERATE_MIPMAPS_0_INFO.write_src(barrier0);
                barrier0 = IMAGE_READY_INFO.write_dst(barrier0);

                barriers.push(barrier0.build());

                let mut barrier1 = vk::ImageMemoryBarrier2::builder()
                    .image(image)
                    .subresource_range(make_last_mip_subresource_range(vk::ImageAspectFlags::COLOR, mip_levels));
                barrier1 = IMAGE_GENERATE_MIPMAPS_1_INFO.write_src(barrier1);
                barrier1 = IMAGE_READY_INFO.write_dst(barrier1);

                barriers.push(barrier1.build());
            }
            (ImageState::GenerateMipmaps, ImageState::TransferWrite) => {
                let mut barrier0 = vk::ImageMemoryBarrier2::builder()
                    .image(image)
                    .subresource_range(make_exclude_last_mips_subresource_range(vk::ImageAspectFlags::COLOR, mip_levels));
                barrier0 = IMAGE_GENERATE_MIPMAPS_0_INFO.write_src(barrier0);
                barrier0 = IMAGE_TRANSFER_WRITE_INFO.write_dst(barrier0);

                barriers.push(barrier0.build());

                let mut barrier1 = vk::ImageMemoryBarrier2::builder()
                    .image(image)
                    .subresource_range(make_last_mip_subresource_range(vk::ImageAspectFlags::COLOR, mip_levels));
                barrier1 = IMAGE_GENERATE_MIPMAPS_1_INFO.write_src(barrier1);
                barrier1 = IMAGE_TRANSFER_WRITE_INFO.write_dst(barrier1);

                barriers.push(barrier1.build());
            }
            (ImageState::Ready, ImageState::Ready) => {
                log::warn!("Transitioned image from ready to ready. Why?");
            }
            (ImageState::Uninitialized, new) => {
                log::error!("Image cannot be transitioned from uninitialized to {:?}", new);
                panic!();
            }
            (old, ImageState::Uninitialized) => {
                log::error!("Image cannot be transitioned into uninitialized (was {:?})", old);
                panic!();
            }
            (ImageState::GenerateMipmaps, ImageState::GenerateMipmaps) => {
                log::error!("Image cannot be transitioned from generate mipmaps to generate mipmaps");
                panic!();
            }
        }
    }

    #[inline]
    fn make_full_subresource_range(aspect_mask: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count: vk::REMAINING_MIP_LEVELS,
            base_array_layer: 0,
            layer_count: vk::REMAINING_ARRAY_LAYERS
        }
    }

    #[inline]
    fn make_exclude_last_mips_subresource_range(aspect_mask: vk::ImageAspectFlags, mip_levels: u32) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count: mip_levels - 1,
            base_array_layer: 0,
            layer_count: vk::REMAINING_ARRAY_LAYERS
        }
    }

    #[inline]
    fn make_last_mip_subresource_range(aspect_mask: vk::ImageAspectFlags, mip_levels: u32) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: mip_levels - 1,
            level_count: 1,
            base_array_layer: 0,
            layer_count: vk::REMAINING_ARRAY_LAYERS
        }
    }

    #[inline]
    fn make_exclude_first_mips_subresource_range(aspect_mask: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 1,
            level_count: vk::REMAINING_MIP_LEVELS,
            base_array_layer: 0,
            layer_count: vk::REMAINING_ARRAY_LAYERS,
        }
    }

    #[inline]
    fn make_first_mip_subresource_range(aspect_mask: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: vk::REMAINING_ARRAY_LAYERS,
        }
    }

    const IMAGE_UNINITIALIZED_INFO: ImageAccessInfo = ImageAccessInfo::new(vk::PipelineStageFlags2::NONE, vk::AccessFlags2::NONE, vk::ImageLayout::UNDEFINED);
    const IMAGE_READY_INFO: ImageAccessInfo = ImageAccessInfo::new(vk::PipelineStageFlags2::FRAGMENT_SHADER, vk::AccessFlags2::SHADER_SAMPLED_READ, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    const IMAGE_TRANSFER_WRITE_INFO: ImageAccessInfo = ImageAccessInfo::new(vk::PipelineStageFlags2::TRANSFER, vk::AccessFlags2::TRANSFER_WRITE, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
    const IMAGE_GENERATE_MIPMAPS_0_INFO: ImageAccessInfo = ImageAccessInfo::new(vk::PipelineStageFlags2::TRANSFER, vk::AccessFlags2::TRANSFER_READ, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
    const IMAGE_GENERATE_MIPMAPS_1_INFO: ImageAccessInfo = ImageAccessInfo::new(vk::PipelineStageFlags2::TRANSFER, vk::AccessFlags2::TRANSFER_WRITE, vk::ImageLayout::TRANSFER_DST_OPTIMAL);

    struct ImageAccessInfo {
        stage_mask: vk::PipelineStageFlags2,
        access_mask: vk::AccessFlags2,
        layout: vk::ImageLayout,
    }

    impl ImageAccessInfo {
        #[inline]
        const fn new(stage_mask: vk::PipelineStageFlags2, access_mask: vk::AccessFlags2, layout: vk::ImageLayout) -> Self {
            Self {
                stage_mask,
                access_mask,
                layout
            }
        }

        #[inline]
        fn write_src<'a>(&self, barrier: vk::ImageMemoryBarrier2Builder<'a>) -> vk::ImageMemoryBarrier2Builder<'a> {
            barrier
                .src_stage_mask(self.stage_mask)
                .src_access_mask(self.access_mask)
                .old_layout(self.layout)
        }

        #[inline]
        fn write_dst<'a>(&self, barrier: vk::ImageMemoryBarrier2Builder<'a>) -> vk::ImageMemoryBarrier2Builder<'a> {
            barrier
                .dst_stage_mask(self.stage_mask)
                .dst_access_mask(self.access_mask)
                .new_layout(self.layout)
        }
    }
}