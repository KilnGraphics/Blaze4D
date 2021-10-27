use ash::vk::{VertexInputAttributeDescription, VertexInputBindingDescription};

const VK_PADDING_FORMAT: i32 = -1;

/// TODO: Documentation
pub mod data_type {
    use std::mem::size_of;

    const UNSIGNED_BYTE: usize = size_of::<u8>();
    const BYTE: usize = size_of::<i8>();
    const UNSIGNED_SHORT: usize = size_of::<u16>();
    const SHORT: usize = size_of::<i16>();
    const UNSIGNED_INT: usize = size_of::<u32>();
    const INT: usize = size_of::<i32>();
    const FLOAT: usize = size_of::<f32>();
}

/// A raw Element of a VertexFormat.
pub struct VertexFormatElement {
    vk_type: i32,
    byte_length: usize,
}

/// The format in which vertex data is stored. For example if you where storing position and color per Vertex, You may store it as 2 vec3's
pub struct VertexFormat {
    pub elements: Vec<VertexFormatElement>,
    pub attributes: VertexInputAttributeDescription,
    pub bindings: VertexInputBindingDescription,
}

impl VertexFormat {
    pub fn new(elements: Vec<VertexFormatElement>) -> VertexFormat {
        let mut corrected_length = 0;
        for element in elements.iter() {
            if element.vk_type != VK_PADDING_FORMAT {
                corrected_length += 1;
            }
        }

        VertexFormat {
            elements,
            attributes: Default::default(), // TODO: finish
            bindings: Default::default(),
        }
    }
}
