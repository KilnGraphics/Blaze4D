use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use ash::vk;
use crate::device::device::VkQueue;
use crate::objects::sync::{SemaphoreOp, SemaphoreOps};
use crate::prelude::DeviceContext;

use crate::UUID;

pub(super) struct RecorderManager {
    share: Rc<RefCell<Share>>,
}

struct Share {
    device: Arc<DeviceContext>,
    queue: VkQueue,

    command_pool: CommandBufferPool,

    active_recorders: HashMap<UUID, RecorderState>,
    completed_recorders: Vec<RecorderState>,

    weak: Weak<RefCell<Share>>,
}

impl Share {
    fn new(device: Arc<DeviceContext>, queue: VkQueue, weak: Weak<RefCell<Share>>) -> Self {
        let command_pool = CommandBufferPool::new(device.clone(), queue.get_queue_family_index());

        Self {
            device,
            queue,

            command_pool,

            active_recorders: HashMap::new(),
            completed_recorders: Vec::new(),

            weak,
        }
    }

    fn create_recorder(&mut self) -> RecorderRef {
        let id = self.make_id();
        let head = Rc::new(RefCell::new(RefChain::Head(self.weak.upgrade().unwrap(), id)));

        self.active_recorders.insert(id, RecorderState::new(&self.device, self.command_pool.get_buffer(), Rc::downgrade(&head)));

        RecorderRef { chain: head }
    }

    fn submit_release(&mut self, min_release: Option<u64>, release_semaphore: vk::Semaphore) -> Option<u64> {
        let max_release = self.completed_recorders.iter().fold(min_release, RecorderState::collect_max_release);

        let max_release = if let Some(max_release) = max_release {
            Some(self.collect_by_release(max_release))
        } else {
            max_release
        };

        let submit_infos = Vec::with_capacity(self.completed_recorders.len());

        todo!();

        max_release
    }

    fn collect_by_release(&mut self, mut max_release: u64) -> u64 {
        // Need to do this until drain_filter is stabilized
        loop {
            let mut marked = None;
            for (id, recorder) in &self.active_recorders {
                if let Some(min) = &recorder.min_release {
                    if *min <= max_release {
                        marked = Some(*id);
                        break;
                    }
                }
            }

            if let Some(id) = marked {
                let recorder = self.active_recorders.remove(&id).unwrap();
                max_release = recorder.collect_max_release(&Some(max_release)).unwrap();
                self.completed_recorders.push(recorder);
            } else {
                return max_release;
            }
        }
    }

    /// Called by the RefChain when it is dropped. This indicates that there are no more live
    /// references to the Recorder and it can be submitted.
    fn mark_dropped(&mut self, id: UUID) {
        let recorder = self.active_recorders.remove(&id).unwrap();
        self.completed_recorders.push(recorder);
    }

    fn merge_into(&mut self, from: UUID, to: UUID) {
        assert_ne!(from, to); // Sanity check

        let mut src = self.active_recorders.remove(&from).unwrap();
        self.active_recorders.get_mut(&to).unwrap().merge_from(&self.device, &mut src);
    }

    fn make_id(&self) -> UUID {
        loop {
            // We can actually avoid even the tiny chances of a collision here
            let id = UUID::new();
            if !self.active_recorders.contains_key(&id) {
                return id;
            }
        }
    }
}

pub(super) struct RecorderState {
    /// Reference to the head of the chain referencing this recorder.
    ///
    /// If this is [`None`] indicates that this is not a valid instance (for example after a merge).
    /// All the members will be properly initialized structs but the data they contain must be ignored.
    chain_head: Option<Weak<RefCell<RefChain>>>,

    active_buffer: vk::CommandBuffer,
    recorded_buffers: Vec<vk::CommandBuffer>,

    min_release: Option<u64>,
    max_release: Option<u64>,

    wait_ops: Vec<SemaphoreOp>,
    signal_ops: Vec<SemaphoreOp>,

    pending_buffer_barriers: Vec<vk::BufferMemoryBarrier2>,
    pending_image_barriers: Vec<vk::ImageMemoryBarrier2>,
}

impl RecorderState {
    fn new(device: &DeviceContext, buffer: vk::CommandBuffer, chain_head: Weak<RefCell<RefChain>>) -> Self {
        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device.vk().begin_command_buffer(buffer, &info)
        }.unwrap();

        Self {
            chain_head: Some(chain_head),

            active_buffer: buffer,
            recorded_buffers: Vec::new(),

            min_release: None,
            max_release: None,

            wait_ops: Vec::new(),
            signal_ops: Vec::new(),

            pending_buffer_barriers: Vec::new(),
            pending_image_barriers: Vec::new(),
        }
    }

    pub(super) fn get_command_buffer(&self) -> vk::CommandBuffer {
        self.active_buffer
    }

    pub(super) fn push_release(&mut self, release: u64) {
        self.min_release = Some(self.min_release.map_or(release, |old| std::cmp::min(old, release)));
        self.max_release = Some(self.max_release.map_or(release, |old| std::cmp::max(old, release)));
    }

    pub(super) fn add_wait_ops(&mut self, wait_ops: SemaphoreOps) {
        self.wait_ops.extend(wait_ops.as_slice())
    }

    pub(super) fn add_wait_op(&mut self, wait_op: SemaphoreOp) {
        self.wait_ops.push(wait_op)
    }

    pub(super) fn add_signal_ops(&mut self, signal_ops: SemaphoreOps) {
        self.signal_ops.extend(signal_ops.as_slice())
    }

    pub(super) fn add_signal_op(&mut self, signal_op: SemaphoreOp) {
        self.signal_ops.push(signal_op)
    }

    pub(super) fn get_buffer_barrier_cache(&mut self) -> &mut Vec<vk::BufferMemoryBarrier2> {
        &mut self.pending_buffer_barriers
    }

    pub(super) fn get_image_barrier_cache(&mut self) -> &mut Vec<vk::ImageMemoryBarrier2> {
        &mut self.pending_image_barriers
    }

    pub(super) fn flush_barriers(&mut self, device: &DeviceContext) {
        if !self.pending_buffer_barriers.is_empty() || !self.pending_image_barriers.is_empty() {
            let info = vk::DependencyInfo::builder()
                .buffer_memory_barriers(self.pending_buffer_barriers.as_slice())
                .image_memory_barriers(self.pending_image_barriers.as_slice());

            unsafe {
                device.vk().cmd_pipeline_barrier2(self.active_buffer, &info)
            };

            self.pending_buffer_barriers.clear();
            self.pending_image_barriers.clear();
        }
    }

    fn collect_min_release(&self, min_release: &Option<u64>) -> Option<u64> {
        match (&self.min_release, min_release) {
            (None, None) => None,
            (Some(val), None) => Some(*val),
            (None, Some(val)) => Some(*val),
            (Some(a), Some(b)) => Some(std::cmp::min(*a, *b))
        }
    }

    fn collect_max_release(&self, max_release: &Option<u64>) -> Option<u64> {
        match (&self.max_release, max_release) {
            (None, None) => None,
            (Some(val), None) => Some(*val),
            (None, Some(val)) => Some(*val),
            (Some(a), Some(b)) => Some(std::cmp::max(*a, *b))
        }
    }

    fn finish(&mut self, device: &DeviceContext) {
        unsafe {
            device.vk().end_command_buffer(self.active_buffer)
        }.unwrap();

        self.recorded_buffers.push(self.active_buffer);
    }

    fn merge_from(&mut self, device: &DeviceContext, other: &mut RecorderState) {
        other.chain_head = None;

        unsafe {
            device.vk().end_command_buffer(other.active_buffer)
        }.unwrap();

        self.recorded_buffers.reserve(other.recorded_buffers.len() + 1);
        self.recorded_buffers.extend(other.recorded_buffers.iter().cloned());
        self.recorded_buffers.push(other.active_buffer);

        self.wait_ops.extend(other.wait_ops.iter().cloned());
        self.signal_ops.extend(other.signal_ops.iter().cloned());

        self.pending_buffer_barriers.extend(other.pending_buffer_barriers.iter().cloned());
        self.pending_image_barriers.extend(other.pending_image_barriers.iter().cloned());

        let min_release = self.collect_min_release(&other.min_release);
        self.min_release = min_release;

        let max_release = self.collect_max_release(&other.max_release);
        self.max_release = max_release;
    }
}

impl Drop for RecorderState {
    fn drop(&mut self) {
        if let Some(head) = self.chain_head.take() {
            if let Some(head) = head.upgrade() {
                head.borrow_mut().mark_dead();
            }
        }
    }
}

#[derive(Clone)]
pub(super) struct RecorderRef {
    chain: Rc<RefCell<RefChain>>,
}

impl RecorderRef {
    pub(super) fn new_empty() -> Self {
        Self {
            chain: Rc::new(RefCell::new(RefChain::None))
        }
    }

    pub(super) fn get_recorder_id(&mut self) -> Option<UUID> {
        let (result, new_next) = self.chain.borrow_mut().get_head();

        if let Some(new_next) = new_next {
            self.chain = new_next;
        }

        result
    }

    /// Merges this reference with another reference. If both point to valid recorders the recorders
    /// will be merged as well.
    pub(super) fn merge_with(&mut self, other: &mut RecorderRef) {
        let (self_id, self_head) = self.get_merge_info();
        let (other_id, other_head) = other.get_merge_info();

        match (self_id, other_id) {
            (None, None) => {} // Nothing to do since the only way to overwrite a None is to change each ref individually.
            (Some(_), None) => {
                other.chain = self_head;
            },
            (None, Some(_)) => {
                self.chain = other_head;
            },
            (Some(self_id), Some(other_id)) => {
                if self_id != other_id {
                    other_head.borrow_mut().merge_into(self_head, self_id);
                }
            }
        }
    }

    /// Returns the id and a reference to the head element
    fn get_merge_info(&self) -> (Option<UUID>, Rc<RefCell<RefChain>>) {
        let (id, head) = self.chain.borrow_mut().get_head();
        let head = match head {
            None => self.chain.clone(),
            Some(head) => head,
        };

        (id, head)
    }
}

/// An entry in a chain point to the UUID of a [`RecorderState`]. This effectively forms a union
/// find data structure allowing states to be merged.
enum RefChain {
    None,
    Head(Rc<RefCell<Share>>, UUID),
    Chain(Rc<RefCell<RefChain>>),
}

impl RefChain {
    /// Called to retrieve the UUID stored in the head of the chain.
    ///
    /// If this element is not the head of the chain also returns a reference to the head. This
    /// allows callers to change their internal reference to the head and reduce the depth of the
    /// chain.
    fn get_head(&mut self) -> (Option<UUID>, Option<Rc<RefCell<RefChain>>>) {
        let result;
        let new_next;
        match self {
            RefChain::None => {
                return (None, None);
            }
            RefChain::Head(_, id) => {
                return (Some(*id), None);
            }
            RefChain::Chain(next) => {
                (result, new_next) = next.borrow_mut().get_head();

                if new_next.is_none() {
                    return (result, Some(next.clone()))
                }
            }
        }

        // We only get here if new_next is something we should use to update our own next pointer
        if let Some(new_next) = &new_next {
            *self = RefChain::Chain(new_next.clone());
        }

        (result, new_next)
    }

    /// Called by the RecorderState if it is dropped.
    fn mark_dead(&mut self) {
        match self {
            Self::Chain(_) => {
                panic!("Called mark_dead on Chain element!")
            },
            Self::None => {
                panic!("Called mark_dead on None element!")
            }
            _ => {}
        }
        *self = RefChain::None;
    }

    /// Merges this chain into another chain. The 2 chains must be disjoint and both must have live
    /// recorders.
    ///
    /// This function will merge the the owned recorder of this chain into the recorder of the passed
    /// chain.
    fn merge_into(&mut self, other_head: Rc<RefCell<RefChain>>, other_id: UUID) {
        match self {
            RefChain::Head(share, self_id) => {
                share.borrow_mut().merge_into(*self_id, other_id);
            },
            _ => {
                panic!("Called merge_into on non head chain element!");
            }
        }
        *self = RefChain::Chain(other_head);
    }
}

impl Drop for RefChain {
    fn drop(&mut self) {
        match self {
            RefChain::Head(share, id) => {
                // Inside the share we only ever use Weak or internally upgraded references. So this should never fail.
                share.borrow_mut().mark_dropped(*id);
            }
            _ => {}
        }
    }
}

struct CommandBufferPool {
    device: Arc<DeviceContext>,
    pool: vk::CommandPool,
    buffers: Vec<vk::CommandBuffer>,
}

impl CommandBufferPool {
    fn new(device: Arc<DeviceContext>, queue: u32) -> Self {
        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue);

        let pool = unsafe {
            device.vk().create_command_pool(&info, None)
        }.unwrap();

        Self {
            device,
            pool,
            buffers: Vec::new(),
        }
    }

    fn get_buffer(&mut self) -> vk::CommandBuffer {
        match self.buffers.first() {
            Some(buffer) => *buffer,
            None => {
                *self.buffers.first().unwrap()
            }
        }
    }

    fn return_buffer(&mut self, buffer: vk::CommandBuffer) {
        self.buffers.push(buffer);
    }

    fn allocate_buffers(&mut self) {
        let info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.pool)
            .command_buffer_count(16)
            .level(vk::CommandBufferLevel::PRIMARY);

        let buffers = unsafe {
            self.device.vk().allocate_command_buffers(&info)
        }.unwrap();

        self.buffers.extend(buffers)
    }
}

impl Drop for CommandBufferPool {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_command_pool(self.pool, None);
        }
    }
}