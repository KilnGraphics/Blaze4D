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

impl ObjectRequestDescription {
    pub fn make_buffer(description: BufferCreateDesc, memory_location: gpu_allocator::MemoryLocation) -> Self {
        ObjectRequestDescription::Buffer(BufferRequestDescription{
            description,
            memory_location
        })
    }

    pub fn make_buffer_view(description: BufferViewCreateDesc, owning_set: Option<ObjectSet>, buffer_id: id::BufferId) -> Self {
        ObjectRequestDescription::BufferView(BufferViewRequestDescription{
            description,
            owning_set,
            buffer_id
        })
    }

    pub fn make_image(description: ImageCreateDesc, memory_location: gpu_allocator::MemoryLocation) -> Self {
        ObjectRequestDescription::Image(ImageRequestDescription{
            description,
            memory_location
        })
    }

    pub fn make_image_view(description: ImageViewCreateDesc, owning_set: Option<ObjectSet>, image_id: id::ImageId) -> Self {
        ObjectRequestDescription::ImageView(ImageViewRequestDescription{
            description,
            owning_set,
            image_id
        })
    }
}