use std::collections::HashMap;
use std::time::SystemTime;

pub struct CacheEntry<T> {
    value: T,
    last_accessed: SystemTime,
    // expired_by_time: C
}

pub struct LRUCache<T> {
    data: HashMap<String, CacheEntry<T>>,
    capacity: usize,
}

impl<T> LRUCache<T> {
    pub fn new(capacity: usize) -> Self {
        LRUCache {
            data: HashMap::new(),
            capacity,
        }
    }

    pub fn remove(&mut self, key: &str) -> bool {
        let status = self.data.remove(key);
        status.is_some()
    }

    pub fn get(&mut self, key: &str) -> Option<&T> {
        if let Some(entry) = self.data.get_mut(key) {
            entry.last_accessed = SystemTime::now();
            Some(&entry.value)
        } else {
            None
        }
    }

    pub fn set(&mut self, key: String, value: T) {
        if self.data.len() >= self.capacity {
            let oldest_key = self
                .data
                .iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(key, _)| key.clone())
                .unwrap();
            self.data.remove(&oldest_key);
        }

        let entry = CacheEntry {
            value,
            last_accessed: SystemTime::now(),
        };
        self.data.insert(key, entry);
    }

}