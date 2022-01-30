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
    /// Returns the id of this object set.
    fn get_id(&self) -> ObjectSetId;

    /// Returns the handle of a buffer that is part of this object set.
    ///
    /// #Panics
    /// If the buffer id does not belong to this object set or does not map to a buffer object.
    fn get_buffer_handle(&self, _: id::BufferId) -> vk::Buffer {
        panic!("ObjectSet does not support buffers");
    }

    /// Returns the [`BufferInfo`] struct for a buffer that is part of this object set.
    ///
    /// #Panics
    /// If the buffer id does not belong to this object set or does not map to a buffer object.
    fn get_buffer_info(&self, _: id::BufferId) -> &Arc<BufferInfo> {
        panic!("ObjectSet does not support buffers");
    }

    /// Returns the handle of a buffer view that is part of this object set.
    ///
    /// #Panics
    /// If the buffer view id does not belong to this object set or does not map to a buffer view
    /// object.
    fn get_buffer_view_handle(&self, _: id::BufferViewId) -> vk::BufferView {
        panic!("ObjectSet does not support buffer views");
    }

    /// Returns the [`BufferViewInfo`] struct for a buffer view that is part of this object set.
    ///
    /// #Panics
    /// If the buffer view id does not belong to this object set or does not map to a buffer view
    /// object.
    fn get_buffer_view_info(&self, _: id::BufferViewId) -> &BufferViewInfo {
        panic!("ObjectSet does not support buffer views");
    }

    /// Returns the handle of a image that is part of this object set.
    ///
    /// #Panics
    /// If the image id does not belong to this object set or does not map to a image object.
    fn get_image_handle(&self, _: id::ImageId) -> vk::Image {
        panic!("ObjectSet does not support images");
    }

    /// Returns the [`ImageInfo`] struct for a image that is part of this object set.
    ///
    /// #Panics
    /// If the image id does not belong to this object set or does not map to a image object.
    fn get_image_info(&self, _: id::ImageId) -> &Arc<ImageInfo> {
        panic!("ObjectSet does not support images");
    }

    /// Returns the handle of a image view that is part of this object set.
    ///
    /// #Panics
    /// If the image view id does not belong to this object set or does not map to a image view
    /// object.
    fn get_image_view_handle(&self, _: id::ImageViewId) -> vk::ImageView {
        panic!("ObjectSet does not support image views");
    }

    /// Returns the [`ImageViewInfo`] struct for a image view that is part of this object set.
    ///
    /// #Panics
    /// If the image view id does not belong to this object set or does not map to a image view
    /// object.
    fn get_image_view_info(&self, _: id::ImageViewId) -> &ImageViewInfo {
        panic!("ObjectSet does not support image views");
    }

    /// Returns the handle of a swapchain that is part of this object set.
    ///
    /// #Panics
    /// If the swapchain id does not belong to this object set or does not map to a swapchain
    /// object.
    fn get_swapchain_handle(&self, _: id::SwapchainId) -> vk::SwapchainKHR {
        panic!("ObjectSet does not support swapchains");
    }

    fn as_any(&self) -> &dyn Any;
}

/// A wrapper type around the [`ObjectSetProvider`] trait.
///
/// Provides a universal object set api.
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