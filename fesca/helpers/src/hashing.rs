use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Create unique ID according to ID and the share itself
fn hash_two_values<T: Hash, U: Hash>(a: T, b: U) -> u64 {
    let mut hasher = DefaultHasher::new();
    a.hash(&mut hasher);
    b.hash(&mut hasher);
    hasher.finish()
}
