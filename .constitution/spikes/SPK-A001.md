# Spike: Content Cache Eviction Strategy

**Spike ID:** SPK-A001  
**Related Epic:** EPIC-A-foundation  
**Related Ticket:** FOUND-A002  
**Timebox:** 2 days

## Objective

Evaluate LRU eviction strategies for the content cache to determine the optimal approach for balancing memory usage, query performance, and implementation complexity.

## Background

The Index container optionally caches file content for content search queries. The cache must:
- Respect a configurable memory budget (default 256MB)
- Evict entries when budget is exceeded
- Support dropping the entire cache under memory pressure
- Provide O(1) get/insert operations

The TechSpec specifies `hashlink::LruCache` as the implementation, but we need to validate:
1. Byte-based budget enforcement (not just entry count)
2. Eviction performance under load
3. Memory pressure detection and response
4. Interaction with filesystem watcher rebuilds

## Investigation Areas

### 1. Byte-Based Budget Enforcement

`hashlink::LruCache` uses entry count as capacity, not bytes. We need to:
- Track total bytes in cache separately
- Implement custom eviction loop to enforce byte budget
- Measure overhead of byte tracking vs. pure LRU

**Questions to answer:**
- What's the performance impact of tracking bytes on every insert/get?
- Should we evict multiple entries at once or one-at-a-time?
- How do we handle files larger than the entire budget?

### 2. Eviction Performance Under Load

Simulate 1M files with varying content sizes (1KB - 10MB). Measure:
- Insert throughput with eviction
- Query latency with cache hits vs. misses
- Memory fragmentation over time

**Questions to answer:**
- Does eviction cause latency spikes?
- Should we use background eviction or synchronous?
- What's the optimal eviction batch size?

### 3. Memory Pressure Detection

The daemon must detect system memory pressure and drop the content cache. Investigate:
- Linux: `/proc/meminfo` parsing, `MemAvailable` threshold
- macOS: `sysctl` `vm.swapusage`, `hw.memsize`
- Threshold: What % of available memory triggers cache drop?

**Questions to answer:**
- How often should we check memory pressure?
- Should we check on every query or on a timer?
- What's the recovery path after cache drop?

### 4. Interaction with Filesystem Watcher

When the filesystem watcher triggers a rebuild, the content cache must be invalidated. Investigate:
- Should we drop the entire cache on rebuild?
- Can we selectively invalidate changed files?
- How does rebuild frequency affect cache hit rate?

**Questions to answer:**
- What's the cache hit rate during active development (frequent rebuilds)?
- Should we pause caching during rebuild?
- How do we handle files that change during a query?

## Deliverables

1. **Recommendation document** with:
   - Chosen eviction strategy (with rationale)
   - Byte budget enforcement approach
   - Memory pressure detection thresholds
   - Rebuild invalidation strategy

2. **Benchmark results** comparing:
   - Pure LRU (entry count) vs. byte-based LRU
   - Synchronous vs. background eviction
   - Cache hit rates under different rebuild frequencies

3. **Implementation sketch** showing:
   - `ContentCache` struct with byte tracking
   - Eviction loop pseudocode
   - Memory pressure check integration

## Success Criteria

- Clear recommendation on eviction strategy
- Benchmarks show <5% overhead for byte tracking
- Memory pressure detection works on Linux and macOS
- Rebuild invalidation strategy doesn't degrade query performance

## References

- TechSpec: ADR-005 (hashlink LruCache)
- Architecture: containers.md (Index container)
- PRD: constraints.md (Memory ceiling: 500MB for 1M files)
