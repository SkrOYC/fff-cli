# Epic H: Polish

**Status:** Active  
**Scope:** Systemd unit, documentation, comprehensive tests, benchmarks, release  
**Dependencies:** Epic G (CLI & Output)  
**Story Points:** 11

## Overview

Polish the product for v0.1.0 release: systemd user unit for daemon lifecycle, documentation (man pages, README), comprehensive compatibility test suite, benchmark suite, and release preparation.

---

#### PLSH-H001 Systemd User Unit
- **Type:** Feature
- **Effort:** 2
- **Dependencies:** CLIO-G003
- **Description:** Create a systemd user service unit for the ff daemon. The unit should start the daemon on login (optional), restart on crash, and stop on logout. Provide installation instructions. Support per-directory daemons via template units or environment variables.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the systemd user unit file
When installing with `systemctl --user enable ff-daemon.service`
Then the service is enabled and starts on login

Given a running daemon managed by systemd
When the daemon crashes
Then systemd restarts it automatically

Given the systemd user unit
When running `systemctl --user stop ff-daemon.service`
Then the daemon exits gracefully
And the socket and PID files are cleaned up
```

---

#### PLSH-H002 Documentation
- **Type:** Chore
- **Effort:** 3
- **Dependencies:** CLIO-G001
- **Description:** Write documentation: README.md with installation instructions, quick start, usage examples, and architecture overview. Generate man pages for ff(1), ff-grep(1), ff-search(1), ff-find(1), ff-daemon(1) using clap_mangen. Write a CONTRIBUTING.md with development setup, coding standards, and PR process.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the repository
When reading README.md
Then it contains installation instructions for Nix and cargo
And it contains a quick start guide with examples
And it contains an architecture overview

Given the man page source files
When running `man ./docs/man/ff.1`
Then the man page renders correctly
And it documents all subcommands and flags

Given the CONTRIBUTING.md file
When reading it
Then it contains development setup instructions
And it documents coding standards
And it describes the PR process
```

---

#### PLSH-H003 Comprehensive Compatibility Test Suite
- **Type:** Feature
- **Effort:** 3
- **Dependencies:** CNTS-C005, NMSR-D005, META-F004
- **Description:** Consolidate all golden file tests into a comprehensive compatibility test suite. Add test cases for edge cases not covered by individual epics: very large files, binary content, special characters in filenames, Unicode content, permission edge cases, symlink cycles. Generate test fixtures programmatically. Measure compatibility percentage against rg/fd/find.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the comprehensive compatibility test suite
When running `cargo test --test compatibility`
Then all golden file comparisons pass
And the test suite covers >95% of commonly-used rg/fd/find flags

Given a test for special characters in filenames
When filenames contain spaces, quotes, and Unicode
Then ff produces identical output to rg/fd/find

Given a test for binary content
When searching binary files
Then ff handles them identically to rg (skip with warning)
```

---

#### PLSH-H004 Benchmark Suite and Release Preparation
- **Type:** Chore
- **Effort:** 3
- **Dependencies:** PLSH-H003
- **Description:** Create a benchmark suite comparing ff vs rg/fd/find on standard workloads (small repo, large repo, /nix/store). Document benchmark results in docs/benchmarks.md. Prepare for v0.1.0 release: update version in Cargo.toml, write release notes, build release binaries for Linux (x86_64, aarch64) and macOS (x86_64, aarch64), create GitHub release with binaries.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the benchmark suite
When running `./scripts/bench.sh`
Then it compares ff vs rg/fd/find on standard workloads
And outputs results in a comparable format

Given the release binaries
When building with `cargo build --release`
Then binaries are produced for all target platforms
And each binary passes the compatibility test suite

Given the v0.1.0 release
When creating the GitHub release
Then release notes document all implemented capabilities
And binaries are attached for download
```
