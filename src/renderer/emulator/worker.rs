use std::cell::RefCell;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use ash::prelude::VkResult;
use ash::vk;
use ash::vk::BufferMemoryBarrier2;
use bumpalo::Bump;

use crate::device::device::Queue;
use crate::device::transfer::{SyncId, Transfer};

use crate::renderer::emulator::pass::PassId;
use crate::renderer::emulator::buffer::BufferPool;
use crate::renderer::emulator::pipeline::{EmulatorOutput, EmulatorPipeline, EmulatorPipelinePass, PipelineTask};
use crate::vk::objects::buffer::Buffer;

use crate::prelude::*;
use crate::renderer::emulator::descriptors::DescriptorPool;
use crate::renderer::emulator::StaticMeshId;
use crate::renderer::emulator::global_objects::GlobalObjects;
use crate::renderer::emulator::mc_shaders::ShaderId;
use crate::vk::objects::allocator::AllocationStrategy;

pub struct Share {
    pub(super) device: Arc<DeviceContext>,
    pub(super) global_objects: GlobalObjects,
    pub(super) descriptors: Mutex<DescriptorPool>,
    pub(super) pool: Arc<Mutex<BufferPool>>,
    channel: Mutex<Channel>,
    signal: Condvar,
    family: u32,
}

impl Share {
    pub fn new(device: Arc<DeviceContext>, pool: Arc<Mutex<BufferPool>>) -> Self {
        let queue = device.get_main_queue();
        let queue_family = queue.get_queue_family_index();

        let global_objects = GlobalObjects::new(device.clone(), queue.clone());
        let descriptors = Mutex::new(DescriptorPool::new(device.clone()));

        Self {
            device,
            global_objects,
            descriptors,
            pool,
            channel: Mutex::new(Channel::new()),
            signal: Condvar::new(),
            family: queue_family,
        }
    }

    pub fn get_render_queue_family(&self) -> u32 {
        self.family
    }

    pub(super) fn push_task(&self, id: PassId, task: WorkerTask) {
        self.channel.lock().unwrap().queue.push_back((id, task));
        self.signal.notify_one();
    }

    fn try_get_next_task_timeout(&self, timeout: Duration) -> NextTaskResult {
        let start = Instant::now();

        let mut guard = self.channel.lock().unwrap_or_else(|_| {
            log::error!("Poisoned channel mutex in Share::try_get_next_task!");
            panic!()
        });

        loop {
            if let Some((id, task)) = guard.queue.pop_front() {
                return NextTaskResult::Ok(id, task);
            }

            let diff = (start + timeout).saturating_duration_since(Instant::now());
            if diff.is_zero() {
                return NextTaskResult::Timeout;
            }

            let (new_guard, timeout) = self.signal.wait_timeout(guard, diff).unwrap_or_else(|_| {
                log::error!("Poisoned channel mutex in Share::try_get_next_task!");
                panic!()
            });
            guard = new_guard;

            if timeout.timed_out() {
                return NextTaskResult::Timeout;
            }
        }
    }
}

enum NextTaskResult {
    Ok(PassId, WorkerTask),
    Timeout,
}

// TODO this is needed because condvar is not unwind safe can we do better?
impl UnwindSafe for Share {
}
impl RefUnwindSafe for Share {
}

struct Channel {
    queue: VecDeque<(PassId, WorkerTask)>,
}

impl Channel {
    fn new() -> Self {
        Self {
            queue: VecDeque::new()
        }
    }
}

pub(super) enum WorkerTask {
    StartPass(Arc<dyn EmulatorPipeline>, Box<dyn EmulatorPipelinePass + Send>),
    EndPass,
    UseDynamicBuffer(Buffer),
    UseStaticMesh(StaticMeshId),
    UseShader(ShaderId),
    UseOutput(Box<dyn EmulatorOutput + Send>),
    WaitTransferSync(SyncId),
    PipelineTask(PipelineTask),
}

pub(super) fn run_worker(device: Arc<DeviceContext>, share: Arc<Share>) {
    let queue = device.get_main_queue();

    let pool = Rc::new(RefCell::new(WorkerObjectPool::new(device.clone(), queue.get_queue_family_index())));
    let mut frames = PassList::new();
    let mut old_frames = Vec::new();

    let queue = device.get_main_queue();

    loop {
        share.global_objects.update();

        old_frames.retain(|old: &PassState| {
            if old.is_complete() {
                let mut guard = share.pool.lock().unwrap();
                for buffer in &old.dynamic_buffers {
                    guard.return_buffer(buffer.get_id(), None);
                }
                drop(guard);
                for static_mesh in &old.static_meshes {
                    share.global_objects.dec_static_mesh(*static_mesh);
                }
                for shader in &old.shaders {
                    old.pipeline.dec_shader_used(*shader);
                }
                false
            } else {
                true
            }
        });

        let (id, task) = match share.try_get_next_task_timeout(Duration::from_micros(500)) {
            NextTaskResult::Ok(id, task) => (id, task),
            NextTaskResult::Timeout => continue,
        };

        match task {
            WorkerTask::StartPass(pipeline, pass) => {
                let state = PassState::new(pipeline, pass, device.clone(), &queue, share.clone(), pool.clone());
                frames.add_pass(id, state);
            }

            WorkerTask::EndPass => {
                share.global_objects.flush();
                let mut pass = frames.pop_pass(id).unwrap();
                pass.submit(&queue);
                old_frames.push(pass);
            }

            WorkerTask::UseDynamicBuffer(buffer) => {
                let op = device.get_transfer().prepare_buffer_release(buffer, Some((
                    vk::PipelineStageFlags2::VERTEX_INPUT | vk::PipelineStageFlags2::INDEX_INPUT,
                    vk::AccessFlags2::VERTEX_ATTRIBUTE_READ | vk::AccessFlags2::INDEX_READ,
                    queue.get_queue_family_index()
                )));
                frames.get_pass(id).unwrap().use_dynamic_buffer(buffer, op.make_barrier().as_ref());
            }

            WorkerTask::UseStaticMesh(mesh_id) => {
                frames.get_pass(id).unwrap().static_meshes.push(mesh_id);
            }

            WorkerTask::UseShader(shader) => {
                frames.get_pass(id).unwrap().shaders.push(shader);
            }

            WorkerTask::UseOutput(output) => {
                frames.get_pass(id).unwrap().use_output(output);
            }

            WorkerTask::WaitTransferSync(sync_id) => {
                frames.get_pass(id).unwrap().wait_transfer(sync_id);
            }

            WorkerTask::PipelineTask(task) => {
                frames.get_pass(id).unwrap().process_task(&task);
            }
        }
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

    pub fn allocate_uniform<T: ToBytes>(&mut self, data: &T) -> (vk::Buffer, vk::DeviceSize) {
        self.share.descriptors.lock().unwrap().allocate_uniform(data)
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
    device: Arc<DeviceContext>,
    transfer: Arc<Transfer>,
    object_pool: PooledObjectProvider,

    pipeline: Arc<dyn EmulatorPipeline>,
    pass: Box<dyn EmulatorPipelinePass>,
    outputs: Vec<Box<dyn EmulatorOutput>>,

    dynamic_buffers: Vec<Buffer>,
    static_meshes: Vec<StaticMeshId>,
    shaders: Vec<ShaderId>,
    transfer_sync_wait: Option<SyncId>,

    pre_cmd: vk::CommandBuffer,
    post_cmd: vk::CommandBuffer,

    end_fence: Option<vk::Fence>,
}

impl PassState {
    fn new(pipeline: Arc<dyn EmulatorPipeline>, mut pass: Box<dyn EmulatorPipelinePass>, device: Arc<DeviceContext>, queue: &Queue, share: Arc<Share>, pool: Rc<RefCell<WorkerObjectPool>>) -> Self {
        let mut object_pool = PooledObjectProvider::new(share, pool);

        let pre_cmd = object_pool.get_begin_command_buffer().unwrap();
        let post_cmd = object_pool.get_begin_command_buffer().unwrap();

        pass.init(queue, &mut object_pool);

        let transfer = device.get_transfer().clone();

        Self {
            device,
            transfer,
            object_pool,

            pipeline,
            pass,
            outputs: Vec::with_capacity(8),

            dynamic_buffers: Vec::with_capacity(8),
            static_meshes: Vec::new(),
            shaders: Vec::new(),
            transfer_sync_wait: None,

            pre_cmd,
            post_cmd,

            end_fence: None,
        }
    }

    fn wait_transfer(&mut self, sync_id: SyncId) {
        self.transfer_sync_wait = Some(self.transfer_sync_wait.map_or(sync_id, |old| std::cmp::max(old, sync_id)));
    }

    fn use_dynamic_buffer(&mut self, buffer: Buffer, pre_barrier: Option<&BufferMemoryBarrier2>) {
        self.dynamic_buffers.push(buffer);

        if let Some(pre_barrier) = pre_barrier {
            let info = vk::DependencyInfo::builder()
                .buffer_memory_barriers(std::slice::from_ref(pre_barrier));

            unsafe {
                self.device.vk().cmd_pipeline_barrier2(self.pre_cmd, &info)
            };
        }
    }

    fn wait_barrier(&mut self, pre_barrier: Option<&BufferMemoryBarrier2>) {
        if let Some(pre_barrier) = pre_barrier {
            let info = vk::DependencyInfo::builder()
                .buffer_memory_barriers(std::slice::from_ref(pre_barrier));

            unsafe {
                self.device.vk().cmd_pipeline_barrier2(self.pre_cmd, &info)
            };
        }
    }

    fn use_output(&mut self, mut output: Box<dyn EmulatorOutput>) {
        output.init(self.pass.as_ref(), &mut self.object_pool);
        self.outputs.push(output);
    }

    fn process_task(&mut self, task: &PipelineTask) {
        self.pass.process_task(task, &mut self.object_pool);
    }

    fn submit(&mut self, queue: &Queue) {
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

        self.record_pre_submits(&mut submit_recorder, &submit_alloc);
        self.pass.record(&mut self.object_pool, &mut submit_recorder, &submit_alloc);
        for output in &mut self.outputs {
            output.record(&mut self.object_pool, &mut submit_recorder, &submit_alloc);
        }
        self.record_post_submits(&mut submit_recorder, &submit_alloc);

        if let Some(sync_id) = &self.transfer_sync_wait {
            // Release barriers on the transfer queue must be submitted before the acquire barriers on the graphics queue
            self.transfer.wait_for_submit(*sync_id)
        }
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
        let wait_infos: &[vk::SemaphoreSubmitInfo] = if let Some(sync_id) = &self.transfer_sync_wait {
            let op = self.transfer.generate_wait_semaphore(*sync_id);
            alloc.alloc([
                vk::SemaphoreSubmitInfo::builder()
                    .semaphore(op.semaphore.get_handle())
                    .value(op.value.unwrap_or(0))
                    .stage_mask(vk::PipelineStageFlags2::VERTEX_INPUT | vk::PipelineStageFlags2::INDEX_INPUT)
                    .build()
            ])
        } else {
            alloc.alloc([])
        };

        let cmd_infos = alloc.alloc([
            vk::CommandBufferSubmitInfo::builder()
                .command_buffer(self.pre_cmd)
                .build()
        ]);

        let submit_info = vk::SubmitInfo2::builder()
            .wait_semaphore_infos(wait_infos)
            .command_buffer_infos(cmd_infos);

        recorder.push(submit_info);
    }

    fn record_post_submits<'a>(&self, _: &mut SubmitRecorder<'a>, _: &'a Bump) {
    }
}

struct PassList {
    frames: Vec<(PassId, PassState)>,
}

impl PassList {
    fn new() -> Self {
        Self {
            frames: Vec::new(),
        }
    }

    fn add_pass(&mut self, id: PassId, state: PassState) {
        self.frames.push((id, state))
    }

    fn pop_pass(&mut self, id: PassId) -> Option<PassState> {
        let mut index = None;
        for (pass_index, (pass_id, _)) in self.frames.iter().enumerate() {
            if id == *pass_id {
                index = Some(pass_index);
            }
        }

        index.map(|index| self.frames.swap_remove(index).1)
    }

    fn get_pass(&mut self, id: PassId) -> Option<&mut PassState> {
        for (pass_id, pass) in &mut self.frames {
            if id == *pass_id {
                return Some(pass);
            }
        }
        None
    }
}