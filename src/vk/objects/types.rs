use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

/// An identifier for object sets
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectSetId(NonZeroU64);

static NEXT_OBJECT_SET_ID : AtomicU64 = AtomicU64::new(1);

impl ObjectSetId {
    const OBJECT_SET_ID_MAX : u64 = (1u64 << 40u32) - 1u64;

    /// Creates a new unique object set id
    pub fn new() -> Self {
        let next = NEXT_OBJECT_SET_ID.fetch_add(1, Ordering::Relaxed);
        if next > Self::OBJECT_SET_ID_MAX {
            panic!("ObjectSetId overflow");
        }

        Self(NonZeroU64::new(next).unwrap())
    }

    fn from_raw(raw: u64) -> Self {
        if raw > Self::OBJECT_SET_ID_MAX {
            panic!("Value passed to ObjectSetId::from_raw is out of bounds");
        }

        Self(NonZeroU64::new(raw).unwrap())
    }

    /// Returns the raw 64bit value of the id
    pub fn get_raw(&self) -> u64 {
        self.0.get()
    }
}

impl Debug for ObjectSetId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ObjectSetId({:#010X})", self.0.get()))
    }
}

pub struct ObjectType;

// TODO Note this should be updated to a enum once adt_const_params is stabilized.
impl ObjectType {
    pub const fn as_str(ty: u8) -> &'static str {
        match ty {
            Self::BUFFER => "Buffer",
            Self::BUFFER_VIEW => "BufferView",
            Self::IMAGE => "Image",
            Self::IMAGE_VIEW => "ImageView",
            Self::SEMAPHORE => "Semaphore",
            Self::EVENT => "Event",
            Self::FENCE => "Fence",
            Self::SURFACE => "Surface",
            Self::SWAPCHAIN => "Swapchain",
            _ => "Invalid",
        }
    }

    pub const GENERIC: u8 = u8::MAX;

    pub const BUFFER: u8 = 1u8;
    pub const BUFFER_VIEW: u8 = 2u8;
    pub const IMAGE: u8 = 3u8;
    pub const IMAGE_VIEW: u8 = 4u8;
    pub const SEMAPHORE: u8 = 5u8;
    pub const EVENT: u8 = 6u8;
    pub const FENCE: u8 = 7u8;
    pub const SURFACE: u8 = 8u8;
    pub const SWAPCHAIN: u8 = 9u8;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId<const TYPE: u8>(NonZeroU64);

impl<const TYPE: u8> ObjectId<TYPE> {
    const SET_ID_BITS: u32 = 40u32;
    const SET_ID_OFFSET: u32 = 0u32;
    const SET_ID_MAX: u64 = (1u64 << Self::SET_ID_BITS) - 1u64;
    const SET_ID_MASK: u64 = Self::SET_ID_MAX << Self::SET_ID_OFFSET;

    const INDEX_OFFSET: u32 = 48u32;
    const INDEX_MAX: u64 = u16::MAX as u64;
    const INDEX_MASK: u64 = Self::INDEX_MAX << Self::INDEX_OFFSET;

    const TYPE_OFFSET: u32 = 40u32;
    const TYPE_MAX: u64 = u8::MAX as u64;
    const TYPE_MASK: u64 = Self::TYPE_MAX << Self::TYPE_OFFSET;

    fn make(set_id: ObjectSetId, index: u16, object_type: u8) -> Self {
        let id = (set_id.get_raw() << Self::SET_ID_OFFSET) | ((index as u64) << Self::INDEX_OFFSET) | ((object_type as u64) << Self::TYPE_OFFSET);

        Self(NonZeroU64::new(id).unwrap())
    }

    pub fn get_set_id(&self) -> ObjectSetId {
        ObjectSetId::from_raw((self.0.get() & Self::SET_ID_MASK) >> Self::SET_ID_OFFSET)
    }

    pub const fn get_index(&self) -> u16 {
        ((self.0.get() & Self::INDEX_MASK) >> Self::INDEX_OFFSET) as u16
    }

    pub const fn get_type(&self) -> u8 {
        ((self.0.get() & Self::TYPE_MASK) >> Self::TYPE_OFFSET) as u8
    }

    /// Converts the id to a generic id
    pub const fn as_generic(&self) -> ObjectId<{ ObjectType::GENERIC }> {
        ObjectId::<{ ObjectType::GENERIC }>(self.0)
    }
}

impl ObjectId<{ ObjectType::GENERIC }> {
    /// Attempts to cast a generic object id to a specific type. If the generic id is not of the
    /// correct type `None` is returned.
    pub const fn downcast<const TRG: u8>(self) -> Option<ObjectId<TRG>> {
        if self.get_type() == TRG {
            Some(ObjectId::<TRG>(self.0))
        } else {
            None
        }
    }
}

impl<const TYPE: u8> Debug for ObjectId<TYPE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ObjectId(Set: {:#010X}, Index: {:#04X}, Type: {})", self.get_set_id().get_raw(), self.get_index(), self.get_type()))
    }
}

impl<const TYPE: u8> Hash for ObjectId<TYPE> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

pub trait ObjectIdType {
    type InstanceInfo;

    fn as_generic(&self) -> GenericId;
}

pub trait UnwrapToInstanceData<'a, I> {
    fn unwrap(self) -> &'a I;
}

macro_rules! declare_object_types {
    ($($name:ident, $id_value:expr, $instance_type:ty;)+) => {
        $(
            impl ObjectId<{$id_value}> {
                pub fn new(set_id: ObjectSetId, index: u16) -> Self {
                    Self::make(set_id, index, $id_value)
                }
            }

            impl ObjectIdType for ObjectId<{$id_value}> {
                type InstanceInfo = $instance_type;

                fn as_generic(&self) -> GenericId {
                    self.as_generic()
                }
            }

            impl From<ObjectId<{$id_value}>> for GenericId {
                fn from(src: ObjectId<{$id_value}>) -> Self {
                    Self(src.0)
                }
            }
        )+

        pub enum ObjectInstanceData<'a> {
            $(
                $name(&'a $instance_type),
            )+
        }

        $(
            impl<'a> UnwrapToInstanceData<'a, $instance_type> for ObjectInstanceData<'a> {
                fn unwrap(self) -> &'a $instance_type {
                    match self {
                        ObjectInstanceData::$name(data) => data,
                        _ => panic!("Invalid object type"),
                    }
                }
            }
        )+

        $(
            impl<'a> From<&'a $instance_type> for ObjectInstanceData<'a> {
                fn from(src: &'a $instance_type) -> Self {
                    ObjectInstanceData::$name(src)
                }
            }
        )+
    }
}

declare_object_types!(
    Buffer, ObjectType::BUFFER, super::buffer::BufferInstanceData;
    BufferView, ObjectType::BUFFER_VIEW, super::buffer::BufferViewInstanceData;
    Image, ObjectType::IMAGE, super::image::ImageInstanceData;
    ImageView, ObjectType::IMAGE_VIEW, super::image::ImageViewInstanceData;
    Surface, ObjectType::SURFACE, ();
    Swapchain, ObjectType::SWAPCHAIN, super::swapchain::SwapchainInstanceData;
);

pub type GenericId = ObjectId<{ ObjectType::GENERIC }>;
pub type BufferId = ObjectId<{ ObjectType::BUFFER }>;
pub type BufferViewId = ObjectId<{ ObjectType::BUFFER_VIEW }>;
pub type ImageId = ObjectId<{ ObjectType::IMAGE }>;
pub type ImageViewId = ObjectId<{ ObjectType::IMAGE_VIEW }>;
pub type SemaphoreId = ObjectId<{ ObjectType::SEMAPHORE }>;
pub type EventId = ObjectId<{ ObjectType::EVENT }>;
pub type FenceId = ObjectId<{ ObjectType::FENCE }>;
pub type SurfaceId = ObjectId<{ ObjectType::SURFACE }>;
pub type SwapchainId = ObjectId<{ ObjectType::SWAPCHAIN }>;