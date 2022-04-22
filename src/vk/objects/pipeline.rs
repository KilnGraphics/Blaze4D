use ash::vk;

pub struct PipelineInstanceData {
    handle: vk::Pipeline,
}

impl PipelineInstanceData {
    pub unsafe fn get_handle(&self) -> vk::Pipeline {
        self.handle
    }
}