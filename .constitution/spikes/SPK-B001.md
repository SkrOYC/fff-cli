# Spike: JSON-RPC Streaming over Unix Sockets

**Spike ID:** SPK-B001  
**Related Epic:** EPIC-B-ipc-daemon  
**Related Ticket:** IPCD-B001  
**Timebox:** 2 days

## Objective

Design and validate the streaming protocol for JSON-RPC 2.0 over Unix domain sockets, ensuring efficient handling of large result sets with backpressure and cancellation support.

## Background

The architecture specifies:
- JSON-RPC 2.0 over Unix domain sockets
- Length-prefixed message framing (4-byte big-endian u32 + JSON payload)
- Streaming results (daemon sends multiple notifications, then final response)
- tokio async runtime

We need to validate this approach and design the streaming protocol.

## Investigation Areas

### 1. Length-Prefixed Framing with tokio_util

Evaluate `tokio_util::codec::LengthDelimitedCodec`:
- How does it handle partial reads?
- What's the overhead of length prefix vs. newline-delimited?
- How does it interact with tokio's backpressure?

**Questions to answer:**
- What's the optimal max frame length? (TechSpec says 100MB)
- Should we use big-endian or little-endian length?
- How do we handle frame size limits?

### 2. Streaming Protocol Design

Design the streaming protocol for large result sets:

**Option A: Notification stream + final response**
```
CLI → Daemon: Request (id=1)
Daemon → CLI: Notification (result, batch 1)
Daemon → CLI: Notification (result, batch 2)
Daemon → CLI: Response (id=1, final summary)
```

**Option B: Chunked response**
```
CLI → Daemon: Request (id=1)
Daemon → CLI: Response chunk 1 (id=1, continuation=true)
Daemon → CLI: Response chunk 2 (id=1, continuation=true)
Daemon → CLI: Response chunk N (id=1, continuation=false)
```

**Questions to answer:**
- Which approach is simpler to implement?
- Which approach handles cancellation better?
- How do we handle errors mid-stream?

### 3. Backpressure

When the CLI can't consume results fast enough:
- Unix socket buffer fills up
- tokio applies backpressure to daemon
- Daemon should pause query execution

**Questions to answer:**
- How does tokio handle socket buffer full?
- Should we implement explicit flow control?
- What's the socket buffer size? Can we configure it?

### 4. Cancellation

When the CLI disconnects or user hits Ctrl+C:
- Daemon should stop query execution
- Resources should be released
- Partial results should be discarded

**Questions to answer:**
- How do we detect client disconnection?
- Should we use tokio::select! for cancellation?
- How do we handle cancellation during action execution (e.g., -exec)?

### 5. Batch Size Optimization

For large result sets (100k+ matches), sending one notification per match is inefficient. We need to batch results.

**Questions to answer:**
- What's the optimal batch size? (10, 100, 1000 items?)
- Should batch size be configurable?
- How do we balance latency (small batches) vs. throughput (large batches)?

### 6. Error Handling Mid-Stream

If an error occurs during query execution:
- Some results may have been sent
- Client has already started processing

**Questions to answer:**
- Should we send an error notification and abort?
- Should we send a final response with error?
- How does the client handle partial results + error?

## Deliverables

1. **Protocol specification** with:
   - Message format (request, notification, response)
   - Streaming protocol (notification stream vs. chunked response)
   - Batch size recommendations
   - Error handling strategy

2. **Benchmark results** comparing:
   - Notification stream vs. chunked response
   - Different batch sizes (10, 100, 1000)
   - Throughput for 100k results

3. **Implementation sketch** showing:
   - LengthDelimitedCodec configuration
   - Server-side streaming loop
   - Client-side result consumption
   - Cancellation handling

## Success Criteria

- Streaming protocol handles 100k results without memory issues
- Backpressure works correctly (daemon pauses when CLI is slow)
- Cancellation stops query within 100ms of client disconnect
- Batch size optimization reduces overhead by >50% vs. per-item notifications

## References

- TechSpec: ADR-003 (Length-prefixed JSON-RPC 2.0)
- TechSpec: contracts/ipc-protocol.md
- Architecture: flows/flow-content-search.md
