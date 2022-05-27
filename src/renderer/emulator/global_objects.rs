use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use ash::vk;

use crate::device::device::Queue;
use crate::device::transfer::{BufferReleaseOp, BufferTransferRanges, SyncId};
use crate::objects::sync::{Semaphore, SemaphoreOp, SemaphoreOps};
use crate::renderer::emulator::{MeshData, VertexFormatId};
use crate::UUID;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};
use crate::vk::objects::buffer::Buffer;

use crate::prelude::*;

/// Manages objects which are global to all passes of a emulator renderer.
///
/// This includes things like static meshes or static textures.
pub(super) struct GlobalObjects {
    device: DeviceEnvironment,
    queue_family: u32,
    data: Mutex<Data>,
}

impl GlobalObjects {
    /// Creates a new instance.
    ///
    /// The passed queue is the queue used for rendering. All created objects will be transferred to
    /// this queue family when accessed for rendering.
    pub(super) fn new(device: DeviceEnvironment, queue: Queue) -> Self {
        let queue_family= queue.get_queue_family_index();
        let data = Data::new(&device, queue);

        Self {
            device,
            queue_family,
            data: Mutex::new(data),
        }
    }

    /// Should be called regularly by the worker thread.
    ///
    /// This runs more heavy weight operations which have been deferred.
    ///
    /// Not calling this function will **never** cause blocked state. However it might cause
    /// inefficient performance or resource usage for example by not destroying unused objects.
    pub(super) fn update(&self) {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned data mutex in GlobalObjects::update");
            panic!()
        }).update(&self.device);
    }

    pub(super) fn create_static_mesh(&self, data: &MeshData) -> StaticMeshId {
        let index_offset = Self::next_aligned(data.index_data.len(), data.get_index_size() as usize);
        let required_size = index_offset + data.index_data.len();

        let (buffer, allocation) = StaticMesh::create_buffer(&self.device, required_size);

        let transfer = self.device.get_transfer();
        let op = transfer.prepare_buffer_acquire(buffer, None);
        transfer.acquire_buffer(op, SemaphoreOps::None).unwrap_or_else(|err| {
            log::error!("Failed to make buffer available to transfer in GlobalObjects::create_static_mesh. {:?}", err);
            panic!();
        });

        let staging = transfer.request_staging_memory(required_size);
        unsafe {
            staging.write(data.vertex_data);
            staging.write_offset(data.index_data, index_offset);
            staging.copy_to_buffer(buffer, BufferTransferRanges::new_single(
                0,
                0,
                required_size as vk::DeviceSize
            ));
        }
        drop(staging);

        let op = transfer.prepare_buffer_release(buffer, Some((
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::AccessFlags2::MEMORY_READ,
            self.queue_family
        )));
        let sync = transfer.release_buffer(op).unwrap_or_else(|err| {
            log::error!("Failed release buffer from transfer in GlobalObjects::create_static_mesh. {:?}", err);
            panic!();
        });

        let draw_info = StaticMeshDrawInfo {
            buffer,
            first_index: (index_offset / (data.get_index_size() as usize)) as u32,
            index_type: data.index_type,
            index_count: data.index_count
        };

        let static_mesh = StaticMesh {
            buffer,
            allocation,
            draw_info,
            used_counter: 0,
            marked: false
        };

        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::create_static_mesh!");
            panic!();
        }).add_static_mesh(&self.device, static_mesh, sync, op)
    }

    pub(super) fn mark_static_mesh(&self, id: StaticMeshId) {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::mark_static_mesh!");
            panic!();
        }).mark_static_mesh(id)
    }

    pub(super) fn inc_static_mesh(&self, id: StaticMeshId) -> StaticMeshDrawInfo {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::inc_static_mesh!");
            panic!();
        }).inc_get_static_mesh(id)
    }

    pub(super) fn dec_static_mesh(&self, id: StaticMeshId) {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::dec_static_mesh!");
            panic!();
        }).dec_static_mesh(id)
    }

    pub(super) fn create_static_texture(&self) {
        todo!()
    }

    pub(super) fn mark_static_texture(&self) {
        todo!()
    }

    /// Flushes any pending operations which need to be executed on global objects.
    ///
    /// Calling this function ensures that all objects created or manipulated before this function
    /// is called are ready to be used by a pass. If [`Some`] is returned any caller must
    /// additionally wait on the semaphore before using any global object.
    ///
    /// This is a heavyweight operation and should ideally only be called from the worker thread.
    pub(super) fn flush(&self) -> Option<SemaphoreOp> {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::flush!");
            panic!();
        }).flush(&self.device)
    }

    /// Returns the next address after and including the base address which has the specified
    /// alignment.
    fn next_aligned(base: usize, alignment: usize) -> usize {
        let diff = base % alignment;
        if diff == 0 {
            base
        } else {
            base + (alignment - diff)
        }
    }
}

impl Drop for GlobalObjects {
    fn drop(&mut self) {
        self.data.get_mut().unwrap_or_else(|_| {
            log::error!("Poisoned data mutex while destroying GlobalObjects!");
            panic!()
        }).destroy(&self.device);
    }
}

struct Data {
    queue: Queue,

    semaphore: Semaphore,
    semaphore_current_value: u64,

    command_pool: vk::CommandPool,
    available_command_buffers: Vec<vk::CommandBuffer>,
    pending_sync: Option<SyncId>,
    pending_command_buffer: Option<vk::CommandBuffer>,
    submitted_command_buffers: VecDeque<(u64, vk::CommandBuffer)>,

    static_meshes: HashMap<StaticMeshId, StaticMesh>,
    droppable_static_meshes: Vec<StaticMesh>,
}

impl Data {
    fn new(device: &DeviceEnvironment, queue: Queue) -> Self {
        let semaphore = Self::create_semaphore(device);
        let command_pool = Self::create_command_pool(device, queue.get_queue_family_index());

        Self {
            queue,

            semaphore: Semaphore::new(semaphore),
            semaphore_current_value: 0,

            command_pool,
            available_command_buffers: Vec::new(),
            pending_sync: None,
            pending_command_buffer: None,
            submitted_command_buffers: VecDeque::new(),

            static_meshes: HashMap::new(),
            droppable_static_meshes: Vec::new(),
        }
    }

    fn add_static_mesh(&mut self, device: &DeviceEnvironment, static_mesh: StaticMesh, sync: SyncId, op: BufferReleaseOp) -> StaticMeshId {
        self.push_sync(sync);

        if let Some(barrier) = op.make_barrier() {
            let cmd = self.get_begin_pending_command_buffer(device);

            let info = vk::DependencyInfo::builder()
                .buffer_memory_barriers(std::slice::from_ref(&barrier));

            unsafe {
                device.vk().cmd_pipeline_barrier2(cmd, &info);
            }
        }

        let mesh_id = StaticMeshId::new();
        if self.static_meshes.insert(mesh_id, static_mesh).is_some() {
            log::error!("UUID collision");
            panic!();
        }

        mesh_id
    }

    fn mark_static_mesh(&mut self, mesh_id: StaticMeshId) {
        let mut drop = false;
        if let Some(static_mesh) = self.static_meshes.get_mut(&mesh_id) {
            static_mesh.marked = true;
            if static_mesh.is_unused() {
                drop = true;
            }
        } else {
            log::error!("Failed to find mesh with id {:?} in Data::mark_static_mesh", mesh_id);
            panic!()
        }

        if drop {
            let static_mesh = self.static_meshes.remove(&mesh_id).unwrap();
            self.droppable_static_meshes.push(static_mesh);
        }
    }

    fn inc_get_static_mesh(&mut self, mesh_id: StaticMeshId) -> StaticMeshDrawInfo {
        if let Some(static_mesh) = self.static_meshes.get_mut(&mesh_id) {
            if !static_mesh.inc() {
                log::error!("Inc was called on marked static mesh!");
                panic!();
            }

            static_mesh.draw_info.clone()
        } else {
            log::error!("Failed to find mesh with id {:?} in Data::inc_get_static_mesh", mesh_id);
            panic!()
        }
    }

    fn dec_static_mesh(&mut self, mesh_id: StaticMeshId) {
        let mut drop = false;
        if let Some(static_mesh) = self.static_meshes.get_mut(&mesh_id) {
            if static_mesh.dec() {
                drop = true;
            }
        } else {
            log::error!("Failed to find mesh with id {:?} in Data::dec_static_mesh", mesh_id);
            panic!()
        }

        if drop {
            let static_mesh = self.static_meshes.remove(&mesh_id).unwrap();
            self.droppable_static_meshes.push(static_mesh);
        }
    }

    fn update(&mut self, device: &DeviceEnvironment) {
        let current_value = unsafe {
            device.vk().get_semaphore_counter_value(self.semaphore.get_handle())
        }.unwrap_or_else(|err| {
            log::error!("vkGetSemaphoreCounterValue returned {:?} in Data::update", err);
            panic!()
        });

        while let Some((value, cmd)) = self.submitted_command_buffers.pop_front() {
            if current_value >= value {
                self.available_command_buffers.push(cmd);
            } else {
                self.submitted_command_buffers.push_front((value, cmd));
                break;
            }
        }

        while let Some(static_mesh) = self.droppable_static_meshes.pop() {
            static_mesh.destroy(device);
        }
    }

    fn flush(&mut self, device: &DeviceEnvironment) -> Option<SemaphoreOp> {
        let sync_id = self.pending_sync.take();
        if let Some(sync_id) = sync_id {
            device.get_transfer().wait_for_submit(sync_id);
        }

        if let Some(cmd) = self.pending_command_buffer.take() {
            unsafe {
                device.vk().end_command_buffer(cmd)
            }.unwrap_or_else(|err| {
                log::error!("vkEndCommandBuffer returned {:?} in Data::flush!", err);
                panic!();
            });

            self.semaphore_current_value += 1;
            let signal_value = self.semaphore_current_value;

            let wait_info = sync_id.map(|sync_id| {
                let wait_op = device.get_transfer().generate_wait_semaphore(sync_id);

                vk::SemaphoreSubmitInfo::builder()
                    .semaphore(wait_op.semaphore.get_handle())
                    .value(wait_op.value.unwrap_or(0))
                    .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            });

            let command_infos = [
                vk::CommandBufferSubmitInfo::builder()
                    .command_buffer(cmd)
                    .build(),
            ];

            let signal_infos = [
                vk::SemaphoreSubmitInfo::builder()
                    .semaphore(self.semaphore.get_handle())
                    .value(signal_value)
                    .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                    .build(),
            ];

            let info = vk::SubmitInfo2::builder()
                .wait_semaphore_infos(wait_info.as_ref().map_or(&[], |r| std::slice::from_ref(r)))
                .command_buffer_infos(&command_infos)
                .signal_semaphore_infos(&signal_infos);

            unsafe {
                self.queue.submit_2(std::slice::from_ref(&info), None)
            }.unwrap_or_else(|err| {
                log::error!("vkQueueSubmit2 returned {:?} in Data::flush!", err);
                panic!();
            });

            self.submitted_command_buffers.push_back((signal_value, cmd));

            Some(SemaphoreOp::new_timeline(self.semaphore, signal_value))
        } else {
            // We still have to wait since 2 callers could call right after each other in which case
            // the second one would not get a wait op despite the submission not having completed yet.
            Some(SemaphoreOp::new_timeline(self.semaphore, self.semaphore_current_value))
        }
    }

    fn push_sync(&mut self, sync: SyncId) {
        self.pending_sync = self.pending_sync.map_or(Some(sync), |old| Some(std::cmp::max(old, sync)));
    }

    fn get_begin_pending_command_buffer(&mut self, device: &DeviceEnvironment) -> vk::CommandBuffer {
        if let Some(cmd) = self.pending_command_buffer {
            cmd
        } else {
            let cmd = self.get_begin_command_buffer(device);
            self.pending_command_buffer = Some(cmd);
            cmd
        }
    }

    fn get_begin_command_buffer(&mut self, device: &DeviceEnvironment) -> vk::CommandBuffer {
        let cmd = self.get_command_buffer(device);

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device.vk().begin_command_buffer(cmd, &info)
        }.unwrap_or_else(|err| {
            log::error!("vkBeginCommandBuffer returned {:?} in Data::get_begin_command_buffer!", err);
            panic!("");
        });

        cmd
    }

    fn get_command_buffer(&mut self, device: &DeviceEnvironment) -> vk::CommandBuffer {
        if let Some(cmd) = self.available_command_buffers.pop() {
            return cmd;
        } else {
            let info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(4);

            let new_buffers = unsafe {
                device.vk().allocate_command_buffers(&info)
            }.unwrap_or_else(|err| {
                log::error!("vkAllocateCommandBuffers returned {:?} in Data::get_command_buffer", err);
                panic!();
            });

            self.available_command_buffers.extend(new_buffers);

            self.available_command_buffers.pop().unwrap()
        }
    }

    fn create_semaphore(device: &DeviceEnvironment) -> vk::Semaphore {
        let mut type_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(0);

        let info = vk::SemaphoreCreateInfo::builder()
            .push_next(&mut type_info);

        unsafe {
            device.vk().create_semaphore(&info, None)
        }.unwrap_or_else(|err| {
            log::error!("vkCreateSemaphore returned {:?} while trying to create GlobalObjects semaphore!", err);
            panic!()
        })
    }

    fn create_command_pool(device: &DeviceEnvironment, queue_family: u32) -> vk::CommandPool {
        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue_family);

        unsafe {
            device.vk().create_command_pool(&info, None)
        }.unwrap_or_else(|err| {
            log::error!("vkCreateCommandPool returned {:?} in Data::create_command_pool!", err);
            panic!()
        })
    }

    fn destroy(&mut self, device: &DeviceEnvironment) {
        unsafe {
            device.vk().destroy_semaphore(self.semaphore.get_handle(), None);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct StaticMeshId(UUID);

impl StaticMeshId {
    pub fn new() -> Self {
        Self(UUID::new())
    }

    pub fn from_raw(raw: UUID) -> Self {
        Self(raw)
    }

    pub fn get_raw(&self) -> UUID {
        self.0
    }
}

#[derive(Copy, Clone)]
pub struct StaticMeshDrawInfo {
    pub buffer: Buffer,
    pub first_index: u32,
    pub index_type: vk::IndexType,
    pub index_count: u32,
}

pub struct StaticMesh {
    buffer: Buffer,
    allocation: Allocation,
    draw_info: StaticMeshDrawInfo,

    used_counter: u32,
    marked: bool,
}

impl StaticMesh {
    /// Attempts to increment the used counter.
    ///
    /// If the mesh is marked the counter is not incremented and false is returned.
    fn inc(&mut self) -> bool {
        if self.marked {
            return false;
        }

        self.used_counter += 1;
        true
    }

    /// Decrements the used counter.
    ///
    /// If the mesh is marked and the counter decrements to 0 true is returned indicating that the
    /// mesh can be destroyed.
    fn dec(&mut self) -> bool {
        if self.used_counter == 0 {
            log::error!("Used counter is already 0 when calling StaticMesh::dec");
            panic!()
        }

        self.used_counter -= 1;

        if self.marked && self.is_unused() {
            return true;
        }
        false
    }

    /// Returns true if the mesh used counter is 0
    fn is_unused(&self) -> bool {
        self.used_counter == 0
    }

    fn destroy(self, device: &DeviceEnvironment) {
        if self.used_counter != 0 {
            log::warn!("Destroying static mesh despite used counter being {:?}", self.used_counter);
        }

        unsafe {
            device.vk().destroy_buffer(self.buffer.get_handle(), None);
        }

        device.get_allocator().free(self.allocation);
    }

    fn create_buffer(device: &DeviceEnvironment, size: usize) -> (Buffer, Allocation) {
        let info = vk::BufferCreateInfo::builder()
            .size(size as vk::DeviceSize)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device.vk().create_buffer(&info, None)
        }.unwrap_or_else(|err| {
            log::error!("vkCreateBuffer returned {:?} when trying to create buffer for static mesh of size {:?}", err, size);
            panic!()
        });

        let alloc = device.get_allocator().allocate_buffer_memory(buffer, &AllocationStrategy::AutoGpuOnly)
            .unwrap_or_else(|err| {
                log::error!("allocate_buffer_memory failed with {:?} when trying to allocate memory for static mesh buffer of size {:?}", err, size);
                panic!()
            });

        unsafe {
            device.vk().bind_buffer_memory(buffer, alloc.memory(), alloc.offset())
        }.unwrap_or_else(|err| {
            log::error!("vkBindBufferMemory returned {:?} when trying to bind memory for static mesh buffer of size {:?}", err, size);
            panic!()
        });

        (Buffer::new(buffer), alloc)
    }
}