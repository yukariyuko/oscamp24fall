#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use alloc::collections;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Eq;
use core::hash::Hash;

pub struct HashMap<K, V> {
    tab: Vec<Vec<(K, V)>>,
    size: usize,
    stamp: usize,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq + Clone + core::convert::AsRef<[u8]>,
    V: Clone,
{
    pub fn new() -> Self {
        HashMap {
            tab: vec![Vec::new(); 16], // Initialize tab vector with a non-zero length
            size: 0,
            stamp: 1145141919810,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let hash = self.hash(&key);
        if hash >= self.tab.len() {
            self.tab.resize(hash * 2, Vec::new());
        }
        let bucket = &mut self.tab[hash];
        for (existing_key, existing_value) in bucket.iter_mut() {
            if *existing_key == key {
                *existing_value = value;
                return;
            }
        }
        bucket.push((key, value));
        self.size += 1;
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash(key);
        let bucket = &self.tab[hash];
        for (existing_key, existing_value) in bucket.iter() {
            if existing_key == key {
                return Some(existing_value);
            }
        }
        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let hash = self.hash(key);
        let bucket = &mut self.tab[hash];
        if let Some(index) = bucket
            .iter()
            .position(|(existing_key, _)| existing_key == key)
        {
            let (_, value) = bucket.remove(index);
            self.size -= 1;
            Some(value)
        } else {
            None
        }
    }

    fn hash(&self, key: &K) -> usize
    where
        K: AsRef<[u8]>,
    {
        let mut hash = self.stamp;

        for byte in key.as_ref() {
            hash = (hash << 5).wrapping_add(hash) ^ *byte as usize;
        }

        hash % self.tab.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.tab.iter().flatten()
    }
}
