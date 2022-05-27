use std::any::Any;
use ash::vk;
use crate::device::transfer::{BufferReleaseOp, SyncId};
use crate::objects::sync::SemaphoreOps;
use crate::renderer::emulator::{MeshData, VertexFormatId};
use crate::UUID;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};
use crate::vk::objects::buffer::Buffer;

use crate::prelude::*;

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

pub(super) struct StaticMesh {
    buffer: Buffer,
    allocation: Option<Allocation>,
    vertex_format: VertexFormatId,
    first_index: u32,
    index_type: vk::IndexType,
    index_count: u32,

    pending_op: Option<(SyncId, BufferReleaseOp)>,

    /// The number of passes using this mesh. This is used to prevent destroying it while it is
    /// still in use.
    used_counter: u64,
    marked: bool,
}

impl StaticMesh {
    pub(super) fn new(device: &DeviceEnvironment, data: &MeshData, queue_family: u32) -> Self {
        let index_offset = Self::next_aligned(data.vertex_data.len(), data.get_index_size() as usize);
        let required_size = index_offset + data.index_data.len();

        let (buffer, allocation) = Self::create_buffer(device, required_size);

        let transfer = device.get_transfer();
        let op = transfer.prepare_buffer_acquire(buffer, None);
        transfer.acquire_buffer(op, SemaphoreOps::None).unwrap();

        let staging = transfer.request_staging_memory(required_size);
        unsafe {
            staging.write(data.vertex_data);
            staging.write_offset(data.index_data, index_offset);
        }
        drop(staging);

        let op = transfer.prepare_buffer_release(buffer, Some((
            vk::PipelineStageFlags2::VERTEX_INPUT | vk::PipelineStageFlags2::INDEX_INPUT,
            vk::AccessFlags2::VERTEX_ATTRIBUTE_READ | vk::AccessFlags2::INDEX_READ,
            queue_family
        )));
        let sync = transfer.release_buffer(op).unwrap();
        transfer.flush(sync);

        Self {
            buffer,
            allocation: Some(allocation),
            vertex_format: data.vertex_format_id,
            first_index: (index_offset / data.get_index_size() as usize) as u32,
            index_type: data.index_type,
            index_count: data.index_count,
            pending_op: Some((sync, op)),
            used_counter: 0,
            marked: false,
        }
    }

    pub(super) fn get_data(&self) -> (Buffer, u32, vk::IndexType, u32) {
        (self.buffer, self.first_index, self.index_type, self.index_count)
    }

    pub(super) fn inc_mesh(&mut self) -> Option<(SyncId, BufferReleaseOp)> {
        self.used_counter += 1;
        self.pending_op.take()
    }

    /// Decrements the used counter and if it hits 0 and this mesh is marked destroys the mesh.
    ///
    /// Returns true if the mesh has been destroyed.
    pub(super) fn dec_mesh(&mut self, device: &DeviceEnvironment) -> bool {
        if self.used_counter == 0 {
            log::error!("Called dec_mesh on mesh where the used counter is already 0");
            panic!()
        }

        self.used_counter -= 1;

        if self.is_unused() && self.marked {
            self.destroy(device);
            return true;
        }

        false
    }

    pub(super) fn is_unused(&self) -> bool {
        self.used_counter == 0
    }

    /// Marks the mesh for destruction and if its used counter is 0 destroys it.
    ///
    /// Returns true if the mesh has been destroyed.
    pub(super) fn mark_destroy(&mut self, device: &DeviceEnvironment) -> bool {
        self.marked = true;

        if self.is_unused() {
            self.destroy(device);
            return true;
        }

        false
    }

    pub(super) fn destroy(&mut self, device: &DeviceEnvironment) {
        if self.used_counter != 0 {
            log::warn!("Destroying static mesh despite used counter being {:?}", self.used_counter);
        }

        if let Some((sync, _)) = self.pending_op.take() {
            device.get_transfer().wait_for_complete(sync);
        }

        unsafe {
            device.vk().destroy_buffer(self.buffer.get_handle(), None);
        }

        if let Some(alloc) = self.allocation.take() {
            device.get_allocator().free(alloc);
        } else {
            log::error!("Called destroy on static mesh without allocation");
            panic!()
        }
    }

    fn create_buffer(device: &DeviceEnvironment, size: usize) -> (Buffer, Allocation) {
        let info = vk::BufferCreateInfo::builder()
            .size(size as vk::DeviceSize)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::INDEX_BUFFER)
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

    fn next_aligned(base: usize, alignment: usize) -> usize {
        let alignment = alignment as usize;
        let diff = base % alignment;
        if diff == 0 {
            base
        } else {
            base + (alignment - diff)
        }
    }
}