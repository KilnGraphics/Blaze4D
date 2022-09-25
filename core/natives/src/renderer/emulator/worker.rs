use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ash::vk;

use crate::prelude::*;
use crate::renderer::emulator::Image;

mod task {
    use std::any::Any;
    use std::cell::RefCell;
    use std::sync::Arc;

    use bumpalo::Bump;
    use crate::renderer::emulator::{EmulatorTask, ExportSet};
    use crate::renderer::emulator::share::Share2;
    use crate::renderer::emulator::staging::StagingAllocationId2;

    pub(in crate::renderer::emulator)
    enum WorkerTask3 {
        Emulator(EmulatorTaskContainer),
        Export(u64, u64, Arc<ExportSet>),
        Flush(u64),
        Shutdown(u64),
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

pub(super) fn run_worker2(share: Arc<Share2>) {
    let object_pool = RefCell::new(ObjectPool2::new(share.clone()).unwrap());
    let mut artifacts = VecDeque::with_capacity(3);

    let mut recorder = None;

    let mut pending_export = None;
    let mut last_sync = 0u64;
    let mut last_update = Instant::now();
    loop {
        if let Some(task) = share.pop_task(Duration::from_millis(33)) {
            match task {
                WorkerTask3::Emulator(task) => {
                    if recorder.is_none() {
                        recorder = Some(Recorder::new(&object_pool));
                    }
                    recorder.as_mut().unwrap().push_task(task);
                }
                WorkerTask3::Export(signal_value, wait_value, export_set) => {
                    let images: Box<_> = export_set.get_images().iter().map(|i| (&**i, vk::ImageLayout::GENERAL)).collect();
                    submit_recorder(&share, &mut recorder, &object_pool, &mut pending_export, &mut last_sync, signal_value, &images, &mut artifacts);

                    last_sync = wait_value;
                    pending_export = Some(wait_value);
                    share.signal_export(signal_value);
                }
                WorkerTask3::Flush(signal_value) => {
                    submit_recorder(&share, &mut recorder, &object_pool, &mut pending_export, &mut last_sync, signal_value, &[], &mut artifacts);
                }
                WorkerTask3::Shutdown(signal_value) => {
                    submit_recorder(&share, &mut recorder, &object_pool, &mut pending_export, &mut last_sync, signal_value, &[], &mut artifacts);
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

fn submit_recorder<'a>(share: &Share2, recorder: &mut Option<Recorder<'a>>, object_pool: &'a RefCell<ObjectPool2>, pending_export: &mut Option<u64>, last_sync: &mut u64, signal_value: u64, image_transitions: &[(&Image, vk::ImageLayout)], artifacts: &mut VecDeque<SubmissionArtifact<'a>>) {
    if recorder.is_none() {
        // Check if we need a recorder for image barriers
        for (image, dst_layout) in image_transitions {
            if unsafe { image.get_current_layout() } != *dst_layout {
                // We need at least one image barrier so we need a recorder.
                *recorder = Some(Recorder::new(&object_pool));
                break;
            }
        }
    }

    if let Some(recorder) = recorder.take() {
        if let Some(pending_export) = pending_export.take() {
            // We cannot submit before the external commands have been submitted so we must wait
            let semaphore = share.get_semaphore();
            let info = vk::SemaphoreWaitInfo::builder()
                .semaphores(std::slice::from_ref(&semaphore))
                .values(std::slice::from_ref(&pending_export));
            unsafe {
                share.get_device().timeline_semaphore_khr().wait_semaphores(&info, u64::MAX)
            }.unwrap();
        }

        artifacts.push_back(recorder.submit(*last_sync, signal_value, image_transitions));
    } else {
        let wait_value = if let Some(pending_export) = pending_export.take() {
            // If a pending export exist its impossible for anything to have been submitted yet
            pending_export
        } else {
            *last_sync
        };

        // There is nothing to submit so we must manually ensure that we signal the required value
        let semaphore = share.get_semaphore();
        let info = vk::SemaphoreWaitInfo::builder()
            .semaphores(std::slice::from_ref(&semaphore))
            .values(std::slice::from_ref(&wait_value));
        unsafe {
            share.get_device().timeline_semaphore_khr().wait_semaphores(&info, u64::MAX)
        }.unwrap();

        let info = vk::SemaphoreSignalInfo::builder()
            .semaphore(semaphore)
            .value(signal_value);
        unsafe {
            share.get_device().timeline_semaphore_khr().signal_semaphore(&info)
        }.unwrap();
    }

    *last_sync = signal_value;
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
            self.device.vk().reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
                .expect("Failed to reset command buffers"); // Maybe recover from out of device memory?

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
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::sync::Arc;

    use ash::vk;
    use ouroboros::self_referencing;

    use crate::renderer::emulator::{Buffer, EmulatorTask, Image};
    use super::{AllocationCache, EmulatorTaskContainer, ObjectCache, ObjectPool2};

    use crate::prelude::*;

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

            let value = unsafe {
                pool.get_device().timeline_semaphore_khr().get_semaphore_counter_value(pool.share.get_semaphore())
            }.unwrap();

            value >= self.wait_value
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
            self.recorder.as_mut().unwrap().push_task(task)
        }

        pub(super) fn submit(mut self, wait_value: u64, signal_value: u64, image_transitions: &[(&Image, vk::ImageLayout)]) -> SubmissionArtifact<'a> {
            self.recorder.take().unwrap().submit(wait_value, signal_value, image_transitions)
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

        fn submit(&mut self, wait_value: u64, signal_value: u64, image_transitions: &[(&Image, vk::ImageLayout)]) -> SubmissionArtifact<'a> {
            self.reorder_main_recorder();
            self.finish_reorder_recorder();

            debug_assert!(self.reorder_recorder.is_none());
            debug_assert!(self.main_recorder.is_none());

            let queue = self.device.get_main_queue();
            let semaphore = self.object_pool.borrow().get_share().get_semaphore();

            let mut post_state = self.pre_state.take().unwrap();
            let mut image_barriers = Vec::with_capacity(image_transitions.len());

            for (image, dst_layout) in image_transitions {
                let new_state = ImageState::ReadUniform(*dst_layout, vk::PipelineStageFlags2::NONE, vk::AccessFlags2::NONE);

                // We will deal with any required barriers here so we can remove the image from the post state
                if let Some(old_state) = post_state.images.remove(*image) {
                    old_state.gen_barriers(&new_state, image, &mut image_barriers);
                } else {
                    let old_layout = unsafe { image.get_current_layout() };
                    if old_layout != *dst_layout {
                        let old_state = ImageState::ReadUniform(old_layout, vk::PipelineStageFlags2::NONE, vk::AccessFlags2::NONE);
                        old_state.gen_barriers(&new_state, image, &mut image_barriers);
                    }
                }
                unsafe { image.set_current_layout(*dst_layout) };
            }

            for (image, state) in post_state.images {
                match state {
                    ImageState::ReadUniform(layout, _, _) |
                    ImageState::ReadWriteUniform(layout, _, _) => {
                        unsafe { image.set_current_layout(layout) };
                    }
                }
            }

            if image_barriers.len() != 0 {
                let dependency_info = vk::DependencyInfo::builder()
                    .image_memory_barriers(&image_barriers);

                unsafe {
                    self.device.synchronization_2_khr().cmd_pipeline_barrier2(self.cmd.unwrap(), &dependency_info);
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
                EmulatorTask::Draw(_) => todo!()
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
use crate::renderer::emulator::share::Share2;