use ash::vk;

pub struct VertexFormat {
    vertex_size: u32,
    position: VertexFormatEntry,
    normal: Option<VertexFormatEntry>,
    color: Option<VertexFormatEntry>,
    uv: Option<VertexFormatEntry>,
}

pub struct VertexFormatEntry {
    format: vk::Format,
    offset: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct PipelineId(u32);

impl PipelineId {
    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }

    pub fn get_raw(&self) -> u32 {
        self.0
    }
}

pub(super) struct PipelineManager {

}

impl PipelineManager {
    pub(super) fn new() -> Self {
        todo!()
    }
}

pub struct Pipeline {
    drop_after_frame: Option<u64>,
    pipeline: vk::Pipeline,
}

