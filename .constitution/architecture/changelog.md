# Changelog — Stage 2 (Architecture)

## v0.1.0 — Initial Architecture

- Established client-daemon architecture with in-memory index
- Defined 9 logical containers: CLI, Daemon, Index, FilesystemWatcher, ContentQueryEngine, NameQueryEngine, MetadataQueryEngine, ActionExecutor, OutputFormatter
- Documented communication patterns: Unix socket IPC (CLI ↔ Daemon), in-process calls (Daemon ↔ engines), streaming results
- Identified key architectural decisions:
  - Separate Index container from Daemon for testability
  - Three separate query engines (Content, Name, Metadata)
  - Separate ActionExecutor from MetadataQueryEngine
  - Single shared OutputFormatter for all query types
  - Separate FilesystemWatcher container
  - One-shot mode reuses same Index container
  - Daemon lifecycle management as part of Daemon container
  - Streaming IPC for large result sets
  - Full rebuild on filesystem changes
  - Block queries during rebuild
- Documented resilience concerns: daemon crash recovery, socket connection failures, index rebuild during queries, query timeouts, memory pressure, stale PID files
- Documented security concerns: socket isolation, file permission respect, action safety
- Identified 6 risks: full rebuild latency, memory pressure, socket conflicts, filesystem watcher limitations, query result size overflow, regex DoS
- Identified 3 technical debt items: no incremental updates, no query cancellation propagation, no query result caching
