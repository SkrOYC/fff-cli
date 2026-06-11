# Project Guidelines

**Version:** v0.1.0

## Repository Structure

```
fff-cli/
├── Cargo.toml                    # Workspace root (virtual manifest)
├── Cargo.lock                    # Locked dependency versions
├── rust-toolchain.toml           # Pin Rust version (1.96+)
├── .cargo/
│   └── config.toml               # Cargo configuration (aliases, target settings)
├── crates/
│   ├── ff-cli/                   # CLI binary crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # Entry point, subcommand dispatch
│   │       ├── args.rs           # clap derive definitions
│   │       ├── client.rs         # Daemon socket client
│   │       ├── oneshot.rs        # One-shot mode (no daemon)
│   │       └── output.rs         # Output formatting (delegates to ff-formatter)
│   │
│   ├── ff-daemon/                # Daemon binary crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # Daemon entry point
│   │       ├── lifecycle.rs      # PID file, socket, idle timeout, systemd
│   │       ├── server.rs         # Socket listener, JSON-RPC dispatch
│   │       └── autostart.rs      # Auto-start detection and spawning
│   │
│   ├── ff-index/                 # Index library crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Public API
│   │       ├── scanner.rs        # File tree scanning (uses ignore crate)
│   │       ├── entry.rs          # FileEntry struct definition
│   │       ├── store.rs          # HashMap + Vec index storage
│   │       ├── content.rs        # Content cache (hashlink LruCache)
│   │       └── rebuild.rs        # Full rebuild orchestration
│   │
│   ├── ff-watcher/               # Filesystem watcher library crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Public API
│   │       ├── watcher.rs        # notify wrapper with debouncing
│   │       └── coalesce.rs       # Event coalescing logic
│   │
│   ├── ff-query/                 # Query engine library crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Public API, query dispatch
│   │       ├── content.rs        # Content search engine (grep)
│   │       ├── name.rs           # Name search engine (search)
│   │       ├── metadata.rs       # Metadata query engine (find)
│   │       ├── expr.rs           # Boolean expression parser
│   │       ├── predicate.rs      # Predicate types and evaluation
│   │       └── action.rs         # Action executor (exec, delete, print)
│   │
│   ├── ff-ipc/                   # IPC protocol library crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Public API
│   │       ├── protocol.rs       # JSON-RPC 2.0 message types
│   │       ├── codec.rs          # Length-prefixed codec (tokio_util::codec)
│   │       ├── transport.rs      # Unix socket transport
│   │       └── messages.rs       # Request/response message definitions
│   │
│   └── ff-common/                # Shared types and utilities
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs            # Public API
│           ├── types.rs          # Shared enums (FileType, CaseMode, etc.)
│           ├── config.rs         # Configuration loading (TOML)
│           ├── paths.rs          # Socket path, PID path, log path computation
│           └── errors.rs         # Common error types
│
├── tests/                        # Integration tests
│   ├── cli/                      # CLI integration tests
│   │   ├── grep.rs
│   │   ├── search.rs
│   │   └── find.rs
│   ├── daemon/                   # Daemon lifecycle tests
│   │   ├── autostart.rs
│   │   ├── idle_timeout.rs
│   │   └── crash_recovery.rs
│   ├── compatibility/            # rg/fd/find compatibility tests
│   │   ├── golden/               # Golden file test fixtures
│   │   │   ├── grep/
│   │   │   ├── search/
│   │   │   └── find/
│   │   └── runner.rs             # Golden file comparison runner
│   └── fixtures/                 # Test fixture files
│       ├── sample_tree/          # Sample directory tree for testing
│       └── patterns/             # Test patterns and expected outputs
│
├── docs/                         # Documentation
│   ├── man/                      # Man page source (roff)
│   ├── completions/              # Shell completion scripts
│   └── architecture/             # Architecture decision records
│
├── scripts/                      # Development scripts
│   ├── bench.sh                  # Benchmark runner
│   └── golden-gen.sh             # Golden file generator
│
├── flake.nix                     # Nix flake for build and dev shell
├── flake.lock                    # Nix flake lock file
├── .gitignore
├── LICENSE-MIT
├── LICENSE-APACHE
└── README.md
```

## Crate Dependency Graph

```
ff-cli ──────────┬──▶ ff-ipc
                 ├──▶ ff-query ──▶ ff-index ──▶ ff-watcher
                 ├──▶ ff-common
                 └──▶ ff-formatter (module within ff-cli)

ff-daemon ───────┬──▶ ff-ipc
                 ├──▶ ff-query ──▶ ff-index ──▶ ff-watcher
                 ├──▶ ff-common
                 └──▶ ff-formatter (module within ff-daemon)
```

**Dependency direction:** All dependencies point inward. ff-common and ff-index have no internal dependents beyond what's shown. ff-query depends on ff-index. ff-cli and ff-daemon are leaf binaries.

## Coding Standards

### Formatting

- **Formatter:** `rustfmt` (stable channel)
- **Configuration:** Use default rustfmt settings
- **Enforcement:** CI runs `cargo fmt --check`

### Linting

- **Linter:** `clippy` (stable channel)
- **Configuration:** Deny all warnings in CI (`-D warnings`)
- **Additional lints:** Enable `clippy::pedantic` where it doesn't conflict with readability
- **Enforcement:** CI runs `cargo clippy --all-targets -- -D warnings`

### Documentation

- **Public items:** All `pub` items must have doc comments (`///`)
- **Examples:** Public functions should have `# Examples` section in doc comment
- **Module docs:** Each `lib.rs` and `main.rs` must have module-level doc comment (`//!`)
- **Enforcement:** CI runs `cargo doc --no-deps --document-private-items`

### Testing

- **Unit tests:** Co-located with source code in `#[cfg(test)] mod tests`
- **Integration tests:** Under `tests/` directory
- **Property tests:** Use `proptest` for complex invariants
- **Golden tests:** Compare output against rg/fd/find on identical inputs
- **Coverage:** Target 80% line coverage (measured with `cargo-tarpaulin` or `cargo-llvm-cov`)
- **Runner:** Use `cargo nextest` for faster test execution

### Error Handling

- **Library crates:** Define error types with `thiserror`
- **Binary crates:** Use `anyhow` for error propagation
- **Error messages:** User-facing errors must be clear and actionable
- **Exit codes:** 0 = match found, 1 = no match, 2 = error (consistent with rg/fd/find)

### Naming Conventions

- **Crates:** `ff-*` prefix (e.g., `ff-index`, `ff-query`)
- **Modules:** `snake_case`
- **Types:** `PascalCase`
- **Functions:** `snake_case`
- **Constants:** `SCREAMING_SNAKE_CASE`
- **CLI subcommands:** `kebab-case` (e.g., `ff grep`, `ff search`, `ff find`)

### Git Conventions

- **Branch naming:** `feat/*`, `fix/*`, `refactor/*`, `docs/*`, `test/*`
- **Commit messages:** Conventional Commits format (`feat:`, `fix:`, `docs:`, etc.)
- **PR titles:** Match commit message format
- **Merge strategy:** Squash merge to keep history clean

## Build & Development

### Development Shell

```bash
# Enter development environment (Nix)
nix develop

# Or manually install dependencies
rustup install 1.96
rustup component add rustfmt clippy
cargo install cargo-nextest cargo-watch
```

### Common Commands

```bash
# Build all crates
cargo build --workspace

# Run all tests
cargo nextest run --workspace

# Run specific test suite
cargo nextest run -p ff-query

# Run benchmarks
cargo bench --workspace

# Check formatting
cargo fmt --check

# Run linter
cargo clippy --all-targets -- -D warnings

# Build release binary
cargo build --release --workspace

# Generate shell completions
cargo run --bin ff -- completions bash > completions/ff.bash
cargo run --bin ff -- completions zsh > completions/_ff
cargo run --bin ff -- completions fish > completions/ff.fish
```

### Benchmarking

```bash
# Run benchmarks against /nix/store (1.3M files)
./scripts/bench.sh

# Compare ff vs rg vs fd vs find
hyperfine \
  'ff grep "function" /nix/store' \
  'rg "function" /nix/store' \
  'fd -H "function" /nix/store' \
  'find /nix/store -name "*function*"'
```

## Release Process

### Versioning

- **SemVer:** Follow Semantic Versioning 2.0.0
- **Pre-1.0:** Breaking changes allowed in minor versions
- **Post-1.0:** Breaking changes only in major versions

### Release Checklist

1. Update version in `Cargo.toml` (workspace root)
2. Run `cargo update` to refresh lockfile
3. Run full test suite: `cargo nextest run --workspace`
4. Run linter: `cargo clippy --all-targets -- -D warnings`
5. Build release binary: `cargo build --release`
6. Test binary manually on sample repos
7. Update `CHANGELOG.md`
8. Commit version bump
9. Tag release: `git tag vX.Y.Z`
10. Push tag: `git push origin vX.Y.Z`
11. Create GitHub release with release notes
12. Upload binaries to release (Linux x86_64, Linux aarch64, macOS x86_64, macOS aarch64)

### Binary Distribution

- **Nix flake:** Primary distribution method
- **GitHub releases:** Pre-built binaries for common platforms
- **crates.io:** Publish library crates (ff-index, ff-query, ff-ipc, ff-common)
- **Do not publish:** Binary crates (ff-cli, ff-daemon) to crates.io
