# ADR-001: Rust 2024 Edition

## Status

Accepted

## Context

ff is a new Rust project starting in 2026. The Rust 2024 edition was stabilized in Rust 1.85.0 (February 2025) and has been the recommended edition for new projects since then. Current stable Rust is 1.96.0 (June 2026).

Key 2024 edition improvements relevant to ff:
- Async closures (stabilized in 1.85)
- `let` chains in patterns
- Improved `gen` blocks (preparing for future stabilization)
- Never type (`!`) improvements
- Enhanced safety guarantees
- Better ergonomics for async code

## Decision

Use **Rust 2024 edition** with MSRV (Minimum Supported Rust Version) of **1.96**.

All crates in the workspace will use `edition = "2024"` in their `Cargo.toml`.

## Consequences

### Positive

- Access to modern language features (async closures, let chains)
- Better async ergonomics for daemon IPC
- Improved safety guarantees
- Future-proof: 2024 edition will receive ongoing support
- Aligns with current Rust ecosystem best practices

### Negative

- Requires Rust 1.96+ (excludes users on older toolchains)
- Some dependencies may not yet support 2024 edition (rare, most do)
- Migration cost if we later want to support 2021 edition (not planned)

### Risks

- **Dependency compatibility:** All key dependencies (tokio, clap, regex, notify, serde) support 2024 edition as of 2026
- **Tooling:** rust-analyzer and clippy fully support 2024 edition
- **Documentation:** Most Rust documentation now defaults to 2024 edition examples

## Migration Path

Not applicable (greenfield project). If we later need to support 2021 edition, we would need to:
1. Remove 2024-specific features (async closures, let chains)
2. Run `cargo fix --edition 2021`
3. Test thoroughly

This is not planned and would only occur if a critical dependency drops 2024 support (unlikely).
