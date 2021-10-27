use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Quickly identify and compare entities while retaining a human readable name.
///
/// comparing existing ID's is very fast so it is highly
/// recommended to avoid creating new instances when not necessary. (Also reduces typing mistakes)
#[derive(Clone, Debug, Eq)]
pub struct NamedID {
    pub name: String,
    pub(crate) id: u64,
}

impl NamedID {
    pub fn new(name: String) -> NamedID {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let id = hasher.finish();
        NamedID { name, id }
    }
}

impl PartialEq<Self> for NamedID {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Hash for NamedID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
