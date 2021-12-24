use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LockResult, Mutex, MutexGuard};

use crate::util::id::GlobalId;
use super::ObjectManager;

use ash::vk;

// Internal struct containing the semaphore payload and metadata
struct SyncData {
    semaphore: vk::Semaphore,
    last_access: u64,
}

impl SyncData {
    fn enqueue_access(&mut self, step_count: u64) -> AccessInfo {
        let begin_access = self.last_access;
        let end_access = begin_access + step_count;
        self.last_access = end_access;

        AccessInfo{
            semaphore: self.semaphore,
            begin_access,
            end_access,
        }
    }
}

// Internal implementation of the synchronization group
struct SynchronizationGroupImpl {
    group_id: GlobalId,
    sync_data: Mutex<SyncData>,
    manager: ObjectManager,
}

impl SynchronizationGroupImpl {
    fn new(manager: ObjectManager, semaphore: vk::Semaphore) -> Self {
        Self{ group_id: GlobalId::new(), sync_data: Mutex::new(SyncData{ semaphore, last_access: 0u64 }), manager }
    }

    fn get_group_id(&self) -> GlobalId {
        self.group_id
    }

    fn lock(&self) -> LockResult<MutexGuard<SyncData>> {
        self.sync_data.lock()
    }
}

impl Drop for SynchronizationGroupImpl {
    fn drop(&mut self) {
        self.manager.destroy_semaphore(self.sync_data.get_mut().unwrap().semaphore)
    }
}

impl PartialEq for SynchronizationGroupImpl {
    fn eq(&self, other: &Self) -> bool {
        self.group_id == other.group_id
    }
}

impl Eq for SynchronizationGroupImpl {
}

impl PartialOrd for SynchronizationGroupImpl {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.group_id.partial_cmp(&other.group_id)
    }
}

impl Ord for SynchronizationGroupImpl {
    fn cmp(&self, other: &Self) -> Ordering {
        self.group_id.cmp(&other.group_id)
    }
}

impl Debug for SynchronizationGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*format!("SynchronizationGroup({:#16X})", self.0.group_id.get_raw()))
    }
}

/// Public synchronization group api.
///
/// This is a smart pointer reference to an internal struct.
pub struct SynchronizationGroup(Arc<SynchronizationGroupImpl>);

impl SynchronizationGroup {
    pub(super) fn new(manager: ObjectManager, semaphore: vk::Semaphore) -> Self {
        Self(Arc::new(SynchronizationGroupImpl::new(manager, semaphore)))
    }

    /// Returns the object manager managing this synchronization group
    pub fn get_manager(&self) -> &ObjectManager {
        &self.0.manager
    }

    /// Enqueues an access to the resources protected by this group.
    ///
    /// `step_count` is the number of steps added to the semaphore payload.
    ///
    /// If access to multiple groups is needed simultaneously; accesses **must not** be queued
    /// individually but by using a synchronization group set. Not doing so may result in a
    /// deadlock when waiting for the semaphores.
    pub fn enqueue_access(&self, step_count: u64) -> AccessInfo {
        self.0.lock().unwrap().enqueue_access(step_count)
    }
}

impl Clone for SynchronizationGroup {
    fn clone(&self) -> Self {
        Self( self.0.clone() )
    }
}

impl PartialEq for SynchronizationGroup {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for SynchronizationGroup {
}

impl PartialOrd for SynchronizationGroup {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for SynchronizationGroup {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for SynchronizationGroup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.group_id.hash(state)
    }
}

/// Stores information for a single accesses queued up in a synchronization group.
pub struct AccessInfo {
    /// The timeline semaphore protecting the group.
    pub semaphore: vk::Semaphore,

    /// The value of the semaphore when the access may begin execution.
    pub begin_access: u64,

    /// The value the semaphore must have to signal the end of the access.
    pub end_access: u64,
}

pub struct SynchronizationGroupSet {
    groups: Box<[SynchronizationGroup]>,
}

impl SynchronizationGroupSet {
    pub fn new(groups: &std::collections::BTreeSet<SynchronizationGroup>) -> Self {
        // BTreeSet is required to guarantee the groups are sorted

        let collected : Vec<_> = groups.into_iter().map(|group| group.clone()).collect();
        Self{ groups: collected.into_boxed_slice() }
    }

    pub fn enqueue_access(&self, step_counts: &[u64]) -> Box<[AccessInfo]> {
        if self.groups.len() != step_counts.len() {
            panic!("Step counts length mismatch")
        }

        let mut guards = Vec::with_capacity(self.groups.len());

        for group in self.groups.iter() {
            guards.push(group.0.lock().unwrap())
        }

        let mut accesses = Vec::with_capacity(self.groups.len());

        for (i, mut guard) in guards.into_iter().enumerate() {
            accesses.push(guard.enqueue_access(*step_counts.get(i).unwrap()));
        }

        accesses.into_boxed_slice()
    }
}