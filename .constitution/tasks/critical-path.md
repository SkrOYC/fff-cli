# Critical Path

**Version:** v0.1.0

## Active Backlog Summary

**Total Active Story Points:** 120  
**Total Active Tickets:** 34  
**Total Spikes:** 2  
**Total Epics:** 7

## Critical Path

The critical path runs through the metadata query chain, which has the deepest dependency chain:

1. **IPCD-B001** — Spike: JSON-RPC Streaming Protocol
2. **IPCD-B002** — ff-ipc: Protocol Types and Codec
3. **IPCD-B003** — ff-daemon: Lifecycle Management
4. **IPCD-B004** — ff-daemon: Socket Server
5. **METAP-E001** — Predicate Types and Evaluator
6. **METAP-E002** — Spike: Find Expression Parser
7. **METAP-E003** — Expression Parser
8. **METAP-E004** — Metadata Query Engine
9. **META-F001** — Action Executor
10. **META-F003** — find Subcommand
11. **CLIO-G001** — Main Dispatch and Daemon Client
12. **CLIO-G002** — One-Shot Mode
13. **PLSH-H003** — Comprehensive Compatibility Tests
14. **PLSH-H004** — Benchmark Suite and Release

**Critical path story points:** 84 of 120 total (70%)

## Build Order Diagram

```mermaid
flowchart LR
    subgraph "Epic A: Foundation ✅"
        A001[FOUND-A001]:::done
        A002[FOUND-A002]:::done
        A003[FOUND-A003]:::done
        A004[FOUND-A004]:::done
        A005[FOUND-A005]:::done
        A006[FOUND-A006]:::done
        A007[FOUND-A007]:::done
        A008[FOUND-A008]:::done
    end

    subgraph "Epic B: IPC & Daemon"
        B001[IPCD-B001]
        B002[IPCD-B002]
        B003[IPCD-B003]
        B004[IPCD-B004]
        B005[IPCD-B005]
        B006[IPCD-B006]
    end

    subgraph "Epic C: Content Search"
        C001[CNTS-C001]
        C002[CNTS-C002]
        C003[CNTS-C003]
        C004[CNTS-C004]
        C005[CNTS-C005]
    end

    subgraph "Epic D: Name Search"
        D001[NMSR-D001]
        D002[NMSR-D002]
        D003[NMSR-D003]
        D004[NMSR-D004]
        D005[NMSR-D005]
    end

    subgraph "Epic E: Metadata Predicates"
        E001[METAP-E001]
        E002[METAP-E002]
        E003[METAP-E003]
        E004[METAP-E004]
        E005[METAP-E005]
    end

    subgraph "Epic F: Metadata Actions"
        F001[META-F001]
        F002[META-F002]
        F003[META-F003]
        F004[META-F004]
    end

    subgraph "Epic G: CLI & Output"
        G001[CLIO-G001]
        G002[CLIO-G002]
        G003[CLIO-G003]
        G004[CLIO-G004]
        G005[CLIO-G005]
    end

    subgraph "Epic H: Polish"
        H001[PLSH-H001]
        H002[PLSH-H002]
        H003[PLSH-H003]
        H004[PLSH-H004]
    end

    A001 --> A003
    A001 --> A002
    A001 --> A006
    A003 --> A004
    A002 --> A005
    A004 --> A005
    A006 --> A007
    A004 --> A008
    A005 --> A008
    A007 --> A008

    A003 --> B002
    B001 --> B002
    B002 --> B003
    A004 --> B003
    A007 --> B003
    B003 --> B004
    B003 --> B005
    B004 --> B006
    B005 --> B006

    B002 --> C001
    A004 --> C001
    C001 --> C002
    C001 --> C003
    C003 --> C004
    B005 --> C004
    C004 --> C005

    B002 --> D001
    A004 --> D001
    D001 --> D002
    D001 --> D003
    D003 --> D004
    B005 --> D004
    D004 --> D005

    B002 --> E001
    A004 --> E001
    E001 --> E002
    E002 --> E003
    E003 --> E004
    E004 --> E005

    E004 --> F001
    E004 --> F002
    F001 --> F003
    F002 --> F003
    B005 --> F003
    F003 --> F004

    C004 --> G001
    D004 --> G001
    F003 --> G001
    G001 --> G002
    G001 --> G003
    G001 --> G004
    G001 --> G005

    G003 --> H001
    G001 --> H002
    C005 --> H003
    D005 --> H003
    F004 --> H003
    H003 --> H004

    classDef done fill:#90EE90,stroke:#333,stroke-width:2px
```

## Phasing Strategy

### v0.1.0 Scope (Active)

All P0 and P1 capabilities from the PRD (72 capabilities). This includes:
- Full content search (grep) with rg flag compatibility
- Full name search (search) with fd flag compatibility
- Full metadata query (find) with predicate and action support
- Daemon with auto-start, idle timeout, and systemd integration
- One-shot mode for scripts and CI
- All output formats (colored, JSON, NUL, plain)
- Shell completions (bash, zsh, fish)
- Comprehensive compatibility test suite

### Deferred to v0.2.0

P2 capabilities:
- C-11: PCRE2 pattern support (lookaround assertions)
- M-22: Ok/prompt action (-ok, -okdir)

### Future Consideration

- Persist metadata-only index to disk for faster daemon startup
- Incremental index updates instead of full rebuilds
- Query result caching
- Query cancellation propagation
