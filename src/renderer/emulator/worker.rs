use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use ash::prelude::VkResult;
use ash::vk;
use bumpalo::Bump;

use crate::device::device::Queue;

use crate::renderer::emulator::pass::PassId;
use crate::renderer::emulator::immediate::ImmediateBuffer;
use crate::renderer::emulator::pipeline::{EmulatorOutput, EmulatorPipeline, EmulatorPipelinePass, PipelineTask};

use crate::prelude::*;
use crate::renderer::emulator::global_objects::{GlobalImage, GlobalMesh};
use crate::renderer::emulator::mc_shaders::ShaderId;
use crate::renderer::emulator::share::{NextTaskResult, Share};
use crate::renderer::emulator::staging::StagingAllocationId;

pub(super) enum WorkerTask {
    StartPass(PassId, Arc<dyn EmulatorPipeline>, Box<dyn EmulatorPipelinePass + Send>),
    EndPass(Box<ImmediateBuffer>),
    UseStaticMesh(StaticMeshId),
    UseStaticImage(StaticImageId),
    UseShader(ShaderId),
    UseOutput(Box<dyn EmulatorOutput + Send>),
    PipelineTask(PipelineTask),
    WriteGlobalMesh(GlobalMeshWrite, bool),
    WriteGlobalImage(GlobalImageWrite, bool),
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
        share.worker_update();

        old_frames.retain(|old: &PassState| {
            !old.is_complete()
        });

        let task = match share.try_get_next_task_timeout(Duration::from_micros(500)) {
            NextTaskResult::Ok(task) => task,
            NextTaskResult::Timeout => continue,
        };

        match task {
            WorkerTask::StartPass(id, pipeline, pass) => {
                if current_pass.is_some() {
                    log::error!("Worker received WorkerTask::StartPass when a pass is already running");
                    panic!()
                }
                let state = PassState::new(id, pipeline, pass, device.clone(), &queue, share.clone(), pool.clone());
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

            WorkerTask::UseStaticMesh(mesh_id) => {
                if let Some(pass) = &mut current_pass {
                    pass.static_meshes.push(mesh_id);
                } else {
                    log::error!("Worker received WorkerTask::UseStaticMesh when no active pass exists");
                    panic!()
                }
            }

            WorkerTask::UseStaticImage(image_id) => {
                if let Some(pass) = &mut current_pass {
                    pass.static_images.push(image_id);
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

            WorkerTask::WriteGlobalImage(write, uninit) => {
                if let Some(current_pass) = &current_pass {
                    if current_pass.pass_id > write.after_pass {
                        get_or_create_recorder(&mut current_global_recorder, &share, &pool).record_global_image_write(write, uninit);
                    } else {
                        get_or_create_recorder(&mut next_global_recorder, &share, &pool).record_global_image_write(write, uninit);
                    }
                } else {
                    get_or_create_recorder(&mut next_global_recorder, &share, &pool).record_global_image_write(write, uninit);
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

    pass_id: PassId,

    pipeline: Arc<dyn EmulatorPipeline>,
    pass: Box<dyn EmulatorPipelinePass>,
    outputs: Vec<Box<dyn EmulatorOutput>>,

    immediate_buffer: Option<Box<ImmediateBuffer>>,
    static_meshes: Vec<StaticMeshId>,
    static_images: Vec<StaticImageId>,
    shaders: Vec<ShaderId>,

    pre_cmd: vk::CommandBuffer,
    post_cmd: vk::CommandBuffer,

    end_fence: Option<vk::Fence>,

    gob: Option<GlobalObjectsRecorder>,
}

impl PassState {
    fn new(pass_id: PassId, pipeline: Arc<dyn EmulatorPipeline>, mut pass: Box<dyn EmulatorPipelinePass>, device: Arc<DeviceContext>, queue: &Queue, share: Arc<Share>, pool: Rc<RefCell<WorkerObjectPool>>, placeholder_image: vk::ImageView, placeholder_id: StaticImageId) -> Self {
        let mut object_pool = PooledObjectProvider::new(share.clone(), pool);

        let pre_cmd = object_pool.get_begin_command_buffer().unwrap();
        let post_cmd = object_pool.get_begin_command_buffer().unwrap();

        pass.init(queue, &mut object_pool, placeholder_image);

        Self {
            share,
            device,
            object_pool,

            pass_id,

            pipeline,
            pass,
            outputs: Vec::with_capacity(8),

            immediate_buffer: None,
            static_meshes: Vec::new(),
            static_images: vec![placeholder_id],
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
        for static_mesh in &self.static_meshes {
            self.share.dec_static_mesh(*static_mesh);
        }
        for static_image in &self.static_images {
            self.share.dec_static_image(*static_image);
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
            let src_size = image.get_image_size();
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
                        device.vk().cmd_pipeline_barrier2(self.cmd, &info);
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
            let info = vk::DependencyInfo::builder()
                .buffer_memory_barriers(buffer_post_barriers.as_slice())
                .image_memory_barriers(image_post_barriers.as_slice());

            unsafe {
                device.vk().cmd_pipeline_barrier2(self.cmd, &info);
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
        self.staging_barriers.push(vk::BufferMemoryBarrier2::builder()
            .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .src_access_mask(vk::AccessFlags2::TRANSFER_READ)
            .dst_stage_mask(vk::PipelineStageFlags2::HOST)
            .dst_access_mask(vk::AccessFlags2::HOST_WRITE)
            .buffer(buffer)
            .offset(offset)
            .size(size)
            .build()
        );
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
                self.share.get_device().vk().cmd_pipeline_barrier2(self.cmd, &info);
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
                self.share.get_device().vk().cmd_pipeline_barrier2(self.cmd, &info);
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