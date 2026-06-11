# Changelog — Stage 4 (Tasks)

## v0.1.1 — Epic A Completed

- Completed Epic A: Foundation (8 tickets, 27 points)
- FOUND-A001: Workspace scaffolding with 7 crates
- FOUND-A002: Content cache eviction spike (3.1% overhead, batch eviction to 80%)
- FOUND-A003: ff-common types, config, paths, errors (18 tests)
- FOUND-A004: ff-index core, scanner, git status (11 tests)
- FOUND-A005: Content cache with byte-based LRU (9 tests)
- FOUND-A006: Watcher debouncing spike (500ms window recommended)
- FOUND-A007: FilesystemWatcher implementation (4 tests)
- FOUND-A008: Integration tests (7 tests)
- Total: 49 tests passing across foundation crates

## v0.1.0 — Initial Task Plan

- Decomposed 72 capabilities (P0 + P1) into 8 epics with 42 tickets
- Epic A: Foundation (8 tickets, 27 points) — ff-common, ff-index, ff-watcher
- Epic B: IPC & Daemon (6 tickets, 21 points) — ff-ipc, ff-daemon lifecycle and server
- Epic C: Content Search (5 tickets, 18 points) — grep engine, formatters, CLI subcommand
- Epic D: Name Search (5 tickets, 18 points) — search engine, formatters, CLI subcommand
- Epic E: Metadata Predicates (5 tickets, 21 points) — predicate types, expression parser, metadata engine
- Epic F: Metadata Actions (4 tickets, 16 points) — action executor, find formatters, CLI subcommand
- Epic G: CLI & Output (5 tickets, 15 points) — main dispatch, one-shot mode, daemon subcommand, completions
- Epic H: Polish (4 tickets, 11 points) — systemd, documentation, compatibility tests, benchmarks
- Planned 4 spikes: content cache eviction (SPK-A001), filesystem watcher debouncing (SPK-A002), JSON-RPC streaming (SPK-B001), find expression parser (SPK-E001)
- Identified critical path: A → B → E → F → G → H (111 of 147 points)
- Deferred P2 capabilities (PCRE2, ok/prompt) to v0.2.0
