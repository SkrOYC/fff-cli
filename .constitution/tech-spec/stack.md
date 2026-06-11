# Technical Stack

**Version:** v0.1.0

## Bill of Materials

### Core Runtime

| Component | Version | Purpose |
|-----------|---------|---------|
| Rust | 1.96+ | Language and toolchain |
| Edition | 2024 | Language edition with async closures, let chains, improved safety |
| Cargo | 1.96+ | Build system and package manager |

### Async Runtime

| Component | Version | Purpose |
|-----------|---------|---------|
| tokio | 1.40+ | Async runtime for daemon and IPC |
| tokio-util | 0.7+ | Utilities for tokio (codecs, cancellation) |

### CLI & Argument Parsing

| Component | Version | Purpose |
|-----------|---------|---------|
| clap | 4.6+ | CLI argument parsing with derive macros |
| clap_complete | 4.5+ | Shell completion generation |

### Regex & Pattern Matching

| Component | Version | Purpose |
|-----------|---------|---------|
| regex | 1.12+ | Regular expression matching (content search) |
| globset | 0.4+ | Glob pattern matching (name search, file filtering) |
| ignore | 0.4+ | Gitignore-aware directory walking |

### Filesystem Watching

| Component | Version | Purpose |
|-----------|---------|---------|
| notify | 8.2+ | Cross-platform filesystem notifications (inotify, FSEvents) |

### Data Structures

| Component | Version | Purpose |
|-----------|---------|---------|
| hashlink | 0.11+ | LRU cache for content index |
| hashbrown | 0.15+ | Fast HashMap implementation (via hashlink) |

### Serialization & IPC

| Component | Version | Purpose |
|-----------|---------|---------|
| serde | 1.0+ | Serialization framework |
| serde_json | 1.0+ | JSON serialization for IPC |
| bytes | 1.7+ | Efficient byte buffer management |

### Logging & Diagnostics

| Component | Version | Purpose |
|-----------|---------|---------|
| tracing | 0.1+ | Structured logging and diagnostics |
| tracing-subscriber | 0.3+ | Log formatting and filtering |

### Error Handling

| Component | Version | Purpose |
|-----------|---------|---------|
| anyhow | 1.0+ | Application-level error handling |
| thiserror | 1.0+ | Library-level error type derivation |

### Testing

| Component | Version | Purpose |
|-----------|---------|---------|
| proptest | 1.5+ | Property-based testing |
| assert_cmd | 2.0+ | CLI integration testing |
| predicates | 3.1+ | Assertion predicates for tests |
| tempfile | 3.13+ | Temporary file/directory creation for tests |

### Development Tools

| Component | Version | Purpose |
|-----------|---------|---------|
| cargo-nextest | 0.9+ | Fast test runner |
| cargo-watch | 8.4+ | Development workflow automation |
| rust-analyzer | Latest | IDE support |

## Compatibility Policy

### Dependency Pinning

- **Lockfile:** Commit `Cargo.lock` to version control (application workspace)
- **Version ranges:** Use caret requirements (`^1.40`) for dependencies
- **Workspace dependencies:** Define shared dependencies in workspace root `Cargo.toml`
- **MSRV:** Minimum Supported Rust Version is 1.96 (matches 2024 edition requirements)

### Upgrade Policy

- **Patch updates:** Apply automatically via `cargo update`
- **Minor updates:** Review changelog, apply if no breaking changes
- **Major updates:** Evaluate impact, test thoroughly before upgrading
- **Security updates:** Apply immediately, even if major version bump required

### Dependency Review

- Run `cargo audit` in CI pipeline
- Review new dependencies for license compatibility (MIT/Apache-2.0 preferred)
- Minimize dependency count to reduce compile times and attack surface
