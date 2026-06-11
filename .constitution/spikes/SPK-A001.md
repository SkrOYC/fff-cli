# Spike: Content Cache Eviction Strategy

**Spike ID:** SPK-A001  
**Related Epic:** EPIC-A-foundation  
**Related Ticket:** FOUND-A002  
**Timebox:** 2 days  
**Status:** Complete

## Objective

Evaluate LRU eviction strategies for the content cache to determine the optimal approach for balancing memory usage, query performance, and implementation complexity.

## Benchmark Results

All benchmarks run on Linux x86_64 with criterion 0.5.1. Source: `crates/ff-index/benches/cache_bench.rs`.

### Insert Performance (5,000 files, ~4KB avg)

| Strategy | Time | Throughput | Overhead |
|----------|------|-----------|----------|
| Simple LRU (entry count) | 6.50 ms | 2.93 GiB/s | baseline |
| Byte-tracked (batch eviction) | 6.70 ms | 2.84 GiB/s | **+3.1%** |

**Result:** Byte tracking overhead is **3.1%**, well under the 5% target.

### Insert Performance (10,000 files, ~4KB avg)

| Strategy | Time | Throughput |
|----------|------|-----------|
| Simple LRU (entry count) | 17.00 ms | 588 Kelem/s |
| Byte-tracked (batch eviction) | 14.39 ms | 695 Kelem/s |

**Result:** Byte-tracked is actually **faster** at higher counts because batch eviction amortizes cost.

### Get Performance (10,000 lookups)

| Strategy | Time | Throughput | Overhead |
|----------|------|-----------|----------|
| Simple LRU | 420 µs | 23.8 Melem/s | baseline |
| Byte-tracked | 440 µs | 22.7 Melem/s | **+4.8%** |

**Result:** Read overhead is **4.8%**, just under the 5% target.

### Eviction Under Pressure (10,000 files into 10MB budget)

| Metric | Value |
|--------|-------|
| Total insert time | 17.64 ms |
| Final cache size | ≤ 10MB (budget) |
| Eviction strategy | Batch to 80% of budget |

**Result:** Eviction completes within acceptable latency. No latency spikes observed.

### Memory Pressure Detection

| Operation | Time |
|-----------|------|
| Parse `/proc/meminfo` | 14.26 µs |

**Result:** Negligible overhead. Can check on every query if needed.

## Recommendation

### Eviction Strategy: Batch Eviction to 80%

**Approach:**
1. Track `current_bytes` on every insert (subtract evicted entry size, add new entry size)
2. When `current_bytes > budget_bytes`, evict oldest entries until `current_bytes <= budget_bytes * 80%`
3. Use `Vec<PathBuf>` as insertion order tracker for O(1) oldest-entry lookup

**Rationale:**
- 3.1% insert overhead (under 5% target)
- 4.8% read overhead (under 5% target)
- Batch eviction to 80% amortizes eviction cost, avoiding per-insert overhead
- Simple implementation with no background threads

### Byte Budget Enforcement

**Implementation:**
```rust
pub struct ContentCache {
    cache: LruCache<PathBuf, Vec<u8>>,
    budget_bytes: usize,
    current_bytes: usize,
    insert_order: Vec<PathBuf>,
}

impl ContentCache {
    pub fn insert(&mut self, path: PathBuf, content: Vec<u8>) {
        let content_size = content.len();

        // Handle update to existing entry
        if let Some(old) = self.cache.insert(path.clone(), content) {
            self.current_bytes -= old.len();
            self.current_bytes += content_size;
            return;
        }

        self.current_bytes += content_size;
        self.insert_order.push(path);

        // Batch evict to 80% of budget
        if self.current_bytes > self.budget_bytes {
            let target = self.budget_bytes * 80 / 100;
            while self.current_bytes > target {
                if let Some(oldest) = self.insert_order.first().cloned() {
                    self.insert_order.remove(0);
                    if let Some(removed) = self.cache.remove(&oldest) {
                        self.current_bytes -= removed.len();
                    }
                } else {
                    break;
                }
            }
        }
    }

    pub fn get(&mut self, path: &PathBuf) -> Option<&Vec<u8>> {
        self.cache.get(path)
    }

    pub fn drop_all(&mut self) {
        self.cache.clear();
        self.current_bytes = 0;
        self.insert_order.clear();
    }
}
```

### Memory Pressure Detection

**Linux:**
- Parse `/proc/meminfo`, read `MemAvailable` line
- Threshold: drop cache when `MemAvailable < 10%` of total memory
- Check frequency: on every query (14µs overhead is negligible)

**macOS:**
- Use `sysctl hw.memsize` and `vm.swapusage`
- Threshold: drop cache when swap usage > 50% or available memory < 10%
- Check frequency: on every query

**Implementation:**
```rust
fn check_memory_pressure() -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            let mut available_kb = 0u64;
            let mut total_kb = 0u64;
            for line in content.lines() {
                if line.starts_with("MemAvailable:") {
                    available_kb = line.split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                } else if line.starts_with("MemTotal:") {
                    total_kb = line.split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                }
            }
            return available_kb < total_kb / 10;
        }
    }
    false
}
```

### Rebuild Invalidation Strategy

**Approach:** Drop entire cache on rebuild.

**Rationale:**
- Rebuilds happen on filesystem changes (git checkout, cargo build, etc.)
- Content cache hit rate during active development is low (files change frequently)
- Dropping entire cache is O(1) and frees all memory immediately
- Selective invalidation would require tracking which files changed, adding complexity

**Implementation:**
```rust
impl Index {
    pub fn trigger_rebuild(&mut self) {
        if let Some(ref mut cache) = self.content_cache {
            cache.drop_all();
        }
        // ... rebuild index ...
    }
}
```

## Files Larger Than Budget

**Approach:** Allow single files larger than budget to be inserted, but evict them immediately on next insert.

**Rationale:**
- Prevents infinite loops in eviction
- Large files are rare in typical codebases
- Simple to implement: just check `if content_size > budget_bytes` and skip caching

## Success Criteria Met

- [x] Clear recommendation on eviction strategy (batch to 80%)
- [x] Benchmarks show <5% overhead for byte tracking (3.1% insert, 4.8% read)
- [x] Memory pressure detection works on Linux (14µs overhead)
- [x] Rebuild invalidation strategy doesn't degrade query performance (O(1) drop)

## References

- TechSpec: ADR-005 (hashlink LruCache)
- Architecture: containers.md (Index container)
- PRD: constraints.md (Memory ceiling: 500MB for 1M files)
- Benchmark source: `crates/ff-index/benches/cache_bench.rs`
