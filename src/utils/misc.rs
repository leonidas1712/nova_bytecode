use std::hash::Hash;
use std::hash::Hasher;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;

pub fn calc_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(Debug)]
pub struct StringIntern {
    strings:HashMap<u64,String> // string interning
}

impl StringIntern {
    pub fn new()->StringIntern {
        let strings=HashMap::new();
        StringIntern { strings }
    }

    pub fn add_string(&mut self, string:String)->u64 {
        let hash=calc_hash(&string);
        if !self.strings.contains_key(&hash) {
            self.strings.insert(hash, string);
        } 
        hash
    }

    /// Get string given its hash if it is interned
    pub fn get_string(&self, hash:u64)->Option<&String> {
        self.strings.get(&hash)
    }

    pub fn has_string(&self, hash:u64)->bool {
        self.strings.contains_key(&hash)
    }
}