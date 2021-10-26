use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Quickly identify and compare entities while retaining a human readable name.
///
/// comparing existing ID's is very fast so it is highly
/// recommended to avoid creating new instances when not necessary. (Also reduces typing mistakes)
#[derive(Clone, Debug)]
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

impl Eq for NamedID {}

impl PartialOrd<Self> for NamedID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for NamedID {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Hash for NamedID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}