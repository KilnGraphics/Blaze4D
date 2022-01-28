use std::any::Any;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;
use ash::vk;

use crate::objects::buffer::{BufferInfo, BufferViewInfo};
use crate::objects::id;
use crate::objects::id::ObjectSetId;
use crate::objects::image::{ImageInfo, ImageViewInfo};

/// A trait that must be implemented by any object set implementation.
pub trait ObjectSetProvider {
    fn get_id(&self) -> ObjectSetId;

    fn get_buffer_handle(&self, _: id::BufferId) -> vk::Buffer {
        panic!("ObjectSet does not support buffers");
    }

    fn get_buffer_info(&self, _: id::BufferId) -> &Arc<BufferInfo> {
        panic!("ObjectSet does not support buffers");
    }

    fn get_buffer_view_handle(&self, _: id::BufferViewId) -> vk::BufferView {
        panic!("ObjectSet does not support buffer views");
    }

    fn get_buffer_view_info(&self, _: id::BufferViewId) -> &BufferViewInfo {
        panic!("ObjectSet does not support buffer views");
    }

    fn get_image_handle(&self, _: id::ImageId) -> vk::Image {
        panic!("ObjectSet does not support images");
    }

    fn get_image_info(&self, _: id::ImageId) -> &Arc<ImageInfo> {
        panic!("ObjectSet does not support images");
    }

    fn get_image_view_handle(&self, _: id::ImageViewId) -> vk::ImageView {
        panic!("ObjectSet does not support image views");
    }

    fn get_image_view_info(&self, _: id::ImageViewId) -> &ImageViewInfo {
        panic!("ObjectSet does not support image views");
    }

    fn as_any(&self) -> &dyn Any;
}

/// A wrapper type around the [`ObjectSetProvider`] trait.
///
/// Provides a uniform object set api.
#[derive(Clone)]
pub struct ObjectSet(Arc<dyn ObjectSetProvider>);

impl ObjectSet {
    /// Creates a new object set from the specified provider.
    pub fn new<T: ObjectSetProvider + 'static>(set: T) -> Self {
        Self(Arc::new(set))
    }
}

impl Deref for ObjectSet {
    type Target = dyn ObjectSetProvider;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl PartialEq for ObjectSet {
    fn eq(&self, other: &Self) -> bool {
        self.0.get_id().eq(&other.0.get_id())
    }
}

impl Eq for ObjectSet {
}

impl PartialOrd for ObjectSet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.get_id().partial_cmp(&other.0.get_id())
    }
}

impl Ord for ObjectSet {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.get_id().cmp(&other.0.get_id())
    }
}

impl Hash for ObjectSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.get_id().hash(state)
    }
}