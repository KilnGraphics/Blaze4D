use ash::vk;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SemaphoreOp {
    pub semaphore: vk::Semaphore,
    pub value: Option<u64>,
}

impl SemaphoreOp {
    pub fn new_binary(semaphore: vk::Semaphore) -> Self {
        Self {
            semaphore,
            value: None,
        }
    }

    pub fn new_timeline(semaphore: vk::Semaphore, value: u64) -> Self {
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
    pub fn single_binary(semaphore: vk::Semaphore) -> Self {
        Self::One(SemaphoreOp::new_binary(semaphore))
    }

    pub fn single_timeline(semaphore: vk::Semaphore, value: u64) -> Self {
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