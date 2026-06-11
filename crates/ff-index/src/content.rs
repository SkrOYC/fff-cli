use hashlink::LruCache;
use std::path::PathBuf;

pub struct ContentCache {
    cache: LruCache<PathBuf, Vec<u8>>,
    budget_bytes: usize,
    current_bytes: usize,
    insert_order: Vec<PathBuf>,
}

impl ContentCache {
    #[must_use]
    pub fn new(budget_bytes: usize) -> Self {
        Self {
            cache: LruCache::new(1_000_000),
            budget_bytes,
            current_bytes: 0,
            insert_order: Vec::new(),
        }
    }

    #[must_use]
    pub fn budget_bytes(&self) -> usize {
        self.budget_bytes
    }

    #[must_use]
    pub fn current_bytes(&self) -> usize {
        self.current_bytes
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn insert(&mut self, path: PathBuf, content: Vec<u8>) {
        let content_size = content.len();

        if content_size > self.budget_bytes {
            return;
        }

        if let Some(old) = self.cache.insert(path.clone(), content) {
            self.current_bytes = self.current_bytes.saturating_sub(old.len());
            self.current_bytes += content_size;
            return;
        }

        self.current_bytes += content_size;
        self.insert_order.push(path);

        if self.current_bytes > self.budget_bytes {
            let target = self.budget_bytes * 80 / 100;
            while self.current_bytes > target && !self.insert_order.is_empty() {
                let oldest = self.insert_order.remove(0);
                if let Some(removed) = self.cache.remove(&oldest) {
                    self.current_bytes = self.current_bytes.saturating_sub(removed.len());
                }
            }
        }
    }

    pub fn get(&mut self, path: &PathBuf) -> Option<&Vec<u8>> {
        self.cache.get(path)
    }

    pub fn remove(&mut self, path: &PathBuf) -> Option<Vec<u8>> {
        if let Some(removed) = self.cache.remove(path) {
            self.current_bytes = self.current_bytes.saturating_sub(removed.len());
            self.insert_order.retain(|p| p != path);
            Some(removed)
        } else {
            None
        }
    }

    pub fn drop_all(&mut self) {
        self.cache.clear();
        self.current_bytes = 0;
        self.insert_order.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cache_is_empty() {
        let cache = ContentCache::new(1024);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.current_bytes(), 0);
        assert_eq!(cache.budget_bytes(), 1024);
    }

    #[test]
    fn insert_and_get() {
        let mut cache = ContentCache::new(1024);
        let path = PathBuf::from("test.txt");
        let content = b"hello world".to_vec();

        cache.insert(path.clone(), content.clone());

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.current_bytes(), 11);
        assert_eq!(cache.get(&path), Some(&content));
    }

    #[test]
    fn eviction_when_over_budget() {
        let mut cache = ContentCache::new(100);

        for i in 0..20 {
            let path = PathBuf::from(format!("file{i}.txt"));
            let content = vec![b'x'; 10];
            cache.insert(path, content);
        }

        assert!(cache.current_bytes() <= 100);
    }

    #[test]
    fn eviction_targets_80_percent() {
        let mut cache = ContentCache::new(100);

        for i in 0..15 {
            let path = PathBuf::from(format!("file{i}.txt"));
            let content = vec![b'x'; 10];
            cache.insert(path, content);
        }

        assert!(cache.current_bytes() <= 100);
    }

    #[test]
    fn files_larger_than_budget_are_rejected() {
        let mut cache = ContentCache::new(100);
        let path = PathBuf::from("large.txt");
        let content = vec![b'x'; 200];

        cache.insert(path.clone(), content);

        assert!(cache.is_empty());
        assert_eq!(cache.get(&path), None);
    }

    #[test]
    fn drop_all_clears_cache() {
        let mut cache = ContentCache::new(1024);

        for i in 0..5 {
            let path = PathBuf::from(format!("file{i}.txt"));
            cache.insert(path, vec![b'x'; 10]);
        }

        assert_eq!(cache.len(), 5);
        assert_eq!(cache.current_bytes(), 50);

        cache.drop_all();

        assert!(cache.is_empty());
        assert_eq!(cache.current_bytes(), 0);
    }

    #[test]
    fn remove_decrements_bytes() {
        let mut cache = ContentCache::new(1024);
        let path = PathBuf::from("test.txt");
        cache.insert(path.clone(), vec![b'x'; 50]);

        assert_eq!(cache.current_bytes(), 50);

        let removed = cache.remove(&path);
        assert_eq!(removed.unwrap().len(), 50);
        assert_eq!(cache.current_bytes(), 0);
    }

    #[test]
    fn update_existing_entry() {
        let mut cache = ContentCache::new(1024);
        let path = PathBuf::from("test.txt");

        cache.insert(path.clone(), vec![b'x'; 10]);
        assert_eq!(cache.current_bytes(), 10);

        cache.insert(path.clone(), vec![b'y'; 20]);
        assert_eq!(cache.current_bytes(), 20);
        assert_eq!(cache.len(), 1);
    }
}
