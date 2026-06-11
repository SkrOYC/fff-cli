# ADR-002: Flat Cargo Workspace Layout

## Status

Accepted

## Context

ff is a multi-crate Rust project with 7 library/binary crates. The architecture defines 9 logical containers (CLI, Daemon, Index, FilesystemWatcher, ContentQueryEngine, NameQueryEngine, MetadataQueryEngine, ActionExecutor, OutputFormatter).

Options considered:
1. **Flat `crates/` layout:** All crates at same level under `crates/`
2. **Grouped hierarchy:** Crates organized by function (e.g., `crates/core/`, `crates/daemon/`)
3. **Single crate with features:** One crate with feature flags for modularity

## Decision

Use a **flat `crates/` layout** with the following structure:

```
crates/
├── ff-cli/          # CLI binary
├── ff-daemon/       # Daemon binary
├── ff-index/        # Index library
├── ff-watcher/      # Filesystem watcher library
├── ff-query/        # Query engines library
├── ff-ipc/          # IPC protocol library
└── ff-common/       # Shared types and utilities
```

Each crate has a single, focused responsibility and one reason to change.

## Consequences

### Positive

- **Simplicity:** Flat structure is easy to navigate (`ls crates/` shows all crates)
- **Compilation:** Cargo can parallelize compilation of independent crates
- **Incremental builds:** Changes to one crate only rebuild dependents
- **Clear boundaries:** Each crate has a single responsibility
- **Naming consistency:** Crate names match directory names (no prefix stripping)
- **Scalability:** Easy to add new crates without deciding where they fit in hierarchy

### Negative

- **Namespace pollution:** All crates visible at same level (mitigated by `ff-*` prefix)
- **No logical grouping:** Related crates (e.g., ff-index, ff-watcher) not visually grouped

### Trade-offs Accepted

- **Flat vs. grouped:** We sacrifice logical grouping for simplicity and consistency
- **Crate count:** 7 crates may seem excessive, but each has clear boundaries
- **Dependency management:** Workspace dependencies in root `Cargo.toml` prevent version drift

## Alternatives Considered

### Grouped Hierarchy

```
crates/
├── core/
│   ├── ff-index/
│   ├── ff-watcher/
│   └── ff-query/
├── daemon/
│   └── ff-daemon/
└── cli/
    └── ff-cli/
```

**Rejected because:**
- Adds hierarchy complexity without clear benefit
- Harder to navigate (need to know which group a crate is in)
- No perfect hierarchy exists (where does ff-ipc fit?)
- matklad's recommendation: flat layout for 10k-1M LOC projects

### Single Crate with Features

```
ff/
├── Cargo.toml
└── src/
    ├── cli/
    ├── daemon/
    ├── index/
    └── ...
```

**Rejected because:**
- Slower compilation (no parallelization)
- Weaker encapsulation (all modules can access each other)
- Feature flag complexity grows with scale
- Harder to test components in isolation

## References

- [matklad: Large Rust Workspaces](https://matklad.github.io/2021/08/22/large-rust-workspaces.html)
- [Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
