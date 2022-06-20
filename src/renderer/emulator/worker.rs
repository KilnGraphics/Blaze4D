use std::cell::RefCell;
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
use crate::objects::sync::SemaphoreOp;

use crate::renderer::emulator::pass::PassId;
use crate::renderer::emulator::immediate::ImmediateBuffer;
use crate::renderer::emulator::pipeline::{EmulatorOutput, EmulatorPipeline, EmulatorPipelinePass, PipelineTask};
use crate::vk::objects::buffer::Buffer;

use crate::prelude::*;
use crate::renderer::emulator::descriptors::DescriptorPool;
use crate::renderer::emulator::StaticMeshId;
use crate::renderer::emulator::global_objects::GlobalObjects;
use crate::renderer::emulator::mc_shaders::ShaderId;
use crate::renderer::emulator::share::{NextTaskResult, Share};
use crate::vk::objects::allocator::AllocationStrategy;

pub(super) enum WorkerTask {
    StartPass(PassId, Arc<dyn EmulatorPipeline>, Box<dyn EmulatorPipelinePass + Send>),
    EndPass(Box<ImmediateBuffer>),
    UseStaticMesh(StaticMeshId),
    UseShader(ShaderId),
    UseOutput(Box<dyn EmulatorOutput + Send>),
    PipelineTask(PipelineTask),
}

pub(super) fn run_worker(device: Arc<DeviceContext>, share: Arc<Share>) {
    let queue = device.get_main_queue();

    let pool = Rc::new(RefCell::new(WorkerObjectPool::new(device.clone(), queue.get_queue_family_index())));
    let mut current_pass: Option<PassState> = None;
    let mut old_frames = Vec::new();

    let queue = device.get_main_queue();

    loop {
        share.worker_update();

        old_frames.retain(|old: &PassState| {
            !old.is_complete()
        });

        let task = match share.try_get_next_task_timeout(Duration::from_micros(500)) {
            NextTaskResult::Ok(task) => task,
            NextTaskResult::Timeout => continue,
        };

        match task {
            WorkerTask::StartPass(_, pipeline, pass) => {
                if current_pass.is_some() {
                    log::error!("Worker received WorkerTask::StartPass when a pass is already running");
                    panic!()
                }
                let state = PassState::new(pipeline, pass, device.clone(), &queue, share.clone(), pool.clone());
                current_pass = Some(state);
            }

            WorkerTask::EndPass(immediate_buffer) => {
                if let Some(mut pass) = current_pass.take() {
                    if let Some(op) = share.flush_global_objects() {
                        pass.semaphore_waits = Some(op);
                    }
                    pass.use_immediate_buffer(immediate_buffer);
                    pass.submit(&queue);
                    old_frames.push(pass);
                } else {
                    log::error!("Worker received WorkerTask::EndPass when no active pass exists");
                    panic!()
                }
            }

            WorkerTask::UseStaticMesh(mesh_id) => {
                if let Some(pass) = &mut current_pass {
                    pass.static_meshes.push(mesh_id);
                } else {
                    log::error!("Worker received WorkerTask::UseStaticMesh when no active pass exists");
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

    pipeline: Arc<dyn EmulatorPipeline>,
    pass: Box<dyn EmulatorPipelinePass>,
    outputs: Vec<Box<dyn EmulatorOutput>>,

    immediate_buffer: Option<Box<ImmediateBuffer>>,
    static_meshes: Vec<StaticMeshId>,
    shaders: Vec<ShaderId>,

    semaphore_waits: Option<SemaphoreOp>,

    pre_cmd: vk::CommandBuffer,
    post_cmd: vk::CommandBuffer,

    end_fence: Option<vk::Fence>,
}

impl PassState {
    fn new(pipeline: Arc<dyn EmulatorPipeline>, mut pass: Box<dyn EmulatorPipelinePass>, device: Arc<DeviceContext>, queue: &Queue, share: Arc<Share>, pool: Rc<RefCell<WorkerObjectPool>>) -> Self {
        let mut object_pool = PooledObjectProvider::new(share.clone(), pool);

        let pre_cmd = object_pool.get_begin_command_buffer().unwrap();
        let post_cmd = object_pool.get_begin_command_buffer().unwrap();

        pass.init(queue, &mut object_pool);

        Self {
            share,
            device,
            object_pool,

            pipeline,
            pass,
            outputs: Vec::with_capacity(8),

            immediate_buffer: None,
            static_meshes: Vec::new(),
            shaders: Vec::new(),
            semaphore_waits: None,

            pre_cmd,
            post_cmd,

            end_fence: None,
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
        let wait_infos: &[vk::SemaphoreSubmitInfo] = if let Some(wait) = self.semaphore_waits.as_ref() {
            alloc.alloc([
                vk::SemaphoreSubmitInfo::builder()
                    .semaphore(wait.semaphore.get_handle())
                    .value(wait.value.unwrap_or(0))
                    .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
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

impl Drop for PassState {
    fn drop(&mut self) {
        if let Some(immediate_buffer) = self.immediate_buffer.take() {
            self.share.return_immediate_buffer(immediate_buffer);
        }
        for static_mesh in &self.static_meshes {
            self.share.dec_static_mesh(*static_mesh);
        }
        for shader in &self.shaders {
            self.pipeline.dec_shader_used(*shader);
        }
    }
}