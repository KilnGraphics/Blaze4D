use ash::vk;

struct VertexFormat {
    vertex_size: u32,
    position: VertexFormatEntry,
    normal: Option<VertexFormatEntry>,
    color: Option<VertexFormatEntry>,
    uv: Option<VertexFormatEntry>,
}

struct VertexFormatEntry {
    format: vk::Format,
    offset: u32,
}

struct Pipeline {

}