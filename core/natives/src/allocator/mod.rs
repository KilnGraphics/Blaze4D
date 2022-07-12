use std::sync::Arc;
use ash::vk;
use crate::prelude::DeviceFunctions;

mod vma;

pub struct Allocator {
    vma_allocator: vma::Allocator,

    functions: Arc<DeviceFunctions>,
}

impl Allocator {
    pub fn new(functions: Arc<DeviceFunctions>) -> Result<Self, vk::Result> {
        let vma_allocator = vma::Allocator::new(&functions, vma::AllocatorCreateFlags::empty())?;

        Ok(Self {
            vma_allocator,
            functions
        })
    }
}