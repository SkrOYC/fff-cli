# Spike: Filesystem Watcher Debouncing

**Spike ID:** SPK-A002  
**Related Epic:** EPIC-A-foundation  
**Related Ticket:** FOUND-A006  
**Timebox:** 2 days

## Objective

Design and validate the event debouncing strategy for the FilesystemWatcher to prevent excessive index rebuilds during bursts of filesystem activity.

## Background

The architecture specifies full index rebuilds on filesystem changes. The watcher must coalesce rapid changes into a single rebuild trigger. Without proper debouncing, operations like `git checkout`, `npm install`, or `cargo build` could trigger dozens of rebuilds in rapid succession.

The architecture specifies:
- 500ms coalescing window
- 2s maximum batch window
- Block queries during rebuild

We need to validate these parameters and design the debouncing mechanism.

## Investigation Areas

### 1. Event Burst Patterns

Profile real-world filesystem activity to understand burst patterns:
- `git checkout <branch>`: How many events? Over what duration?
- `cargo build`: How many events? Over what duration?
- `npm install`: How many events? Over what duration?
- Editor save (vim, vscode): How many events per save?
- `cargo fmt`: How many events?

**Questions to answer:**
- What's the typical burst duration?
- What's the typical inter-burst gap?
- Are there patterns that require different debouncing strategies?

### 2. Debouncing Strategy Options

Evaluate different debouncing approaches:

**Option A: Simple timer reset**
- On first event, start 500ms timer
- On subsequent events, reset timer
- Rebuild when timer fires

**Option B: Sliding window**
- Collect events in 500ms windows
- Rebuild at end of each window
- Merge consecutive windows within 2s

**Option C: Adaptive debouncing**
- Start with 500ms window
- If events continue, extend to 1s, then 2s
- Cap at 2s maximum

**Questions to answer:**
- Which strategy minimizes rebuild count without excessive latency?
- How do we handle very long operations (e.g., `npm install` taking 30s)?
- Should we have different debounce times for different event types?

### 3. notify Crate Behavior

Investigate the `notify` crate's event delivery:
- How does it coalesce events internally?
- Does it deliver events in batches or one-at-a-time?
- How does the debouncer (notify-debouncer-mini, notify-debouncer-full) work?
- What's the overhead of recursive watching on large trees?

**Questions to answer:**
- Should we use notify's built-in debouncer or implement our own?
- What's the event delivery latency?
- How many watches can we create before hitting inotify limits?

### 4. inotify Watch Limits

On Linux, inotify has a per-user watch limit (default 8192). For large trees:
- How many directories in a typical repo?
- How many in /nix/store (11k dirs)?
- Should we watch directories or use recursive polling?

**Questions to answer:**
- Do we need to increase `fs.inotify.max_user_watches`?
- Should we document this requirement?
- Can we use a hybrid approach (watch top-level dirs, poll subdirs)?

### 5. Rebuild Blocking Strategy

When a rebuild is triggered:
- New queries should block until rebuild completes
- In-flight queries should complete against old index
- Rebuild should wait for in-flight queries to finish

**Questions to answer:**
- How do we implement the blocking mechanism?
- Should we use a RwLock, Mutex, or channel?
- What's the maximum acceptable query delay during rebuild?

## Deliverables

1. **Event burst profile** documenting real-world patterns
2. **Debouncing strategy recommendation** with rationale
3. **notify crate evaluation** (built-in debouncer vs. custom)
4. **inotify limit analysis** with recommendations
5. **Implementation sketch** showing:
   - Debounce loop pseudocode
   - Event coalescing data structure
   - Rebuild trigger mechanism

## Success Criteria

- Debouncing reduces rebuild count by >80% during burst operations
- Maximum latency from change to rebuild trigger: <3s
- Works correctly on Linux (inotify) and macOS (FSEvents)
- No dropped events during debouncing window

## References

- Architecture: flows/flow-index-rebuild.md
- Architecture: resilience.md (Index Consistency)
- PRD: constraints.md (Index consistency: <2s after change)
