# Epic A: Foundation

**Status:** Active  
**Scope:** ff-common types, ff-index core, ff-watcher  
**Dependencies:** None  
**Story Points:** 27

## Overview

Establish the foundational crates that all other components depend on: shared types and utilities (ff-common), the in-memory file index with content cache (ff-index), and the filesystem watcher with debouncing (ff-watcher).

This epic includes two spikes to de-risk the content cache eviction strategy and filesystem watcher debouncing approach before committing to implementation.

---

#### FOUND-A001 Workspace Scaffolding
- **Type:** Chore
- **Effort:** 2
- **Dependencies:** None
- **Description:** Create the Cargo workspace with all 7 crate stubs (ff-cli, ff-daemon, ff-index, ff-watcher, ff-query, ff-ipc, ff-common). Configure workspace-level dependencies, rust-toolchain.toml (1.96+, 2024 edition), .cargo/config.toml, and CI scripts. Each crate should compile independently with a minimal lib.rs or main.rs.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the repository is cloned
When running `cargo build --workspace`
Then all 7 crates compile without errors

Given the workspace root
When running `cargo fmt --check`
Then all crates pass formatting checks

Given the workspace root
When running `cargo clippy --workspace -- -D warnings`
Then all crates pass linting with zero warnings
```

---

#### FOUND-A002 Spike: Content Cache Eviction Strategy
- **Type:** Spike
- **Effort:** 3
- **Dependencies:** FOUND-A001
- **Description:** Investigate LRU eviction strategies for the content cache. Evaluate byte-based budget enforcement, eviction performance under load, memory pressure detection, and interaction with filesystem watcher rebuilds. Output: recommendation document with benchmarks and implementation sketch. See SPK-A001.md.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the spike is complete
When reading .constitution/spikes/SPK-A001.md
Then it contains a clear recommendation on eviction strategy
And it includes benchmark results for byte tracking overhead
And it specifies memory pressure detection thresholds for Linux and macOS

Given the spike recommendation
When reviewing the implementation sketch
Then it shows a ContentCache struct with byte tracking
And it demonstrates eviction loop pseudocode
```

---

#### FOUND-A003 ff-common: Shared Types and Utilities
- **Type:** Feature
- **Effort:** 3
- **Dependencies:** FOUND-A001
- **Description:** Implement the ff-common crate with shared types (FileType, GitStatus, CaseMode, PatternMode, ExecMode), configuration loading (TOML config file with env var and CLI flag overrides), path computation (socket path, PID path, log path), and common error types. These types are used by all other crates.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the ff-common crate
When importing ff_common::types::FileType
Then all file type variants are available (RegularFile, Directory, Symlink, Socket, BlockDevice, CharacterDevice, Fifo, Unknown)

Given a config file at ~/.config/ff/config.toml
When calling ff_common::config::load()
Then configuration values are loaded with correct precedence (CLI > env > config > defaults)

Given the current working directory is /home/user/project
When calling ff_common::paths::socket_path()
Then it returns /tmp/fffd-<user>/<sha256>.sock with correct permissions
```

---

#### FOUND-A004 ff-index: Core Index and Scanner
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** FOUND-A003
- **Description:** Implement the ff-index crate with FileEntry struct (path, size, mtime, atime, ctime, mode, uid, gid, inode, device, link_count, file_type, git_status), Index struct (HashMap + Vec storage), and file tree scanner using the `ignore` crate for gitignore-aware walking. The scanner must collect stat metadata for every file and directory.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given a directory tree with 1000 files
When calling Index::scan(root_path)
Then the index contains exactly 1000 FileEntry objects
And each entry has correct metadata (size, mtime, mode, uid, gid, inode)

Given an index with 1000 files
When calling index.get_by_path("src/main.rs")
Then it returns the correct FileEntry in O(1) time

Given a directory with .gitignore containing "target/"
When scanning the directory
Then files under target/ are excluded from the index

Given an index being rebuilt
When a query is in flight
Then the query completes against the old index
And new queries block until rebuild completes
```

---

#### FOUND-A005 ff-index: Content Cache
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** FOUND-A004, FOUND-A002
- **Description:** Implement the ContentCache using hashlink::LruCache with byte-based budget enforcement (per SPK-A001 recommendation). Support configurable memory budget, LRU eviction when budget exceeded, and full cache drop for memory pressure. Integrate with the Index struct so content is lazily loaded on first access.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given a content cache with 256MB budget
When inserting files totaling 300MB
Then the cache evicts LRU entries to stay under 256MB
And the most recently accessed files remain cached

Given a content cache with cached content
When calling cache.get(path)
Then it returns the cached content in O(1) time
And the entry is marked as recently used

Given a content cache under memory pressure
When the daemon detects low system memory
Then calling cache.drop_all() frees all cached content
And metadata-only queries continue to work
```

---

#### FOUND-A006 Spike: Filesystem Watcher Debouncing
- **Type:** Spike
- **Effort:** 3
- **Dependencies:** FOUND-A001
- **Description:** Investigate event debouncing strategies for the FilesystemWatcher. Profile real-world burst patterns (git checkout, cargo build, npm install), evaluate notify crate behavior, analyze inotify watch limits, and design rebuild blocking strategy. Output: recommendation document with event burst profile and implementation sketch. See SPK-A002.md.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the spike is complete
When reading .constitution/spikes/SPK-A002.md
Then it contains an event burst profile for common operations
And it recommends a debouncing strategy with rationale
And it evaluates notify's built-in debouncer vs. custom implementation

Given the spike recommendation
When reviewing the inotify limit analysis
Then it specifies the watch count for typical repos and /nix/store
And it documents whether sysctl changes are required
```

---

#### FOUND-A007 ff-watcher: Filesystem Watcher
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** FOUND-A006
- **Description:** Implement the ff-watcher crate using the `notify` crate for cross-platform filesystem notifications (inotify on Linux, FSEvents on macOS). Implement event debouncing per SPK-A002 recommendation. The watcher must coalesce rapid changes and notify the Index to trigger rebuilds.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given a watcher monitoring a directory
When a file is created
Then the watcher emits a Create event within 100ms

Given a watcher monitoring a directory
When 100 files are created in rapid succession (burst)
Then the watcher coalesces events into a single rebuild trigger
And the rebuild is triggered within 3s of the last event

Given a watcher on a directory with 50000 subdirectories
When the watcher is started
Then it successfully creates watches for all directories
And it does not exceed inotify watch limits (or documents the requirement)
```

---

#### FOUND-A008 Foundation Integration Tests
- **Type:** Feature
- **Effort:** 2
- **Dependencies:** FOUND-A004, FOUND-A005, FOUND-A007
- **Description:** Write integration tests for the foundation crates: index scan correctness, content cache eviction, watcher event delivery, rebuild blocking. Use tempfile for test fixtures. Verify memory usage stays within PRD constraints (~140 bytes/file metadata, ~500 bytes/file with content).
- **Acceptance Criteria (Gherkin):**
```gherkin
Given a test directory with 10000 files
When scanning and measuring memory
Then the index uses less than 2MB (140 bytes/file × 10000 + overhead)

Given an index with content cache at 10MB budget
When inserting 20MB of content
Then memory usage stays under 12MB (budget + overhead)

Given a watcher and index
When creating and deleting files
Then the index reflects the changes after the debounce window
And queries during rebuild return correct results

Given the test suite
When running `cargo nextest run -p ff-index -p ff-watcher -p ff-common`
Then all tests pass with zero failures
```
