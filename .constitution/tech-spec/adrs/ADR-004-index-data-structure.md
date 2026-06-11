# ADR-004: HashMap + Vec Index Data Structure

## Status

Accepted

## Context

The Index container maintains an in-memory representation of the file tree. Each file entry contains metadata (path, size, mtime, mode, uid, gid, inode, device, link_count, file_type) and optionally cached content.

Requirements:
- Fast path lookups (O(1) preferred)
- Fast iteration over all entries (for queries)
- Memory efficient (~140 bytes/file for metadata, ~500 bytes/file with content)
- Support for 1M+ files

Options considered:
1. **HashMap<PathBuf, FileEntry> + Vec<FileEntry>:** O(1) lookups, fast iteration
2. **BTreeMap<PathBuf, FileEntry>:** Sorted iteration, O(log n) lookups
3. **Custom trie:** Prefix-based lookups, complex implementation

## Decision

Use **HashMap<PathBuf, FileEntry> + Vec<FileEntry>** with the following structure:

```rust
pub struct Index {
    entries: Vec<FileEntry>,              // All entries (for iteration)
    path_to_index: HashMap<PathBuf, usize>, // Path → index in entries Vec
    content_cache: LruCache<PathBuf, Vec<u8>>, // Optional content cache
}

pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub mtime: i64,
    pub atime: i64,
    pub ctime: i64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub inode: u64,
    pub device: u64,
    pub link_count: u64,
    pub file_type: FileType,
    pub git_status: GitStatus,
}
```

## Consequences

### Positive

- **O(1) path lookups:** HashMap provides constant-time lookups by path
- **Fast iteration:** Vec provides sequential memory access (cache-friendly)
- **Memory efficient:** ~140 bytes/file for metadata (matches PRD constraint)
- **Simple implementation:** Standard library types, well-understood
- **Content cache separation:** LRU cache is independent, can be dropped under memory pressure

### Negative

- **Dual storage:** Path stored in both HashMap key and FileEntry (redundant)
- **Index invalidation:** Rebuild requires rebuilding both HashMap and Vec
- **No sorted order:** Vec order is arbitrary (not sorted by path)

### Trade-offs Accepted

- **HashMap vs. BTreeMap:** We sacrifice sorted order for O(1) lookups
- **Vec + HashMap vs. single HashMap:** We accept redundancy for fast iteration
- **No trie:** We sacrifice prefix-based optimizations for simplicity

## Memory Layout

For 1M files:
- **Metadata only:** ~140MB (140 bytes/file × 1M files)
- **With content cache:** ~500MB (500 bytes/file × 1M files)
- **HashMap overhead:** ~48 bytes/entry (hash table overhead)
- **Vec overhead:** ~24 bytes (Vec metadata)

Total: ~140MB (metadata) to ~500MB (with content) — within PRD constraints.

## Query Patterns

### Path Lookup (O(1))

```rust
let entry = index.get_by_path(&path);
```

### Iteration (O(n))

```rust
for entry in index.entries() {
    // Process entry
}
```

### Filtered Iteration (O(n))

```rust
for entry in index.entries() {
    if matches_filter(entry, &filters) {
        // Process matching entry
    }
}
```

## Alternatives Considered

### BTreeMap<PathBuf, FileEntry>

```rust
pub struct Index {
    entries: BTreeMap<PathBuf, FileEntry>,
}
```

**Rejected because:**
- O(log n) lookups (slower than HashMap for large trees)
- Sorted iteration not required for our use case
- Higher memory overhead (~160 bytes/file vs. ~140 bytes/file)

### Custom Trie

```rust
pub struct Index {
    root: TrieNode,
}

struct TrieNode {
    children: HashMap<String, TrieNode>,
    entries: Vec<FileEntry>,
}
```

**Rejected because:**
- Complex implementation (custom data structure)
- Overhead for prefix-based queries (not a primary use case)
- Harder to maintain and debug

## Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| Path lookup | O(1) | HashMap lookup |
| Insert | O(1) | HashMap insert + Vec push |
| Iteration | O(n) | Sequential Vec iteration |
| Rebuild | O(n) | Clear and rebuild both structures |
| Memory | O(n) | ~140 bytes/file (metadata only) |

## References

- [Rust HashMap documentation](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
- [Rust Vec documentation](https://doc.rust-lang.org/std/vec/struct.Vec.html)
