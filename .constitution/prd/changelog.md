# Changelog — Stage 1 (PRD)

## v0.1.0 — Initial PRD

- Established product vision: ff as a unified CLI replacing rg, fd, and find via persistent in-memory indexing
- Defined target actors: interactive developer (primary), script author (secondary)
- Documented 6 epics with prioritized capabilities:
  - Epic 1: Content Search (11 capabilities, 6 P0 / 4 P1 / 1 P2)
  - Epic 2: Name Search (17 capabilities, 8 P0 / 9 P1)
  - Epic 3: Metadata Query (22 capabilities, 9 P0 / 12 P1 / 1 P2)
  - Epic 4: Daemon & Indexing (11 capabilities, 6 P0 / 5 P1)
  - Epic 5: CLI Interface (7 capabilities, 5 P0 / 2 P1)
  - Epic 6: Output Formatting (6 capabilities, 4 P0 / 2 P1)
- Set non-functional constraints: <250ms hot query on 1M files, <500MB memory, full flag compatibility
- Defined out-of-scope: remote search, IDE integrations, Windows support, transparent fallback
- Recorded operator preferences: Rust, Nix flake, fff-search crate dependency, JSON-RPC over Unix socket
