use std::collections::HashMap;

use ash::vk;

use crate::vk::objects::buffer::Buffer;

use crate::prelude::*;

pub struct BufferStateTracker {
    buffers: HashMap<UUID, BufferState>,
}

impl BufferStateTracker {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }

    /// Registers a buffer into the tracker.
    ///
    /// The buffer is initialized to having no pending reads or writes.
    ///
    /// If the buffer is already registered [`Err`] is returned.
    pub fn register(&mut self, buffer: Buffer) -> Result<(), ()> {
        if self.buffers.contains_key(&buffer.get_id()) {
            return Err(());
        }
        self.buffers.insert(buffer.get_id(), BufferState::new(buffer));
        Ok(())
    }

    /// Updates the state of a buffer, records any required barriers and returns the handle of the
    /// buffer.
    ///
    /// If the buffer could not be found [`None`] is returned.
    pub fn update_state(&mut self, id: UUID, read: bool, write: bool, barriers: &mut Vec<vk::BufferMemoryBarrier2>) -> Option<vk::Buffer> {
        if let Some(buffer) = self.buffers.get_mut(&id) {
            buffer.update_state(read, write, barriers);
            Some(buffer.handle)
        } else {
            None
        }
    }

    /// Releases a registered buffer returning its handle and [`ash::vk::AccessFlags2`] representing
    /// any pending operations on the buffer.
    ///
    /// If the buffer could not be found [`None`] is returned.
    pub fn release(&mut self, id: UUID) -> Option<(vk::Buffer, vk::AccessFlags2)> {
        if let Some(buffer) = self.buffers.remove(&id) {
            let mut access_mask = vk::AccessFlags2::empty();
            if buffer.read_pending {
                access_mask |= vk::AccessFlags2::TRANSFER_READ;
            }
            if buffer.write_pending {
                access_mask |= vk::AccessFlags2::TRANSFER_WRITE;
            }

            Some((buffer.handle, access_mask))
        } else {
            None
        }
    }
}

struct BufferState {
    handle: vk::Buffer,
    read_pending: bool,
    write_pending: bool,
}

impl BufferState {
    fn new(buffer: Buffer) -> Self {
        Self {
            handle: buffer.get_handle(),
            read_pending: false,
            write_pending: false,
        }
    }

    fn update_state(&mut self, read: bool, write: bool, barriers: &mut Vec<vk::BufferMemoryBarrier2>) {
        let mut src_access_mask = vk::AccessFlags2::empty();
        if read && self.write_pending {
            src_access_mask |= vk::AccessFlags2::TRANSFER_WRITE;
        }
        if write && (self.write_pending || self.read_pending) {
            src_access_mask |= vk::AccessFlags2::TRANSFER_WRITE | vk::AccessFlags2::TRANSFER_READ;
        }
        self.read_pending |= read;
        self.write_pending |= write;

        if src_access_mask != vk::AccessFlags2::empty() {
            barriers.push(vk::BufferMemoryBarrier2::builder()
                .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .src_access_mask(src_access_mask)
                .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .dst_access_mask(vk::AccessFlags2::TRANSFER_READ | vk::AccessFlags2::TRANSFER_WRITE)
                .buffer(self.handle)
                .offset(0)
                .size(vk::WHOLE_SIZE)
                .build()
            );
        }
    }
}