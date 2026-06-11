use hashlink::LruCache;
use std::path::PathBuf;

pub struct ContentCache {
    cache: LruCache<PathBuf, Vec<u8>>,
    budget_bytes: usize,
    current_bytes: usize,
}

impl ContentCache {
    #[must_use]
    pub fn new(budget_bytes: usize) -> Self {
        Self {
            cache: LruCache::new_unbounded(),
            budget_bytes,
            current_bytes: 0,
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

        if let Some(old) = self.cache.insert(path, content) {
            self.current_bytes = self.current_bytes.saturating_sub(old.len());
            self.current_bytes += content_size;
        } else {
            self.current_bytes += content_size;
        }

        self.evict_if_over_budget();
    }

    pub fn get(&mut self, path: &PathBuf) -> Option<&Vec<u8>> {
        self.cache.get(path)
    }

    pub fn remove(&mut self, path: &PathBuf) -> Option<Vec<u8>> {
        if let Some(removed) = self.cache.remove(path) {
            self.current_bytes = self.current_bytes.saturating_sub(removed.len());
            Some(removed)
        } else {
            None
        }
    }

    pub fn drop_all(&mut self) {
        self.cache.clear();
        self.current_bytes = 0;
    }

    fn evict_if_over_budget(&mut self) {
        if self.current_bytes <= self.budget_bytes {
            return;
        }

        let target = self.budget_bytes * 80 / 100;
        while self.current_bytes > target {
            if let Some((_, evicted)) = self.cache.remove_lru() {
                self.current_bytes = self.current_bytes.saturating_sub(evicted.len());
            } else {
                break;
            }
        }
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
    fn lru_promotion_keeps_recent_entries() {
        let mut cache = ContentCache::new(100);

        for i in 0..10 {
            let path = PathBuf::from(format!("file{i}.txt"));
            cache.insert(path, vec![b'x'; 10]);
        }

        let old_path = PathBuf::from("file0.txt");
        cache.get(&old_path);

        for i in 10..15 {
            let path = PathBuf::from(format!("file{i}.txt"));
            cache.insert(path, vec![b'x'; 10]);
        }

        assert!(cache.get(&old_path).is_some());
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

    #[test]
    fn update_triggers_eviction() {
        let mut cache = ContentCache::new(100);

        for i in 0..10 {
            let path = PathBuf::from(format!("file{i}.txt"));
            cache.insert(path, vec![b'x'; 10]);
        }
        assert_eq!(cache.current_bytes(), 100);

        let path = PathBuf::from("file0.txt");
        cache.insert(path, vec![b'y'; 50]);

        assert!(cache.current_bytes() <= 100);
    }
}
