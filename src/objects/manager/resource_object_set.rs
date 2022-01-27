
use ash::vk;

use crate::objects::ObjectSet2;



pub(super) enum ResourceObjectType {
    Buffer,
    BufferView,
    Image,
    ImageView,
}

pub(super) struct ResourceObjectData {
    object_type: ResourceObjectType,
    handle: u64,
    source_set: Option<ObjectSet2>
}