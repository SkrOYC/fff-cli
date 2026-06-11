use ff_index::{Index, ScanOptions, scan};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn get_test_root() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("FF_TEST_ROOT") {
        return Some(PathBuf::from(path));
    }
    let nix_store = PathBuf::from("/nix/store");
    if nix_store.exists() {
        return Some(nix_store);
    }
    None
}

#[test]
fn integration_scan_small_tree() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "hello").unwrap();
    fs::write(dir.path().join("b.txt"), "world").unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub/c.txt"), "nested").unwrap();

    let index = scan(dir.path(), &ScanOptions::default()).unwrap();

    assert_eq!(index.len(), 4);
    assert!(index.get_by_path(std::path::Path::new("a.txt")).is_some());
    assert!(
        index
            .get_by_path(std::path::Path::new("sub/c.txt"))
            .is_some()
    );
}

#[test]
#[ignore = "slow: takes ~100s on large datasets; run explicitly with --include-ignored"]
fn integration_scan_large_dataset() {
    let Some(root) = get_test_root() else {
        eprintln!("Skipping: no large dataset available (set FF_TEST_ROOT or use NixOS)");
        return;
    };

    let index = scan(&root, &ScanOptions::default()).unwrap();

    assert!(
        index.len() > 1000,
        "expected large dataset, got {} entries",
        index.len()
    );

    let bytes_per_entry = 140;
    let metadata_bytes = index.len() * bytes_per_entry;

    let entries_with_inode = index.entries().iter().filter(|e| e.inode > 0).count();
    assert!(
        entries_with_inode > index.len() / 2,
        "most entries should have valid inodes, got {}/{}",
        entries_with_inode,
        index.len()
    );

    eprintln!(
        "Large dataset scan: {} files, ~{}MB metadata",
        index.len(),
        metadata_bytes / 1024 / 1024
    );
}

#[test]
fn integration_content_cache_eviction() {
    let mut cache = ff_index::ContentCache::new(1024);

    for i in 0..100 {
        let path = PathBuf::from(format!("file_{i}.txt"));
        let content = vec![b'x'; 50];
        cache.insert(path, content);
    }

    assert!(cache.current_bytes() <= 1024);
    assert!(!cache.is_empty());
}

#[test]
fn integration_content_cache_drop_all() {
    let mut cache = ff_index::ContentCache::new(1024);

    for i in 0..10 {
        let path = PathBuf::from(format!("file_{i}.txt"));
        cache.insert(path, vec![b'x'; 50]);
    }

    assert_eq!(cache.current_bytes(), 500);

    cache.drop_all();

    assert!(cache.is_empty());
    assert_eq!(cache.current_bytes(), 0);
}

#[test]
fn integration_index_with_content_cache() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.txt"), "hello world").unwrap();

    let mut index = Index::with_content_cache(dir.path().to_path_buf(), 1024);
    let entries = scan(dir.path(), &ScanOptions::default())
        .unwrap()
        .entries()
        .to_vec();
    index.set_entries(entries);

    assert_eq!(index.len(), 1);

    let path = PathBuf::from("test.txt");
    index.cache_content(path.clone(), b"hello world".to_vec());

    let content = index.get_content(&path);
    assert!(content.is_some());
    assert_eq!(content.unwrap(), b"hello world");
}

#[test]
fn integration_watcher_detects_changes() {
    let dir = TempDir::new().unwrap();
    let watcher = ff_watcher::FilesystemWatcher::new(dir.path()).unwrap();

    fs::write(dir.path().join("new_file.txt"), "content").unwrap();

    let events = watcher
        .try_wait_for_changes(std::time::Duration::from_secs(2))
        .unwrap();
    assert!(events.is_some());
    assert!(!events.unwrap().is_empty());
}

#[test]
fn integration_watcher_and_index_together() {
    let dir = TempDir::new().unwrap();
    let watcher = ff_watcher::FilesystemWatcher::new(dir.path()).unwrap();

    let mut index = scan(dir.path(), &ScanOptions::default()).unwrap();
    assert_eq!(index.len(), 0);

    fs::write(dir.path().join("file1.txt"), "a").unwrap();
    fs::write(dir.path().join("file2.txt"), "b").unwrap();

    let events = watcher
        .try_wait_for_changes(std::time::Duration::from_secs(2))
        .unwrap();
    assert!(events.is_some());

    index = scan(dir.path(), &ScanOptions::default()).unwrap();
    assert_eq!(index.len(), 2);
}
