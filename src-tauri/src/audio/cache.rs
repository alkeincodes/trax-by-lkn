/// Audio cache management for pre-decoded stem data
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AudioCache {
    cache: Arc<Mutex<HashMap<String, Arc<Vec<f32>>>>>,
    max_size_bytes: usize,
    current_size_bytes: usize,
}

impl AudioCache {
    pub fn new(max_size_gb: f32) -> Self {
        let max_size_bytes = (max_size_gb * 1024.0 * 1024.0 * 1024.0) as usize;
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_size_bytes,
            current_size_bytes: 0,
        }
    }

    pub fn get(&self, key: &str) -> Option<Arc<Vec<f32>>> {
        let cache = self.cache.lock().unwrap();
        cache.get(key).cloned()
    }

    pub fn insert(&mut self, key: String, data: Arc<Vec<f32>>) {
        let size = data.len() * std::mem::size_of::<f32>();

        // Simple cache eviction if we exceed max size
        if self.current_size_bytes + size > self.max_size_bytes {
            self.clear();
        }

        let mut cache = self.cache.lock().unwrap();
        cache.insert(key, data);
        self.current_size_bytes += size;
    }

    pub fn clear(&mut self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        self.current_size_bytes = 0;
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        let cache = self.cache.lock().unwrap();
        (cache.len(), self.current_size_bytes, self.max_size_bytes)
    }

    pub fn set_max_size(&mut self, max_size_bytes: usize) {
        self.max_size_bytes = max_size_bytes;
        if self.current_size_bytes > self.max_size_bytes {
            self.clear();
        }
    }
}