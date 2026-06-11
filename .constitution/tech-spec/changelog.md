# Changelog — Stage 3 (TechSpec)

## v0.1.0 — Initial TechSpec

- Established technical stack: Rust 1.96+ / 2024 edition
- Defined Cargo workspace structure: flat `crates/` layout with 7 crates (ff-cli, ff-daemon, ff-index, ff-watcher, ff-query, ff-ipc, ff-common)
- Specified IPC protocol: length-prefixed JSON-RPC 2.0 over Unix sockets with tokio async runtime
- Defined index data structure: HashMap<PathBuf, usize> + Vec<FileEntry> for O(1) lookups and fast iteration
- Specified content cache: hashlink::LruCache with byte-based budget and LRU eviction
- Selected CLI parsing: clap 4.6+ with derive macros
- Defined testing strategy: golden file tests + property-based tests + integration tests
- Documented 6 ADRs:
  - ADR-001: Rust 2024 edition
  - ADR-002: Flat workspace layout
  - ADR-003: Length-prefixed JSON-RPC 2.0
  - ADR-004: HashMap + Vec index
  - ADR-005: hashlink LruCache
  - ADR-006: Golden file + property-based testing
- Defined data models: FileEntry, FileType, GitStatus, Index, ContentCache
- Specified IPC contract: JSON-RPC 2.0 methods (grep, search, find, ping, shutdown), error model, streaming protocol
- Specified CLI contract: subcommands (grep, search, find, daemon), flags, arguments, exit codes
- Established coding standards: rustfmt, clippy, documentation, testing, error handling, naming conventions
- Defined build and development workflow: Nix flake, cargo commands, benchmarking
- Specified release process: versioning, checklist, binary distribution
