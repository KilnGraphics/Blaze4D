use std::any::Any;
use crate::vk::objects::object_set::ObjectSetProvider;
use crate::vk::objects::pipeline::PipelineInstanceData;
use crate::vk::objects::types::{GenericId, ObjectInstanceData, ObjectSetId};

pub struct PipelineObjectSet {
    set_id: ObjectSetId,
}

impl PipelineObjectSet {
}

impl ObjectSetProvider for PipelineObjectSet {
    fn get_id(&self) -> ObjectSetId {
        self.set_id
    }

    fn get_object_data(&self, id: GenericId) -> ObjectInstanceData {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}