# Epic B: IPC & Daemon

**Status:** Active  
**Scope:** ff-ipc protocol, ff-daemon lifecycle and server  
**Dependencies:** Epic A (Foundation)  
**Story Points:** 21

## Overview

Implement the IPC protocol layer (ff-ipc) and daemon process (ff-daemon). This includes JSON-RPC 2.0 message framing over Unix sockets, daemon lifecycle management (PID files, socket creation, idle timeout), auto-start detection, and the socket server that dispatches queries to query engines.

This epic includes one spike to validate the streaming protocol design before implementation.

---

#### IPCD-B001 Spike: JSON-RPC Streaming Protocol
- **Type:** Spike
- **Effort:** 3
- **Dependencies:** FOUND-A001
- **Description:** Design and validate the streaming protocol for JSON-RPC 2.0 over Unix sockets. Evaluate length-prefixed framing with tokio_util, design notification stream vs. chunked response approach, implement backpressure handling, and design cancellation mechanism. Output: protocol specification with benchmarks and implementation sketch. See SPK-B001.md.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the spike is complete
When reading .constitution/spikes/SPK-B001.md
Then it contains a protocol specification with message formats
And it recommends streaming approach (notification stream vs. chunked)
And it specifies optimal batch size for large result sets

Given the spike benchmark results
When reviewing throughput measurements
Then they show <5% overhead for length-prefixed framing
And cancellation stops query within 100ms of client disconnect
```

---

#### IPCD-B002 ff-ipc: Protocol Types and Codec
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** IPCD-B001, FOUND-A003
- **Description:** Implement the ff-ipc crate with JSON-RPC 2.0 message types (Request, Response, Notification, Error), length-prefixed codec using tokio_util::codec::LengthDelimitedCodec, Unix socket transport, and message definitions for all RPC methods (grep, search, find, ping, shutdown). Define request/response parameter schemas per the IPC contract.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given a JSON-RPC request message
When serializing and deserializing
Then the message round-trips without data loss
And the length prefix is correctly encoded as 4-byte big-endian u32

Given a Unix socket connection
When sending a 1MB message
Then the message is transmitted successfully
And the receiver decodes it correctly

Given an invalid JSON message
When attempting to decode
Then a parse error is returned with clear error message
And the connection remains open for subsequent messages
```

---

#### IPCD-B003 ff-daemon: Lifecycle Management
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** IPCD-B002, FOUND-A004, FOUND-A007
- **Description:** Implement daemon lifecycle management: PID file creation and validation, Unix socket creation with correct permissions (0600), idle timeout tracking and auto-exit, stale socket/PID detection and cleanup, and graceful shutdown on SIGTERM/SIGINT. The daemon must initialize the Index (scan file tree) and start the FilesystemWatcher on startup.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the daemon is started
When checking the PID file
Then it contains the correct process ID
And the socket file exists with mode 0600

Given a daemon that has been idle for 5 minutes
When the idle timeout expires
Then the daemon exits gracefully
And it cleans up the PID and socket files

Given a stale PID file from a crashed daemon
When a new daemon is started
Then it detects the stale PID
And removes the old PID and socket files
And starts successfully

Given a running daemon
When receiving SIGTERM
Then it completes in-flight queries
And exits with code 0
And cleans up all temporary files
```

---

#### IPCD-B004 ff-daemon: Socket Server and Query Dispatch
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** IPCD-B003
- **Description:** Implement the socket server that accepts client connections, decodes JSON-RPC requests, dispatches to the appropriate query engine (ContentQueryEngine, NameQueryEngine, MetadataQueryEngine), streams results back as notifications, and sends final response. Handle concurrent connections, query timeouts, and error responses.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given a running daemon
When a client connects and sends a grep request
Then the daemon dispatches to ContentQueryEngine
And streams results back as notifications
And sends a final response with totalMatched and elapsedMs

Given a running daemon
When multiple clients connect concurrently
Then each client receives correct responses
And queries execute in parallel without interference

Given a query that takes longer than the timeout
When the timeout expires
Then the daemon cancels the query
And sends an error response to the client
And releases all query resources
```

---

#### IPCD-B005 ff-daemon: Auto-Start Detection
- **Type:** Feature
- **Effort:** 3
- **Dependencies:** IPCD-B003
- **Description:** Implement auto-start detection: when the CLI attempts to connect and the socket doesn't exist or connection is refused, the CLI must detect this condition, spawn the daemon process, wait for the socket to appear (up to 2s timeout), and retry the connection. Handle race conditions when multiple CLI instances try to auto-start simultaneously.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given no daemon is running
When the CLI executes a query
Then it detects the missing socket
And spawns the daemon process
And waits for the socket to appear (up to 2s)
And successfully connects and executes the query

Given two CLI instances executing simultaneously with no daemon
When both detect the missing socket
Then only one daemon is started (no race condition)
And both CLI instances successfully connect

Given a daemon that crashed (stale socket)
When the CLI attempts to connect
Then it detects the connection failure
And removes the stale socket
And triggers auto-start
And successfully executes the query
```

---

#### IPCD-B006 IPC & Daemon Integration Tests
- **Type:** Feature
- **Effort:** 2
- **Dependencies:** IPCD-B004, IPCD-B005
- **Description:** Write integration tests for the IPC layer and daemon: socket connection, message framing, query dispatch, streaming results, auto-start, idle timeout, crash recovery. Use assert_cmd for CLI testing and tempfile for isolated test environments.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the test suite
When running `cargo nextest run -p ff-daemon -p ff-ipc`
Then all tests pass with zero failures

Given a test that spawns a daemon
When the test completes
Then the daemon process is cleaned up
And no stale socket or PID files remain

Given a test that sends 1000 concurrent queries
When all queries complete
Then all responses are correct
And no memory leaks are detected
```
