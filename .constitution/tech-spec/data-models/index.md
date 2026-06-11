# Index Data Model

## Overview

The Index is the core data structure of ff. It holds an in-memory representation of the file tree, including metadata for every file and directory, and optionally caches file content for content search.

## FileEntry

The fundamental unit of the index. One `FileEntry` per file or directory.

```rust
pub struct FileEntry {
    /// Relative path from index root
    pub path: PathBuf,

    /// File size in bytes
    pub size: u64,

    /// Last modification time (Unix timestamp, nanoseconds)
    pub mtime: i64,

    /// Last access time (Unix timestamp, nanoseconds)
    pub atime: i64,

    /// Last status change time (Unix timestamp, nanoseconds)
    pub ctime: i64,

    /// Permission bits (mode_t)
    pub mode: u32,

    /// Owner user ID
    pub uid: u32,

    /// Owner group ID
    pub gid: u32,

    /// Inode number
    pub inode: u64,

    /// Device ID (filesystem identifier)
    pub device: u64,

    /// Hard link count
    pub link_count: u64,

    /// File type
    pub file_type: FileType,

    /// Git status (if in a git repository)
    pub git_status: GitStatus,
}
```

### Memory Layout

| Field | Type | Size |
|-------|------|------|
| path | PathBuf | 24 bytes (pointer) + variable |
| size | u64 | 8 bytes |
| mtime | i64 | 8 bytes |
| atime | i64 | 8 bytes |
| ctime | i64 | 8 bytes |
| mode | u32 | 4 bytes |
| uid | u32 | 4 bytes |
| gid | u32 | 4 bytes |
| inode | u64 | 8 bytes |
| device | u64 | 8 bytes |
| link_count | u64 | 8 bytes |
| file_type | FileType (u8) | 1 byte |
| git_status | GitStatus (u8) | 1 byte |
| **Padding** | | 6 bytes |
| **Total (fixed)** | | **~100 bytes** |
| **Path storage** | | **~40 bytes (avg)** |
| **Total per entry** | | **~140 bytes** |

With content cache: ~500 bytes/file (avg 360 bytes content per file).

## FileType

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FileType {
    RegularFile = 0,
    Directory = 1,
    Symlink = 2,
    Socket = 3,
    BlockDevice = 4,
    CharacterDevice = 5,
    Fifo = 6,
    Unknown = 7,
}
```

## GitStatus

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GitStatus {
    Untracked = 0,
    Unmodified = 1,
    Modified = 2,
    Added = 3,
    Deleted = 4,
    Renamed = 5,
    Copied = 6,
    UpdatedButUnmerged = 7,
    NotInGit = 255,
}
```

## Index Structure

```rust
pub struct Index {
    /// All file entries (for iteration)
    entries: Vec<FileEntry>,

    /// Path → index in entries Vec (for O(1) lookups)
    path_to_index: HashMap<PathBuf, usize>,

    /// Root path of the indexed tree
    root: PathBuf,

    /// Optional content cache
    content_cache: Option<ContentCache>,

    /// Whether the index is currently rebuilding
    rebuilding: AtomicBool,

    /// Timestamp of last successful rebuild
    last_rebuild: Instant,
}
```

## ContentCache

```rust
pub struct ContentCache {
    /// LRU cache of file content
    cache: LruCache<PathBuf, Vec<u8>>,

    /// Memory budget in bytes
    budget_bytes: usize,

    /// Current memory usage in bytes
    current_bytes: usize,
}
```

## State Invariants

1. **entries.len() == path_to_index.len()** — Every entry has a path mapping
2. **All path_to_index values < entries.len()** — Indices are valid
3. **current_bytes <= budget_bytes** — Content cache respects budget (after eviction)
4. **rebuilding == true → queries block** — No queries during rebuild

## Migration Notes

### v0.1.0 (Initial)

- Initial schema with all fields listed above
- No migration needed (greenfield)

### Future Changes

- **Adding fields:** Add to end of struct, provide default value
- **Removing fields:** Deprecate in one release, remove in next major
- **Changing types:** Requires full index rebuild (automatic on daemon restart)
- **Changing path representation:** Requires full index rebuild

## Persistence

The index is **not persisted to disk**. It is rebuilt from scratch on every daemon startup. This is intentional:
- Simplifies implementation (no serialization/deserialization)
- Ensures consistency (no stale persisted state)
- Rebuild is fast (~10s for 1M files)
- Content cache would be invalid after restart anyway

Future consideration: persist metadata-only index to disk for faster startup (skip stat calls). Not planned for v1.0.
