# ADR-005: hashlink LruCache for Content Index

## Status

Accepted

## Context

The Index container optionally caches file content for content search. The cache must:
- Support configurable memory budget (default 256MB)
- Evict least-recently-used entries when budget exceeded
- Provide O(1) get/insert operations
- Allow dropping entire cache under memory pressure

Options considered:
1. **hashlink::LruCache:** HashMap-like container with LRU eviction
2. **lru crate:** Popular LRU cache implementation
3. **Custom LRU:** HashMap + doubly-linked list

## Decision

Use **hashlink::LruCache<PathBuf, Vec<u8>>** for the content cache.

```rust
use hashlink::LruCache;

pub struct ContentCache {
    cache: LruCache<PathBuf, Vec<u8>>,
    budget_bytes: usize,
    current_bytes: usize,
}

impl ContentCache {
    pub fn new(budget_mb: usize) -> Self {
        let budget_bytes = budget_mb * 1024 * 1024;
        let capacity = budget_bytes / 500; // Estimate avg file size
        Self {
            cache: LruCache::new(capacity),
            budget_bytes,
            current_bytes: 0,
        }
    }

    pub fn get(&mut self, path: &PathBuf) -> Option<&Vec<u8>> {
        self.cache.get(path)
    }

    pub fn insert(&mut self, path: PathBuf, content: Vec<u8>) {
        let size = content.len();
        if let Some(old) = self.cache.insert(path, content) {
            self.current_bytes -= old.len();
        }
        self.current_bytes += size;
        self.evict_if_needed();
    }

    fn evict_if_needed(&mut self) {
        while self.current_bytes > self.budget_bytes {
            if let Some((_, old)) = self.cache.remove_lru() {
                self.current_bytes -= old.len();
            } else {
                break;
            }
        }
    }

    pub fn drop_all(&mut self) {
        self.cache.clear();
        self.current_bytes = 0;
    }
}
```

## Consequences

### Positive

- **O(1) operations:** get/insert are constant time
- **Automatic LRU eviction:** Built-in LRU tracking
- **Memory budget:** Can enforce byte-based budget (not just entry count)
- **Production-tested:** hashlink is used by many projects (200M+ downloads)
- **Raw entry API:** Advanced API for optimizing key construction
- **Well-maintained:** Active development, MSRV 1.65

### Negative

- **Unsafe code:** hashlink contains unsafe code (linked list manipulation)
- **Capacity vs. bytes:** LruCache uses entry count, need wrapper for byte budget
- **Dependency:** Adds hashlink dependency (mitigated by wide adoption)

### Trade-offs Accepted

- **hashlink vs. lru:** We choose hashlink for raw entry API and better maintenance
- **Byte budget vs. entry count:** We implement wrapper to track bytes (not just count)
- **Unsafe code:** We accept unsafe code in dependency (mitigated by miri testing)

## Alternatives Considered

### lru crate

```rust
use lru::LruCache;

let mut cache = LruCache::new(NonZeroUsize::new(1000).unwrap());
```

**Rejected because:**
- Entry-count based (no byte budget support)
- No raw entry API (less optimization opportunities)
- Less actively maintained than hashlink

### Custom LRU

```rust
pub struct LruCache {
    map: HashMap<PathBuf, *mut Node>,
    list: DoublyLinkedList,
}
```

**Rejected because:**
- More code to maintain
- Risk of bugs in unsafe linked list code
- hashlink already solves this problem well

## Memory Management

### Budget Enforcement

```rust
// Default budget: 256MB
let cache = ContentCache::new(256);

// Insert content
cache.insert(path, content);

// If current_bytes > budget_bytes, evict LRU entries
// until under budget
```

### Memory Pressure Handling

```rust
// Daemon monitors system memory
if system_memory_low() {
    content_cache.drop_all();
    log::warn!("Content cache dropped due to memory pressure");
}
```

## Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| get | O(1) | HashMap lookup + LRU update |
| insert | O(1) | HashMap insert + LRU update |
| remove_lru | O(1) | Remove least-recently-used entry |
| clear | O(n) | Drop all entries |
| Memory | O(n) | ~content_size per entry |

## References

- [hashlink crate documentation](https://docs.rs/hashlink/latest/hashlink/)
- [hashlink::LruCache documentation](https://docs.rs/hashlink/latest/hashlink/lru_cache/struct.LruCache.html)
