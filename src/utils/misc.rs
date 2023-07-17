use std::hash::Hash;
use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

pub fn calc_hash<T: Hash>(t: T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}