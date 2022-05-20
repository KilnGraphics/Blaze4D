use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use ash::vk;
use ash::vk::Handle;
use winit::event::VirtualKeyCode::H;
use crate::objects::id::SemaphoreId;

#[derive(Copy, Clone)]
pub struct Semaphore {
    id: SemaphoreId,
    handle: vk::Semaphore,
}

impl Semaphore {
    pub fn new(handle: vk::Semaphore) -> Self {
        Self {
            id: SemaphoreId::new(),
            handle,
        }
    }

    pub fn get_id(&self) -> SemaphoreId {
        self.id
    }

    pub fn get_handle(&self) -> vk::Semaphore {
        self.handle
    }
}

impl PartialEq for Semaphore {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for Semaphore {
}

impl PartialOrd for Semaphore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Semaphore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Hash for Semaphore {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Debug for Semaphore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("vkSemaphore(UUID: {:#016X}, Handle: {:#016X})", self.id.get_raw(), self.handle.as_raw()))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SemaphoreOp {
    pub semaphore: Semaphore,
    pub value: Option<u64>,
}

impl SemaphoreOp {
    pub fn new_binary(semaphore: Semaphore) -> Self {
        Self {
            semaphore,
            value: None,
        }
    }

    pub fn new_timeline(semaphore: Semaphore, value: u64) -> Self {
        Self {
            semaphore,
            value: Some(value),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SemaphoreOps {
    None,
    One(SemaphoreOp),
    Multiple(Box<[SemaphoreOp]>),
}

impl SemaphoreOps {
    pub fn single_binary(semaphore: Semaphore) -> Self {
        Self::One(SemaphoreOp::new_binary(semaphore))
    }

    pub fn single_timeline(semaphore: Semaphore, value: u64) -> Self {
        Self::One(SemaphoreOp::new_timeline(semaphore, value))
    }

    pub fn from_option(op: Option<SemaphoreOp>) -> Self {
        match op {
            None => Self::None,
            Some(op) => Self::One(op)
        }
    }

    pub fn as_slice(&self) -> &[SemaphoreOp] {
        match self {
            SemaphoreOps::None => &[],
            SemaphoreOps::One(op) => std::slice::from_ref(op),
            SemaphoreOps::Multiple(ops) => ops.as_ref(),
        }
    }
}