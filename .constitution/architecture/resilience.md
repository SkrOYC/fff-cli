# Resilience & Cross-Cutting Concerns

## Failure Handling

### Daemon Crash Recovery

**Scenario:** Daemon process crashes (segfault, OOM kill, unhandled panic).

**Detection:**
- CLI attempts to connect to socket, connection refused
- CLI checks PID file, finds stale PID (process not running)
- CLI deletes stale socket and PID files

**Recovery:**
- CLI automatically restarts daemon
- Daemon performs full index rebuild on startup
- Original query is retried against new daemon

**Trade-off:** User experiences a one-time delay (~10s for 1M files) on the query that detects the crash. Subsequent queries are fast.

---

### Socket Connection Failure

**Scenario:** CLI cannot connect to daemon socket (daemon not running, socket deleted, permission denied).

**Detection:**
- Connection refused: daemon not running
- No such file: socket deleted, daemon crashed
- Permission denied: socket owned by different user

**Recovery:**
- Connection refused / No such file: CLI triggers auto-start, retries connection
- Permission denied: CLI returns error (exit code 2), suggests checking socket ownership

**Timeout:** Connection attempt times out after 2 seconds. If daemon doesn't start within 2s, CLI returns error.

---

### Index Rebuild During Query

**Scenario:** FilesystemWatcher detects changes while a query is in flight.

**Detection:**
- Watcher notifies Index of changes
- Index sets rebuild flag

**Handling:**
- In-flight queries complete against current index (results may be stale)
- New queries block until rebuild completes
- Daemon returns "index rebuilding" status to blocked queries after 5s wait

**Trade-off:** Queries during rebuild may return stale results. This is acceptable because the user initiated the filesystem change and expects eventual consistency.

---

### Query Timeout

**Scenario:** Query takes too long (complex regex, huge result set, slow content retrieval).

**Detection:**
- CLI sets a query timeout (default 30s, configurable)
- If no results received within timeout, CLI cancels query

**Recovery:**
- CLI sends cancellation message to daemon
- Daemon aborts query, releases resources
- CLI returns error (exit code 2), suggests narrowing search

---

### Memory Pressure

**Scenario:** System memory is low, daemon is consuming significant memory for content index.

**Detection:**
- Daemon monitors system memory (via /proc/meminfo on Linux, sysctl on macOS)
- Triggers when available memory drops below 10% of total

**Recovery:**
- Daemon drops content index, retains metadata-only index
- Logs warning to stderr
- Subsequent content queries return error: "content index dropped due to memory pressure, restart daemon to rebuild"

**Trade-off:** Content search becomes unavailable until daemon is restarted. Metadata and name search continue to work.

---

### Stale PID File

**Scenario:** Daemon crashes without cleaning up PID file. Next startup detects stale PID.

**Detection:**
- On startup, daemon reads PID file
- Checks if PID is still running (kill(pid, 0))
- If not running, PID file is stale

**Recovery:**
- Daemon deletes stale PID file and socket file
- Proceeds with normal startup

---

## Security

### Socket Isolation

**Requirement:** Unix socket must be accessible only to the user who started the daemon.

**Implementation:**
- Socket created with mode 0600 (owner read/write only)
- Socket directory `/tmp/fffd/` created with mode 0700
- Daemon runs as the invoking user (no privilege escalation)

**Verification:**
- On socket connection, daemon verifies peer credentials (SO_PEERCRED on Linux)
- Rejects connections from different UIDs

---

### File Permission Respect

**Requirement:** ff must not index or cache content for files the daemon cannot read.

**Implementation:**
- During index scan, daemon checks file readability before reading content
- Unreadable files are indexed for metadata (path, size, mtime) but content is not cached
- Content queries on unreadable files return "permission denied" error

---

### Action Safety

**Requirement:** Destructive actions must require explicit user opt-in.

**Implementation:**
- `-delete` action requires explicit flag; not triggered by default
- `-exec` with write commands (rm, mv, etc.) is user's responsibility; ff does not inspect command content
- `-ok` and `-okdir` prompt for confirmation before each execution
- Daemon does not have special privileges; actions run with user's permissions

---

## Telemetry & Logging

### Daemon Logs

**Location:** `$XDG_STATE_HOME/ff/daemon.log` or `~/.local/state/ff/daemon.log`

**Format:** Structured JSON lines

**Fields:**
- timestamp (ISO 8601)
- level (info, warn, error, debug)
- message
- context (root path, query type, elapsed time)

**Rotation:** Log file rotated at 10MB, 3 files retained

**Content:**
- Daemon startup/shutdown events
- Index rebuild start/complete/duration
- Query dispatch (type, parameters, result count, elapsed time)
- Filesystem change events (coalesced count)
- Errors (connection failures, query timeouts, memory pressure)

---

### Query Tracing

**Requirement:** Debugging slow queries requires visibility into query execution.

**Implementation:**
- Each query assigned a unique ID (UUID)
- Query ID included in daemon logs
- CLI can request query trace via `ff daemon trace <query-id>` (future enhancement)

---

## Configuration

### Configuration Hierarchy

1. **CLI flags** (highest priority): Override all other sources
2. **Environment variables:** `FF_*` prefix (e.g., `FF_IDLE_TIMEOUT`)
3. **Config file:** `$XDG_CONFIG_HOME/ff/config.toml` or `~/.config/ff/config.toml`
4. **Defaults** (lowest priority): Built-in defaults

---

### Configuration Parameters

**Daemon:**
- `idle_timeout`: Duration before daemon exits (default: 5m)
- `content_budget_mb`: Memory budget for content index (default: 256MB)
- `log_level`: Logging verbosity (default: info)

**Index:**
- `index_content`: Whether to index file content (default: true)
- `respect_ignore`: Whether to respect .gitignore/.ignore (default: true)
- `include_hidden`: Whether to include hidden files (default: false)

**Query:**
- `max_results`: Maximum results per query (default: 10000)
- `query_timeout`: Query timeout duration (default: 30s)

---

## Data Integrity

### Index Consistency

**Requirement:** Index must reflect current filesystem state within 2 seconds of a change.

**Implementation:**
- FilesystemWatcher uses platform-specific APIs (inotify, FSEvents) for immediate notification
- Watcher coalesces events over 500ms window to batch rapid changes
- Index rebuild triggered after coalescing window closes
- Rebuild completes in ~10s for 1M files

**Trade-off:** Queries during rebuild may return stale results. This is acceptable for interactive use.

---

### Content Index Integrity

**Requirement:** Cached file content must match current file content.

**Implementation:**
- Content indexed by file path + mtime + size
- On filesystem change, affected files' content is invalidated
- Rebuild re-indexes content for changed files

**Trade-off:** Full rebuild re-indexes all content, not just changed files. This is slower but simpler than incremental content updates.
