# Logical Risks & Technical Debt

## Risk 1: Full Rebuild Latency on Frequent Changes

**Severity:** Medium

**Description:** The architecture uses full index rebuilds on any filesystem change. If the user is actively modifying files (e.g., running a build system that touches thousands of files), the daemon may repeatedly rebuild, causing query delays.

**Impact:**
- Queries block during rebuild (~10s for 1M files)
- User experiences intermittent slowness
- Daemon CPU usage spikes during rebuild

**Mitigation:**
- FilesystemWatcher debounces events (500ms coalescing window)
- If multiple changes arrive within 2s, they're batched into a single rebuild
- Daemon prioritizes in-flight queries over rebuilds (rebuild waits for queries to complete)

**Residual Risk:**
- If changes are continuous (e.g., watch mode build), daemon may spend most time rebuilding
- User may need to temporarily stop daemon or switch to one-shot mode

**Follow-up:**
- Monitor rebuild frequency in daemon logs
- If rebuilds are too frequent, consider incremental updates for future versions

---

## Risk 2: Memory Pressure from Content Index

**Severity:** Medium

**Description:** The content index can consume up to 500MB for 1M files. On memory-constrained systems (e.g., 4GB RAM laptops, CI runners), this may cause swapping or OOM kills.

**Impact:**
- System performance degrades due to swapping
- Daemon may be OOM-killed, requiring restart
- User may need to disable content indexing

**Mitigation:**
- Configurable content budget (`content_budget_mb`, default 256MB)
- LRU eviction when budget exceeded
- Memory pressure detection: daemon drops content index when system memory < 10%
- One-shot mode for constrained environments (no persistent memory usage)

**Residual Risk:**
- User may not be aware of memory usage until system slows down
- Dropping content index disables content search until daemon restart

**Follow-up:**
- Document memory usage in README
- Consider adding `ff daemon status` command showing memory usage
- Consider lazy content loading (index content only when first queried)

---

## Risk 3: Socket File Conflicts

**Severity:** Low

**Description:** Multiple users on the same system may have conflicting socket paths if `/tmp/fffd/` is shared. Socket path is based on `sha256(root)`, which doesn't include the username.

**Impact:**
- User A's daemon creates socket at `/tmp/fffd/<hash>.sock`
- User B's daemon tries to create socket at same path, fails
- User B cannot start daemon

**Mitigation:**
- Socket directory includes username: `/tmp/fffd-<username>/<hash>.sock`
- Socket file permissions (0600) prevent cross-user access
- Daemon verifies peer credentials on connection

**Residual Risk:**
- None if mitigation is implemented correctly

**Follow-up:**
- Ensure socket path includes username in tech spec

---

## Risk 4: FilesystemWatcher Limitations

**Severity:** Medium

**Description:** Platform-specific filesystem notification APIs have limitations:
- inotify: Limited number of watches (default 8192 on Linux), no recursive watching
- FSEvents: Delayed notifications (up to 1s), may miss rapid changes

**Impact:**
- Large directory trees may exceed inotify watch limit
- Watcher may miss changes, causing stale index
- User queries return outdated results

**Mitigation:**
- inotify: Increase watch limit via sysctl (`fs.inotify.max_user_watches`)
- inotify: Implement recursive watching manually (add watch per directory)
- FSEvents: Use latency=0 for immediate notifications
- Fallback: Periodic full scan (every 60s) to catch missed changes

**Residual Risk:**
- inotify watch limit may require user intervention (sysctl change)
- Periodic full scan adds CPU overhead

**Follow-up:**
- Document inotify watch limit in README
- Consider adding `ff daemon check` command to verify watcher health

---

## Risk 5: Query Result Size Overflow

**Severity:** Low

**Description:** Queries can return millions of results (e.g., `ff find -type f` on a large tree). Streaming results over the socket may cause memory pressure in the CLI or slow output rendering.

**Impact:**
- CLI memory usage grows with result count
- Output rendering becomes slow (terminal can't keep up)
- User may need to kill the query

**Mitigation:**
- CLI processes results incrementally (doesn't buffer all results)
- OutputFormatter writes to stdout immediately (no buffering)
- CLI respects `max_results` configuration (default 10000)
- User can pipe to `head` to limit output

**Residual Risk:**
- If user disables `max_results`, CLI may consume significant memory
- Terminal rendering is outside ff's control

**Follow-up:**
- Document `max_results` configuration
- Consider adding warning when result count exceeds 100k

---

## Risk 6: Regex Denial of Service

**Severity:** Low

**Description:** Malicious or poorly-written regex patterns can cause catastrophic backtracking, hanging the query indefinitely.

**Impact:**
- Query hangs, consuming CPU
- User must kill the query
- Daemon may become unresponsive if query is not cancellable

**Mitigation:**
- Query timeout (default 30s)
- Regex engine uses backtracking limits (Rust's regex crate is safe by default)
- CLI can cancel query via socket message

**Residual Risk:**
- User may disable timeout for legitimate slow queries
- Rust's regex crate doesn't support all regex features (e.g., backreferences), which may frustrate users

**Follow-up:**
- Document regex limitations (no backreferences, no lookaround without PCRE2)
- Consider adding PCRE2 support as optional feature (P2 capability C-11)

---

## Technical Debt 1: No Incremental Index Updates

**Description:** The architecture uses full rebuilds instead of incremental updates. This is simpler but less efficient for frequent small changes.

**Impact:**
- Rebuilds take ~10s for 1M files, even if only one file changed
- CPU and I/O overhead from re-scanning entire tree

**Rationale for Acceptance:**
- Incremental updates are complex (handle renames, moves, hard links, race conditions)
- Full rebuilds are fast enough for interactive use (<15s for 1M files)
- FilesystemWatcher debouncing reduces rebuild frequency

**Future Consideration:**
- If users report frequent rebuild latency, consider incremental updates for v2.0
- Incremental updates would require tracking file identity (inode + device) and handling edge cases

---

## Technical Debt 2: No Query Cancellation Propagation

**Description:** If the CLI disconnects (user hits Ctrl+C), the daemon may continue processing the query until it completes or times out.

**Impact:**
- Daemon wastes CPU on abandoned queries
- Resources (file handles, memory) held until query completes

**Rationale for Acceptance:**
- Query timeout (30s) limits waste
- Daemon can detect disconnected socket and abort query
- Implementation complexity of graceful cancellation is high

**Future Consideration:**
- Implement query cancellation via socket close detection
- Daemon monitors socket readability; if socket closes, abort query

---

## Technical Debt 3: No Query Result Caching

**Description:** Identical queries are re-executed from scratch. No caching of previous results.

**Impact:**
- Repeated queries (e.g., user running same search multiple times) take full time
- No benefit from query locality

**Rationale for Acceptance:**
- Query results may be stale if filesystem changed
- Caching adds complexity (invalidation, memory management)
- Queries are fast enough (<250ms) that caching provides minimal benefit

**Future Consideration:**
- If users report slow repeated queries, consider result caching with TTL
- Cache key: query parameters + filesystem change counter
