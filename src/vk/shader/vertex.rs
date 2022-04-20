use ash::vk::{
    Format, PipelineVertexInputStateCreateInfo, VertexInputAttributeDescription,
    VertexInputBindingDescription, VertexInputRate,
};

pub mod data_type {
    use std::mem::size_of;

    pub const UNSIGNED_BYTE: usize = size_of::<u8>();
    pub const BYTE: usize = size_of::<i8>();
    pub const UNSIGNED_SHORT: usize = size_of::<u16>();
    pub const SHORT: usize = size_of::<i16>();
    pub const UNSIGNED_INT: usize = size_of::<u32>();
    pub const INT: usize = size_of::<i32>();
    pub const FLOAT: usize = size_of::<f32>();
}

#[derive(Default)]
pub struct VertexFormatBuilder {
    elements: Vec<VertexFormatElement>,
}

impl VertexFormatBuilder {
    pub fn new() -> VertexFormatBuilder {
        VertexFormatBuilder {
            elements: vec![]
        }
    }

    pub fn element(mut self, data_type: usize, amount: i32) -> VertexFormatBuilder {
        self.elements.push(VertexFormatElement {
            vk_type: Some(match data_type {
                data_type::FLOAT =>
                    match amount {
                        3 => Format::R32G32B32_SFLOAT,
                        _ => panic!("Cannot Handle '{}' Floats", amount)
                    }

                _ => panic!("Cannot Handle DataType '{}'", data_type)
            }),
            byte_length: data_type * amount as usize,
        });

        self
    }

    pub fn build(mut self) -> VertexFormat {
        VertexFormat::new(self.elements)
    }
}

/// A raw Element of a VertexFormat.
pub struct VertexFormatElement {
    vk_type: Option<Format>,
    byte_length: usize,
}

/// The format in which vertex data is stored. For example if you where storing position and color per Vertex, You may store it as 2 vec3's
pub struct VertexFormat {
    pub elements: Vec<VertexFormatElement>,
    pub vertex_stage_pipeline_info: PipelineVertexInputStateCreateInfo,
    pub size: u32,
}

impl VertexFormat {
    pub fn new(elements: Vec<VertexFormatElement>) -> VertexFormat {
        let mut corrected_length = 0;
        for element in elements.iter() {
            if element.vk_type.is_some() {
                corrected_length += 1;
            }
        }

        let mut attributes: Vec<VertexInputAttributeDescription> = vec![];
        let mut offset = 0;
        let mut element_id = 0;
        for element in elements.iter() {
            // Check if the element is just padding.
            if element.vk_type.is_some() {
                let attribute = VertexInputAttributeDescription::builder()
                    .binding(0)
                    .location(element_id)
                    .format(element.vk_type.unwrap())
                    .offset(offset);
                attributes.push(attribute.build()); // Build is done here so the compiler has a chance to warn about dropped items
                element_id += 1;
            }
            offset += element.byte_length as u32;
        }

        let binding = VertexInputBindingDescription::builder()
            .binding(0)
            .stride(offset)
            .input_rate(VertexInputRate::VERTEX);

        let bindings = vec![binding.build()];

        let pipeline_create_info = PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&*attributes)
            .vertex_binding_descriptions(&*bindings)
            .build();

        VertexFormat {
            elements,
            vertex_stage_pipeline_info: pipeline_create_info,
            size: offset,
        }
    }
}
