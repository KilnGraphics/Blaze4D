use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

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
pub struct ObjectId<const TYPE: u8> {
    local: u64,
    global: NonZeroU64,
}

impl<const TYPE: u8> ObjectId<TYPE> {
    const LOCAL_ID_BITS: u32 = 56u32;
    const LOCAL_ID_OFFSET: u32 = 0u32;
    pub const LOCAL_ID_MAX: u64 = (1u64 << Self::LOCAL_ID_BITS) - 1u64;
    const LOCAL_ID_MASK: u64 = Self::LOCAL_ID_MAX << Self::LOCAL_ID_OFFSET;

    const TYPE_BITS: u32 = 8u32;
    const TYPE_OFFSET: u32 = Self::LOCAL_ID_OFFSET + Self::LOCAL_ID_BITS;
    const TYPE_MASK: u64 = (u8::MAX as u64) << Self::TYPE_OFFSET;

    const fn make(local_id: u64, global_id: u64, object_type: u8) -> Self {
        if local_id > Self::LOCAL_ID_MAX {
            panic!("Local id out of range");
        }

        let local = (local_id << Self::LOCAL_ID_OFFSET) | ((object_type as u64) << Self::TYPE_OFFSET);
        let global = global_id;

        if global == 0 {
            panic!("Global id must be non zero");
        }
        unsafe { // Need to wait for const unwrap
            ObjectId { local, global: NonZeroU64::new_unchecked(global) }
        }
    }

    pub const fn get_local_id(&self) -> u64 {
        (self.local & Self::LOCAL_ID_MASK) >> Self::LOCAL_ID_OFFSET
    }

    pub const fn get_type(&self) -> u8 {
        ((self.local & Self::TYPE_MASK) >> Self::TYPE_OFFSET) as u8
    }

    pub const fn get_global_id(&self) -> u64 {
        self.global.get()
    }

    pub const fn as_generic(&self) -> ObjectId<{ ObjectType::GENERIC }> {
        ObjectId::<{ ObjectType::GENERIC }>{ local: self.local, global: self.global }
    }
}

impl ObjectId<{ ObjectType::GENERIC }> {
    pub const fn downcast<const TRG: u8>(self) -> Option<ObjectId<TRG>> {
        if self.get_type() == TRG {
            Some(ObjectId::<TRG>{ local: self.local, global: self.global })
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
        self.local.hash(state);
        self.global.hash(state);
    }
}

impl ObjectId<{ ObjectType::BUFFER }> {
    pub const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::BUFFER)
    }
}

impl ObjectId<{ ObjectType::BUFFER_VIEW }> {
    pub const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::BUFFER_VIEW)
    }
}

impl ObjectId<{ ObjectType::IMAGE }> {
    pub const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::IMAGE)
    }
}

impl ObjectId<{ ObjectType::IMAGE_VIEW }> {
    pub const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::IMAGE_VIEW)
    }
}

impl ObjectId<{ ObjectType::BINARY_SEMAPHORE }> {
    pub const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::BINARY_SEMAPHORE)
    }
}

impl ObjectId<{ ObjectType::TIMELINE_SEMAPHORE }> {
    pub const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::TIMELINE_SEMAPHORE)
    }
}

impl ObjectId<{ ObjectType::EVENT }> {
    pub const fn new(local_id: u64, global_id: u64) -> Self {
        Self::make(local_id, global_id, ObjectType::EVENT)
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

static NEXT_GLOBAL_ID: AtomicU64 = AtomicU64::new(1);

pub fn make_global_id() -> u64 {
    NEXT_GLOBAL_ID.fetch_add(1, Ordering::Relaxed)
}

pub struct IdPool {
    global_id: u64,
    next_local_id: AtomicU64,
}

impl IdPool {
    fn make_local_id(&self) -> u64 {
        self.next_local_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn new() -> Self {
        Self{ global_id: make_global_id(), next_local_id: AtomicU64::new(0) }
    }

    pub fn make_buffer(&self) -> BufferId {
        BufferId::new(self.make_local_id(), self.global_id)
    }

    pub fn make_buffer_view(&self) -> BufferViewId {
        BufferViewId::new(self.make_local_id(), self.global_id)
    }

    pub fn make_image(&self) -> ImageId {
        ImageId::new(self.make_local_id(), self.global_id)
    }

    pub fn make_image_view(&self) -> ImageViewId {
        ImageViewId::new(self.make_local_id(), self.global_id)
    }

    pub fn make_binary_semaphore(&self) -> BinarySemaphoreId {
        BinarySemaphoreId::new(self.make_local_id(), self.global_id)
    }

    pub fn make_timeline_semaphore(&self) -> TimelineSemaphoreId {
        TimelineSemaphoreId::new(self.make_local_id(), self.global_id)
    }

    pub fn make_event(&self) -> EventId {
        EventId::new(self.make_local_id(), self.global_id)
    }
}