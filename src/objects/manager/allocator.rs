use crate::objects::buffer::{BufferCreateDesc, BufferViewCreateDesc};
use crate::objects::image::{ImageCreateDesc, ImageViewCreateDesc};
use crate::objects::{id, ObjectSet};

pub(super) struct BufferRequestDescription {
    pub description: BufferCreateDesc,
    pub memory_location: gpu_allocator::MemoryLocation,
}

pub(super) struct BufferViewRequestDescription {
    pub description: BufferViewCreateDesc,
    /// The set that owns the source buffer of the view. If None the source buffer must be part of
    /// the same set of requests as this request.
    pub owning_set: Option<ObjectSet>,
    pub buffer_id: id::BufferId,
}

pub(super) struct ImageRequestDescription {
    pub description: ImageCreateDesc,
    pub memory_location: gpu_allocator::MemoryLocation,
}

pub(super) struct ImageViewRequestDescription {
    pub description: ImageViewCreateDesc,
    /// The set that owns the source image of the view. If None the source image must be part of
    /// the same set of requests as this request.
    pub owning_set: Option<ObjectSet>,
    pub image_id: id::ImageId,
}

/// Describes a single object request
pub(super) enum ObjectRequestDescription {
    Buffer(BufferRequestDescription),
    BufferView(BufferViewRequestDescription),
    Image(ImageRequestDescription),
    ImageView(ImageViewRequestDescription),
}