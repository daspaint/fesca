use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Create unique ID according to ID and the share itself
pub fn hash_value(a: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    a.hash(&mut hasher);
    hasher.finish()
}
