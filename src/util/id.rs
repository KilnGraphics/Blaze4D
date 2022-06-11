/// Utilities for globally unique identifiers.
use std::cell::RefCell;

use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use crate::util::rand::Xoshiro256PlusPlus;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UUID(NonZeroU64);

lazy_static! {
    static ref UUID_SEEDER : Mutex<Xoshiro256PlusPlus> = Mutex::new(Xoshiro256PlusPlus::from_seed([1u64, 1u64, 1u64, 1u64]));
}

thread_local! {
    static THREAD_UUID_SEEDER : RefCell<Xoshiro256PlusPlus> = {
        let mut seeder = UUID_SEEDER.lock().unwrap();
        seeder.jump();

        RefCell::new(*seeder)
    }
}

impl UUID {
    pub fn new() -> Self {
        let id = THREAD_UUID_SEEDER.with(|seeder| seeder.borrow_mut().find(|id| *id != 0u64)).unwrap();

        Self(NonZeroU64::new(id).unwrap())
    }

    pub const fn from_raw(id: u64) -> Self {
        if id == 0u64 {
            panic!("Zero id")
        }
        Self(unsafe { NonZeroU64::new_unchecked(id) })
    }

    pub const fn get_raw(&self) -> u64 {
        self.0.get()
    }
}

impl Debug for UUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("UUID({:#016X})", self.get_raw()))
    }
}

#[derive(Clone, Debug)]
enum NameType {
    Static(&'static str),
    String(Arc<String>),
}

impl NameType {
    const fn new_static(str: &'static str) -> Self {
        Self::Static(str)
    }

    fn new_string(str: String) -> Self {
        Self::String(Arc::new(str))
    }

    fn get(&self) -> &str {
        match self {
            NameType::Static(str) => *str,
            NameType::String(str) => str.as_ref(),
        }
    }
}

/// A UUID generated from a string.
///
/// NamedUUIDs use a predefined global id with the local id being calculated as the hash of a
/// string. The name is stored along side the UUID for easy debugging or printing. The name is
/// stored by Arc enabling fast Copying of the struct.
#[derive(Clone)]
pub struct NamedUUID {
    name: NameType,
    id: UUID,
}

impl NamedUUID {
    const fn hash_str_const(name: &str) -> u64 {
        xxhash_rust::const_xxh3::xxh3_64(name.as_bytes())
    }

    fn hash_str(name: &str) -> u64 {
        xxhash_rust::xxh3::xxh3_64(name.as_bytes())
    }

    /// Creates a new uuid based on the hash of the string. Calling this function with the same
    /// string will always return the same id.
    pub const fn from_str(name: &'static str) -> NamedUUID {
        let hash = Self::hash_str_const(name);

        NamedUUID { name: NameType::new_static(name), id: UUID::from_raw(hash) }
    }

    /// Creates a new uuid based on the hash of the string. Calling this function with the same
    /// string will always return the same id.
    pub fn from_string(name: String) -> NamedUUID {
        let hash = Self::hash_str(name.as_str());

        NamedUUID { name: NameType::new_string(name), id: UUID::from_raw(hash) }
    }

    /// Creates a new random uuid with a string attached. Calling this function with the same
    /// string will not return the same id.
    pub fn with_str(name: &'static str) -> NamedUUID {
        NamedUUID { name: NameType::new_static(name), id: UUID::new() }
    }

    /// Creates a new random uuid with a string attached. Calling this function with the same
    /// string will not return the same id.
    pub fn with_string(name: String) -> NamedUUID {
        NamedUUID { name: NameType::new_string(name), id: UUID::new() }
    }

    /// Generates the uuid for a string. Does not store the name to allow for parsing non static
    /// strings
    pub const fn uuid_for(name: &str) -> UUID {
        UUID::from_raw(Self::hash_str_const(name))
    }

    /// Returns the attached string
    pub fn get_name(&self) -> &str {
        self.name.get()
    }

    /// Returns the uuid
    pub fn get_uuid(&self) -> UUID {
        self.id
    }

    /// Utility function to clone a uuid that has a const string attached to it.
    /// This function will panic if the uuid has a non const string attached.
    pub const fn clone_const(&self) -> Self {
        match self.name {
            NameType::String(_) => {
                panic!("Cloned non const name")
            }
            NameType::Static(str) => {
                Self{ name: NameType::Static(str), id: self.id }
            }
        }
    }
}

impl PartialEq for NamedUUID {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for NamedUUID {
}

impl PartialEq<UUID> for NamedUUID {
    fn eq(&self, other: &UUID) -> bool {
        self.get_uuid().eq(other)
    }
}

impl PartialOrd for NamedUUID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for NamedUUID {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd<UUID> for NamedUUID {
    fn partial_cmp(&self, other: &UUID) -> Option<Ordering> {
        self.get_uuid().partial_cmp(other)
    }
}

impl Hash for NamedUUID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // The hash should be identical to the one generated from the uuid
        self.get_uuid().hash(state)
    }
}

impl Into<UUID> for NamedUUID {
    fn into(self) -> UUID {
        self.get_uuid()
    }
}

impl Debug for NamedUUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match &self.name {
            NameType::Static(str) => *str,
            NameType::String(str) => str.as_str()
        };
        f.write_fmt(format_args!("NamedUUID{{\"{}\", {:?}}}", name, &self.id))
    }
}

/// Utility macro to define new id types using a [`UUID`] internally.
#[macro_export]
macro_rules! define_uuid_type {
    ($vis:vis, $name:ident) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        $vis struct $name(UUID);

        impl $name {
            $vis fn new() -> Self {
                Self(UUID::new())
            }

            $vis fn from_uuid(raw: UUID) -> Self {
                Self(raw)
            }

            $vis fn as_uuid(&self) -> UUID {
                self.0
            }
        }

        impl From<$name> for UUID {
            fn from(id: $name) -> Self {
                id.as_uuid
            }
        }
    }
}

pub use define_uuid_type;