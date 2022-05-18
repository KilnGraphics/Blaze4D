use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::ops::Deref;

use ash::vk;
use ash::vk::Handle;

use crate::UUID;

pub trait ObjectId: Copy + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + Debug {
    type HandleType: Handle + Copy;

    fn from_raw(id: UUID) -> Self;

    fn as_uuid(&self) -> UUID;
}

macro_rules! declare_object_id {
    ($name:ident, $handle_type:ty) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(UUID);

        impl $name {
            pub fn new() -> Self {
                Self(UUID::new())
            }
        }

        impl ObjectId for $name {
            type HandleType = $handle_type;

            fn from_raw(id: UUID) -> Self {
                Self(id)
            }

            fn as_uuid(&self) -> UUID {
                self.0
            }
        }

        impl Deref for $name {
            type Target = UUID;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<$name> for UUID {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_fmt(format_args!(concat!(stringify!($name), "({:#016X})"), self.0.get_raw()))
            }
        }
    }
}

declare_object_id!(BufferId, vk::Buffer);
declare_object_id!(BufferViewId, vk::BufferView);
declare_object_id!(ImageId, vk::Image);
declare_object_id!(ImageViewId, vk::ImageView);
declare_object_id!(SurfaceId, vk::SurfaceKHR);
declare_object_id!(SwapchainId, vk::SwapchainKHR);