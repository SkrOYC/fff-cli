use crate::content::ContentCache;
use crate::entry::FileEntry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

pub struct Index {
    entries: Vec<FileEntry>,
    path_to_index: HashMap<PathBuf, usize>,
    root: PathBuf,
    content_cache: Option<ContentCache>,
    rebuilding: AtomicBool,
    last_rebuild: Instant,
}

impl Index {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self {
            entries: Vec::new(),
            path_to_index: HashMap::new(),
            root,
            content_cache: None,
            rebuilding: AtomicBool::new(false),
            last_rebuild: Instant::now(),
        }
    }

    #[must_use]
    pub fn with_content_cache(root: PathBuf, budget_bytes: usize) -> Self {
        Self {
            entries: Vec::new(),
            path_to_index: HashMap::new(),
            root,
            content_cache: Some(ContentCache::new(budget_bytes)),
            rebuilding: AtomicBool::new(false),
            last_rebuild: Instant::now(),
        }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn entries(&self) -> &[FileEntry] {
        &self.entries
    }

    #[must_use]
    pub fn get_by_path(&self, path: &Path) -> Option<&FileEntry> {
        self.path_to_index
            .get(path)
            .and_then(|&idx| self.entries.get(idx))
    }

    #[must_use]
    pub fn get_content(&mut self, path: &Path) -> Option<&Vec<u8>> {
        let path_buf = path.to_path_buf();
        self.content_cache.as_mut()?.get(&path_buf)
    }

    pub fn cache_content(&mut self, path: PathBuf, content: Vec<u8>) {
        if let Some(ref mut cache) = self.content_cache {
            cache.insert(path, content);
        }
    }

    pub fn drop_content_cache(&mut self) {
        if let Some(ref mut cache) = self.content_cache {
            cache.drop_all();
        }
    }

    #[must_use]
    pub fn is_rebuilding(&self) -> bool {
        self.rebuilding.load(Ordering::Acquire)
    }

    pub fn set_rebuilding(&self, rebuilding: bool) {
        self.rebuilding.store(rebuilding, Ordering::Release);
    }

    #[must_use]
    pub fn last_rebuild(&self) -> Instant {
        self.last_rebuild
    }

    pub fn set_entries(&mut self, entries: Vec<FileEntry>) {
        self.path_to_index.clear();
        for (idx, entry) in entries.iter().enumerate() {
            self.path_to_index.insert(entry.path.clone(), idx);
        }
        self.entries = entries;
        self.last_rebuild = Instant::now();
        self.drop_content_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_common::types::{FileType, GitStatus};

    fn make_entry(path: &str, size: u64) -> FileEntry {
        FileEntry {
            path: PathBuf::from(path),
            size,
            mtime: 0,
            atime: 0,
            ctime: 0,
            mode: 0o100644,
            uid: 1000,
            gid: 1000,
            inode: 0,
            device: 0,
            link_count: 1,
            file_type: FileType::RegularFile,
            git_status: GitStatus::NotInGit,
        }
    }

    #[test]
    fn new_index_is_empty() {
        let index = Index::new(PathBuf::from("/test"));
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn set_entries_populates_index() {
        let mut index = Index::new(PathBuf::from("/test"));
        let entries = vec![make_entry("file1.txt", 100), make_entry("file2.txt", 200)];
        index.set_entries(entries);

        assert_eq!(index.len(), 2);
        assert!(!index.is_empty());
    }

    #[test]
    fn get_by_path_returns_correct_entry() {
        let mut index = Index::new(PathBuf::from("/test"));
        let entries = vec![make_entry("file1.txt", 100), make_entry("file2.txt", 200)];
        index.set_entries(entries);

        let entry = index.get_by_path(Path::new("file2.txt")).unwrap();
        assert_eq!(entry.size, 200);
    }

    #[test]
    fn get_by_path_returns_none_for_missing() {
        let index = Index::new(PathBuf::from("/test"));
        assert!(index.get_by_path(Path::new("missing.txt")).is_none());
    }

    #[test]
    fn rebuilding_flag() {
        let index = Index::new(PathBuf::from("/test"));
        assert!(!index.is_rebuilding());

        index.set_rebuilding(true);
        assert!(index.is_rebuilding());

        index.set_rebuilding(false);
        assert!(!index.is_rebuilding());
    }
}
