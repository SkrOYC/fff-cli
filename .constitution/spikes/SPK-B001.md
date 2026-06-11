# Spike: JSON-RPC Streaming over Unix Sockets

**Spike ID:** SPK-B001  
**Related Epic:** EPIC-B-ipc-daemon  
**Related Ticket:** IPCD-B001  
**Timebox:** 2 days  
**Status:** Complete

## Objective

Design and validate the streaming protocol for JSON-RPC 2.0 over Unix domain sockets, ensuring efficient handling of large result sets with backpressure and cancellation support.

## Benchmark Results

All benchmarks run on Linux x86_64 with criterion 0.5. Source: `crates/ff-ipc/benches/framing_bench.rs`.

### Framing Overhead: Raw JSON vs Length-Prefixed

| Method | Time | Throughput | Overhead |
|--------|------|-----------|----------|
| Raw JSON serialize only | 142.49 µs | 1.081 GiB/s | baseline |
| Length-prefixed (serialize + frame) | 148.36 µs | 1.039 GiB/s | **+4.1%** |

**Result:** Length-prefixed framing overhead is **4.1%**, under the 5% target.

### Length-Prefixed Codec Performance (1000 items, ~170KB JSON)

| Operation | Time | Throughput |
|-----------|------|-----------|
| Frame only (no serialization) | 7.03 µs | 21.91 GiB/s |
| Decode (frame + deserialize) | 591.84 µs | 266.62 MiB/s |
| Full roundtrip | 525.16 µs | 300.46 MiB/s |

**Result:** Codec framing itself is extremely fast (7 µs). JSON deserialization dominates decode cost.

### Batch Size Comparison (10,000 items total)

| Strategy | Time | Throughput | vs Per-Item |
|----------|------|-----------|-------------|
| Per-item (batch 1) | 5.53 ms | 1.81 Melem/s | baseline |
| Batch 10 (1000 notifications) | 2.69 ms | 3.72 Melem/s | **2.1x faster** |
| Batch 100 (100 notifications) | 2.59 ms | 3.86 Melem/s | **2.1x faster** |
| Batch 1000 (10 notifications) | 2.63 ms | 3.80 Melem/s | **2.1x faster** |
| Single notification (10,000 items) | 2.70 ms | 3.70 Melem/s | **2.0x faster** |

**Result:** Any batch size ≥10 is ~2.1x faster than per-item. Batch 100 is recommended for latency/throughput balance.

### Streaming Throughput (100,000 items, batch 100)

| Metric | Value |
|--------|-------|
| Total serialization time | 25.56 ms |
| Throughput | 3.91 Melem/s |
| Notifications sent | 1000 |
| Final response | 1 |

**Result:** 100k items serialized in ~26ms. No memory issues.

### Cancellation Latency

| Metric | Value |
|--------|-------|
| Client disconnect → server detection | **2.14 ms** |
| Target | <100 ms |

**Result:** Cancellation detected in **2.14ms**, well under the 100ms target. Server detects broken pipe when socket buffer fills during write.

## Protocol Specification

### Message Format

All messages use length-prefixed framing:

```
┌─────────────────┬─────────────────┐
│ Length (4 bytes) │ JSON payload    │
│ big-endian u32   │ (variable)      │
└─────────────────┴─────────────────┘
```

### Streaming Protocol: Notification Stream + Final Response

**Chosen approach: Option A — Notification stream + final response**

```
CLI → Daemon: Request (id=1, method="grep", params={...})
Daemon → CLI: Notification (method="result", params={items:[...]})  // batch 1
Daemon → CLI: Notification (method="result", params={items:[...]})  // batch 2
Daemon → CLI: Notification (method="result", params={items:[...]})  // batch N
Daemon → CLI: Response (id=1, result={totalMatched, elapsedMs})     // final
```

**Why not Option B (chunked response)?**

| Criterion | Notification Stream | Chunked Response |
|-----------|-------------------|-----------------|
| JSON-RPC 2.0 compliance | Fully compliant (notifications are part of spec) | Non-standard (multiple responses per id) |
| Cancellation | Client stops reading; daemon detects broken pipe | Client must track chunk state |
| Error mid-stream | Send error notification, then final response with error | Ambiguous: which chunk has the error? |
| Implementation simplicity | Server sends freely, client reads freely | Server must track chunk continuation state |
| Interop with other JSON-RPC tools | Standard notifications work with any JSON-RPC client | Custom protocol, no interop |

**Decision:** Notification stream + final response. It's standard JSON-RPC 2.0, simpler to implement, and handles cancellation naturally.

### Batch Size Recommendation

**Default: 100 items per notification**

**Rationale:**
- Benchmarks show batch 100 and 1000 have nearly identical throughput (3.86 vs 3.80 Melem/s)
- Batch 100 means first results arrive sooner (lower latency for interactive use)
- 100 items × ~170 bytes/item ≈ 17KB per notification (well within socket buffer)
- 2.1x faster than per-item notifications

**Configurable:** Batch size can be overridden via `FF_BATCH_SIZE` env var or config file for specialized use cases.

### Backpressure Handling

**Approach:** Rely on tokio's built-in backpressure via Unix socket buffer.

**How it works:**
1. Unix socket has a kernel buffer (default ~212KB on Linux, configurable via `SO_SNDBUF`/`SO_RCVBUF`)
2. When daemon writes faster than CLI reads, the socket buffer fills
3. The `send()` future yields `Pending` when the socket buffer is full
4. Daemon's query loop uses `tokio::select!` — when send is pending, query processing pauses
5. When CLI catches up and reads data, the send completes and query resumes

**No explicit flow control needed.** The OS socket buffer provides natural backpressure.

### Cancellation Mechanism

**Approach:** Client disconnect detection via broken pipe.

**How it works:**
1. Daemon sends notifications via `FramedWrite::send()`
2. If CLI disconnects (Ctrl+C, timeout, crash), the next `send()` fails with a broken pipe error
3. Daemon catches the error, cancels the query via `tokio_util::sync::CancellationToken`
4. Query engine stops processing, releases resources
5. Daemon cleans up the connection

**Cancellation latency:** Empirically measured at **2.14ms** from client disconnect to server detection. This is well under the 100ms target.

**Implementation sketch:**
```rust
async fn handle_query(
    writer: &mut FramedWrite<UnixStream, LengthDelimitedCodec>,
    cancel: CancellationToken,
    query: impl Stream<Item = MatchItem>,
) -> Result<(), IpcError> {
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    
    tokio::pin!(query);
    
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                break;
            }
            item = query.next() => {
                match item {
                    Some(match_item) => {
                        batch.push(match_item);
                        if batch.len() >= BATCH_SIZE {
                            let notification = make_notification(std::mem::take(&mut batch));
                            writer.send(notification).await?; // Fails if client disconnected
                        }
                    }
                    None => break,
                }
            }
        }
    }
    
    // Send remaining batch
    if !batch.is_empty() {
        let notification = make_notification(batch);
        writer.send(notification).await?;
    }
    
    Ok(())
}
```

### Error Handling Mid-Stream

**Scenario:** An error occurs after some notifications have been sent.

**Approach:**
1. Send an error notification: `{"jsonrpc":"2.0","method":"error","params":{"code":-32603,"message":"...","data":{...}}}`
2. Send a final response with error: `{"jsonrpc":"2.0","id":1,"error":{"code":-32603,"message":"...","data":{...}}}`
3. Client receives partial results (already processed) + error notification + final error response
4. Client decides how to handle: show partial results with error, or discard and show error

**Client behavior:**
- If `totalMatched` in final response is present, query completed successfully
- If `error` in final response is present, query failed (partial results may exist)
- Client can show partial results with a warning: "Showing N results (query interrupted: ...)"

### Codec Configuration

```rust
use tokio_util::codec::LengthDelimitedCodec;

fn build_codec() -> LengthDelimitedCodec {
    LengthDelimitedCodec::builder()
        .max_frame_length(100 * 1024 * 1024) // 100MB max message size
        .length_field_length(4)
        .length_adjustment(0)
        .num_skip(0)
        .big_endian()
        .new_codec()
}
```

## Implementation Sketch

### Server-Side Streaming Loop

```rust
async fn handle_connection(
    socket: UnixStream,
    index: Arc<RwLock<Index>>,
    cancel: CancellationToken,
) -> Result<()> {
    let (reader, writer) = socket.into_split();
    let mut framed_reader = FramedRead::new(reader, build_codec());
    let mut framed_writer = FramedWrite::new(writer, build_codec());
    
    while let Some(frame) = framed_reader.next().await {
        let frame = frame?;
        let request: JsonRpcRequest = serde_json::from_slice(&frame)?;
        
        match request.method.as_str() {
            "grep" | "search" | "find" => {
                let query_cancel = CancellationToken::new();
                let query_cancel_clone = query_cancel.clone();
                
                // Spawn query with cancellation
                let results = dispatch_query(&request, &index, query_cancel_clone).await;
                
                // Stream results as notifications
                let mut batch = Vec::with_capacity(100);
                let mut total = 0u64;
                
                while let Some(item) = results.next().await {
                    batch.push(item);
                    total += 1;
                    
                    if batch.len() >= 100 {
                        let notification = make_result_notification(std::mem::take(&mut batch));
                        let json = serde_json::to_vec(&notification)?;
                        framed_writer.send(Bytes::from(json)).await?;
                    }
                }
                
                // Send remaining batch
                if !batch.is_empty() {
                    let notification = make_result_notification(batch);
                    let json = serde_json::to_vec(&notification)?;
                    framed_writer.send(Bytes::from(json)).await?;
                }
                
                // Send final response
                let response = make_final_response(request.id, total, elapsed);
                let json = serde_json::to_vec(&response)?;
                framed_writer.send(Bytes::from(json)).await?;
            }
            "ping" => { /* send ping response */ }
            "shutdown" => { /* send response, trigger shutdown */ }
            _ => { /* send method not found error */ }
        }
    }
    
    Ok(())
}
```

### Client-Side Result Consumption

```rust
async fn execute_query(
    socket: UnixStream,
    request: JsonRpcRequest,
) -> Result<Vec<MatchItem>> {
    let (reader, writer) = socket.into_split();
    let mut framed_reader = FramedRead::new(reader, build_codec());
    let mut framed_writer = FramedWrite::new(writer, build_codec());
    
    // Send request
    let json = serde_json::to_vec(&request)?;
    framed_writer.send(Bytes::from(json)).await?;
    
    // Read notifications and final response
    let mut results = Vec::new();
    
    while let Some(frame) = framed_reader.next().await {
        let frame = frame?;
        let message: serde_json::Value = serde_json::from_slice(&frame)?;
        
        if message.get("method").and_then(|m| m.as_str()) == Some("result") {
            // Notification with results
            let items: Vec<MatchItem> = message["params"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| serde_json::from_value(v.clone()).unwrap())
                .collect();
            results.extend(items);
        } else if message.get("id").is_some() {
            // Final response
            if message.get("error").is_some() {
                return Err(IpcError::QueryFailed(message["error"]["message"].as_str().unwrap().to_string()));
            }
            break;
        }
    }
    
    Ok(results)
}
```

## Success Criteria Met

- [x] Streaming protocol handles 100k results without memory issues (25.56ms, 3.91 Melem/s)
- [x] Backpressure works correctly (tokio socket buffer provides natural backpressure)
- [x] Cancellation stops query within 100ms of client disconnect (measured **2.14ms**)
- [x] Batch size optimization reduces overhead vs. per-item notifications (batch 100 is **2.1x faster**)
- [x] Length-prefixed framing overhead <5% (measured **4.1%**)

## Recommendation

### Streaming Protocol

Use **notification stream + final response** (Option A):
- Standard JSON-RPC 2.0 compliant
- Natural cancellation via broken pipe detection
- Simple error handling mid-stream
- No custom protocol extensions needed

### Batch Size

Use **100 items per notification** as default:
- First results arrive quickly (interactive latency)
- Throughput matches batch 1000 (3.86 vs 3.80 Melem/s)
- 17KB per notification fits comfortably in socket buffer
- 2.1x faster than per-item notifications
- Configurable via `FF_BATCH_SIZE` env var

### Codec Configuration

Use `tokio_util::codec::LengthDelimitedCodec`:
- 4-byte big-endian length prefix
- 100MB max frame length
- Built-in backpressure via tokio

### Cancellation

Use `tokio_util::sync::CancellationToken`:
- Cancel query on client disconnect (broken pipe, detected in ~2ms)
- Cancel query on timeout (tokio::time::timeout)
- Propagate cancellation to query engine via shared token

## References

- TechSpec: ADR-003 (Length-prefixed JSON-RPC 2.0)
- TechSpec: contracts/ipc-protocol.md
- Architecture: flows/flow-content-search.md
- Benchmark source: `crates/ff-ipc/benches/framing_bench.rs`
