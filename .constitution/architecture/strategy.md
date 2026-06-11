# Architecture Strategy

**Version:** v0.1.0

## Architectural Pattern

**Client-Daemon with In-Memory Index**

ff uses a persistent daemon architecture where a long-running background process maintains a complete in-memory index of the file tree. The CLI acts as a thin client that dispatches queries to the daemon via Unix domain sockets. The daemon's index eliminates repeated filesystem traversal, enabling sub-250ms query latency on trees with 1M+ files.

## Why This Pattern Fits

1. **Performance requirement demands persistent state.** The PRD requires <250ms hot query latency on 1M+ files. This is impossible if every query re-traverses the filesystem. A persistent daemon with an in-memory index is the only way to achieve this.

2. **Single-user, single-machine scope.** ff targets interactive developers on their local workstation. There is no multi-tenancy, no distributed system, no network boundary. A Unix socket between a local CLI and daemon is the simplest possible IPC mechanism that satisfies the constraint.

3. **Clear separation of concerns.** The daemon owns the expensive, stateful work (scanning, indexing, watching). The CLI owns the ephemeral, user-facing work (parsing flags, formatting output, exit codes). This separation makes each component independently testable and evolvable.

4. **One-shot mode as a fallback, not a separate architecture.** The PRD requires one-shot mode for scripts and CI. Rather than building a second architecture, one-shot mode reuses the same Index container with a short-lived lifecycle. This avoids code duplication and ensures feature parity.

5. **Streaming IPC for large result sets.** Content search and metadata queries can return thousands of matches. Streaming results over the socket (rather than buffering) prevents memory pressure in the daemon and provides incremental feedback to the user.

## Trade-offs Accepted

1. **Daemon lifecycle complexity.** A persistent daemon introduces failure modes that pure CLI tools don't have: stale sockets, crashed processes, idle timeout races, and auto-start detection. Mitigation: robust PID/socket file management, crash detection, and transparent auto-restart.

2. **Memory overhead.** The daemon holds the entire file tree in memory (~140 bytes/file for metadata, ~500 bytes/file with content). On a 1M-file tree, this is 140–500MB. Mitigation: configurable content index budget, LRU eviction, and one-shot mode for constrained environments.

3. **Index rebuild latency.** When the filesystem changes significantly (e.g., `git checkout` switching branches), the daemon must rebuild the index. During rebuild, queries block. Mitigation: filesystem watcher coalesces rapid changes to avoid unnecessary rebuilds; rebuilds are fast (~10s for 1M files) because they're sequential I/O.

4. **Per-root daemon proliferation.** Each indexed root gets its own daemon. A user working in 10 directories has 10 daemon processes. Mitigation: idle timeout (default 5 minutes) ensures daemons exit when not in use; systemd integration allows explicit lifecycle control.

5. **Streaming IPC complexity.** Streaming results requires handling partial results, cancellation, and backpressure. Mitigation: JSON-RPC 2.0 over Unix sockets is well-understood; Rust's async/await makes streaming natural.

6. **Full rebuild on change.** The architecture uses full index rebuilds rather than incremental updates. This is simpler but slower for frequent small changes. Mitigation: filesystem watcher debounces events; rebuilds are fast enough (<15s for 1M files) that the user rarely notices.
