/// TODO: Documentation
pub mod data_type {
    use std::mem::size_of;

    const FLOAT: usize = size_of::<F32>();
}

/// The format in which vertex data is stored. For example if you where storing position and color per Vertex, You may store it as 2 vec3's
pub struct VertexFormat {}
