# ADR-003: Length-Prefixed JSON-RPC 2.0 IPC

## Status

Accepted

## Context

ff uses a client-daemon architecture where the CLI communicates with the daemon over Unix domain sockets. The architecture specifies JSON-RPC 2.0 as the protocol, but we need to decide on message framing.

Options considered:
1. **Newline-delimited JSON (NDJSON):** Each message is a JSON object followed by `\n`
2. **Length-prefixed messages:** 4-byte big-endian length prefix followed by JSON payload
3. **File descriptor passing:** Use `jsonrpc_fdpass` crate for advanced FD passing

## Decision

Use **length-prefixed JSON-RPC 2.0** with the following format:

```
┌─────────────────┬─────────────────┐
│ Length (4 bytes) │ JSON payload    │
│ big-endian u32   │ (variable)      │
└─────────────────┴─────────────────┘
```

- **Length field:** 4-byte big-endian unsigned integer (max message size: 4GB)
- **JSON payload:** Valid JSON-RPC 2.0 message (request, response, or notification)
- **Encoding:** UTF-8
- **Streaming:** Use `tokio_util::codec::LengthDelimitedCodec` for framing

## Consequences

### Positive

- **Explicit boundaries:** Length prefix makes message boundaries explicit
- **Large messages:** Can handle large result sets without buffering entire message in memory
- **Binary safety:** Works with any JSON content (no special handling for newlines in strings)
- **Standard pattern:** Length-prefixed framing is well-understood and widely used
- **Tokio integration:** `LengthDelimitedCodec` provides efficient streaming

### Negative

- **Complexity:** More complex than NDJSON (need to handle length field)
- **Debugging:** Harder to read raw socket traffic (binary length prefix)
- **Overhead:** 4-byte overhead per message (negligible)

### Trade-offs Accepted

- **Length-prefixed vs. NDJSON:** We sacrifice simplicity for robustness with large messages
- **No FD passing:** We don't need file descriptor passing (no advanced IPC requirements)

## Alternatives Considered

### Newline-Delimited JSON (NDJSON)

```
{"jsonrpc":"2.0","method":"grep","params":{...}}\n
{"jsonrpc":"2.0","method":"search","params":{...}}\n
```

**Rejected because:**
- JSON strings can contain newlines (need escaping, adds complexity)
- Large messages may be split across multiple reads (need buffering logic)
- Less explicit about message boundaries

### File Descriptor Passing

```rust
// jsonrpc_fdpass crate
let message = MessageWithFds::new(JsonRpcMessage::Request(request), vec![fd]);
```

**Rejected because:**
- Over-engineering for our use case
- We don't need to pass file descriptors between CLI and daemon
- Adds complexity and dependency on less-maintained crate

## Implementation Notes

### Codec Configuration

```rust
use tokio_util::codec::LengthDelimitedCodec;

let codec = LengthDelimitedCodec::builder()
    .max_frame_length(100 * 1024 * 1024) // 100MB max message size
    .length_field_length(4)
    .length_adjustment(0)
    .num_skip(0)
    .big_endian()
    .new_codec();
```

### Message Types

```rust
#[derive(Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,           // "2.0"
    method: String,            // "grep", "search", "find", "ping", "shutdown"
    params: serde_json::Value, // Method-specific parameters
    id: u64,                   // Request ID
}

#[derive(Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,           // "2.0"
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: u64,                   // Matches request ID
}

#[derive(Serialize, Deserialize)]
struct JsonRpcNotification {
    jsonrpc: String,           // "2.0"
    method: String,            // "result", "progress", "error"
    params: serde_json::Value,
}
```

### Streaming Results

For large result sets, daemon sends multiple notifications followed by a final response:

```
CLI → Daemon: Request (id=1, method="grep", params={...})
Daemon → CLI: Notification (method="result", params={items:[...]})
Daemon → CLI: Notification (method="result", params={items:[...]})
Daemon → CLI: Response (id=1, result={totalMatched:1234, elapsedMs:52})
```

## References

- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [tokio_util::codec::LengthDelimitedCodec](https://docs.rs/tokio-util/latest/tokio_util/codec/struct.LengthDelimitedCodec.html)
