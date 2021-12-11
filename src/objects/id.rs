use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::util::id::{GlobalId, LocalId, UUID};

pub struct ObjectType;

// TODO Note this should be updated to a enum once adt_const_params is stabilized.
impl ObjectType {
    pub const fn as_str(ty: u8) -> &'static str {
        match ty {
            Self::BUFFER => "Buffer",
            Self::BUFFER_VIEW => "BufferView",
            Self::IMAGE => "Image",
            Self::IMAGE_VIEW => "ImageView",
            Self::BINARY_SEMAPHORE => "BinarySemaphore",
            Self::TIMELINE_SEMAPHORE => "TimelineSemaphore",
            Self::EVENT => "Event",
            _ => "Invalid",
        }
    }

    pub const GENERIC: u8 = u8::MAX;

    pub const BUFFER: u8 = 1u8;
    pub const BUFFER_VIEW: u8 = 2u8;
    pub const IMAGE: u8 = 3u8;
    pub const IMAGE_VIEW: u8 = 4u8;
    pub const BINARY_SEMAPHORE: u8 = 5u8;
    pub const TIMELINE_SEMAPHORE: u8 = 6u8;
    pub const EVENT: u8 = 7u8;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId<const TYPE: u8>(UUID);

impl<const TYPE: u8> ObjectId<TYPE> {
    const INDEX_BITS: u32 = 56u32;
    const INDEX_OFFSET: u32 = 0u32;
    pub const INDEX_MAX: u64 = (1u64 << Self::INDEX_BITS) - 1u64;
    const INDEX_MASK: u64 = Self::INDEX_MAX << Self::INDEX_OFFSET;

    const TYPE_BITS: u32 = 8u32;
    const TYPE_OFFSET: u32 = Self::INDEX_OFFSET + Self::INDEX_BITS;
    const TYPE_MASK: u64 = (u8::MAX as u64) << Self::TYPE_OFFSET;

    fn make(global_id: GlobalId, index: u64, object_type: u8) -> Self {
        if index > Self::INDEX_MAX {
            panic!("Local id out of range");
        }

        let local = (index << Self::INDEX_OFFSET) | ((object_type as u64) << Self::TYPE_OFFSET);

        Self(UUID{
            global: global_id,
            local: LocalId::from_raw(local),
        })
    }

    pub const fn get_global_id(&self) -> GlobalId {
        self.0.global
    }

    pub const fn get_local_id(&self) -> LocalId {
        self.0.local
    }

    pub const fn get_index(&self) -> u64 {
        (self.0.local.get_raw() & Self::INDEX_MASK) >> Self::INDEX_OFFSET
    }

    pub const fn get_type(&self) -> u8 {
        ((self.0.local.get_raw() & Self::TYPE_MASK) >> Self::TYPE_OFFSET) as u8
    }

    pub const fn as_generic(&self) -> ObjectId<{ ObjectType::GENERIC }> {
        ObjectId::<{ ObjectType::GENERIC }>(self.0)
    }
}

impl<const TYPE: u8> Into<UUID> for ObjectId<TYPE> {
    fn into(self) -> UUID {
        self.0
    }
}

impl ObjectId<{ ObjectType::GENERIC }> {
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
        f.debug_struct("ObjectId")
            .field("type", &self.get_type())
            .field("local_id", &self.get_local_id())
            .field("global_id", &self.get_global_id())
            .finish()
    }
}

impl<const TYPE: u8> Hash for ObjectId<TYPE> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl ObjectId<{ ObjectType::BUFFER }> {
    pub fn new(global_id: GlobalId, index: u64) -> Self {
        Self::make(global_id, index, ObjectType::BUFFER)
    }
}

impl ObjectId<{ ObjectType::BUFFER_VIEW }> {
    pub fn new(global_id: GlobalId, index: u64) -> Self {
        Self::make(global_id, index, ObjectType::BUFFER_VIEW)
    }
}

impl ObjectId<{ ObjectType::IMAGE }> {
    pub fn new(global_id: GlobalId, index: u64) -> Self {
        Self::make(global_id, index, ObjectType::IMAGE)
    }
}

impl ObjectId<{ ObjectType::IMAGE_VIEW }> {
    pub fn new(global_id: GlobalId, index: u64) -> Self {
        Self::make(global_id, index, ObjectType::IMAGE_VIEW)
    }
}

impl ObjectId<{ ObjectType::BINARY_SEMAPHORE }> {
    pub fn new(global_id: GlobalId, index: u64) -> Self {
        Self::make(global_id, index, ObjectType::BINARY_SEMAPHORE)
    }
}

impl ObjectId<{ ObjectType::TIMELINE_SEMAPHORE }> {
    pub fn new(global_id: GlobalId, index: u64) -> Self {
        Self::make(global_id, index, ObjectType::TIMELINE_SEMAPHORE)
    }
}

impl ObjectId<{ ObjectType::EVENT }> {
    pub fn new(global_id: GlobalId, index: u64) -> Self {
        Self::make(global_id, index, ObjectType::EVENT)
    }
}

pub type GenericId = ObjectId<{ ObjectType::GENERIC }>;
pub type BufferId = ObjectId<{ ObjectType::BUFFER }>;
pub type BufferViewId = ObjectId<{ ObjectType::BUFFER_VIEW }>;
pub type ImageId = ObjectId<{ ObjectType::IMAGE }>;
pub type ImageViewId = ObjectId<{ ObjectType::IMAGE_VIEW }>;
pub type BinarySemaphoreId = ObjectId<{ ObjectType::BINARY_SEMAPHORE }>;
pub type TimelineSemaphoreId = ObjectId<{ ObjectType::TIMELINE_SEMAPHORE }>;
pub type EventId = ObjectId<{ ObjectType::EVENT }>;