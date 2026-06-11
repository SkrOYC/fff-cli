# Product Vision

**Version:** v0.1.0

## Executive Summary

ff is a unified command-line tool that replaces rg, fd, and find by keeping a persistent in-memory index of the file tree. A background daemon performs a single full scan on startup, after which every query hits warm memory — eliminating the repeated filesystem traversal that bottlenecks traditional tools. The product exposes three subcommands (`ff grep`, `ff search`, `ff find`) that cover the full flag surface of the tools they replace, targeting interactive developers working in terminal environments on Linux and macOS.

## Jobs to Be Done

1. **Search file contents across large trees without waiting.** Developers routinely search for strings, patterns, and symbols across repositories and system directories containing hundreds of thousands to millions of files. Current tools re-traverse the entire tree on every invocation, causing multi-second to minute-long waits. ff must return content search results in under 250ms on trees with 1M+ files after the initial index is warm.

2. **Locate files by name or glob pattern instantly.** Developers frequently need to find files by name fragment, extension, or path pattern. ff must return file name matches from the in-memory index without any filesystem traversal at query time.

3. **Query file metadata with the expressive power of find.** Developers need to filter files by size, modification time, permissions, ownership, inode, and other metadata — including compound boolean expressions. ff must evaluate these predicates against cached metadata, not by re-stating files.

4. **Operate as a persistent background service with zero friction.** The daemon must auto-start on first query, require no manual lifecycle management, and auto-exit after a configurable idle period. A systemd user unit must be provided for environments where explicit service management is preferred.

5. **Serve as a complete replacement, not a partial alternative.** ff must support the full flag and feature surface of rg, fd, and find. When a feature cannot be supported natively, ff must return a clear error with the equivalent command in the original tool, rather than silently degrading or delegating.

## Appendix: Operator Preferences

The following technical preferences were stated during the PRD stage and are recorded here as non-binding implementation hints for downstream stages:

- **Language:** Rust
- **Runtime:** Native binary (no VM/runtime dependency)
- **IPC:** JSON-RPC 2.0 over Unix domain sockets
- **Daemon socket path:** `/tmp/fffd/<sha256(cwd)>.sock`
- **Package management:** Nix flake for build and dev shell
- **Existing dependency:** `fff-search` crate (v0.9.4) for core search/indexing
- **Distribution:** Single static binary
