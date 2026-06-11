# ADR-006: Golden File + Property-Based Testing

## Status

Accepted

## Context

The PRD requires full compatibility with rg, fd, and find. We need a testing strategy that:
- Verifies output matches rg/fd/find on identical inputs
- Covers edge cases (special characters, large files, binary content)
- Scales to thousands of test cases
- Catches regressions

Options considered:
1. **Golden file tests:** Compare output against stored expected outputs
2. **Property-based tests:** Generate random inputs and verify invariants
3. **Manual tests:** Human-driven testing
4. **Unit tests only:** Test individual components

## Decision

Use a **combination of golden file tests and property-based tests**:

### Golden File Tests

For each rg/fd/find command, store the expected output in a golden file:

```
tests/compatibility/golden/
├── grep/
│   ├── basic_pattern.txt
│   ├── case_insensitive.txt
│   ├── context_lines.txt
│   └── ...
├── search/
│   ├── regex_pattern.txt
│   ├── glob_pattern.txt
│   └── ...
└── find/
    ├── name_predicate.txt
    ├── type_predicate.txt
    └── ...
```

Test runner compares ff output against golden files:

```rust
#[test]
fn test_grep_basic_pattern() {
    let output = run_ff("grep", &["TODO", "tests/fixtures/sample_tree"]);
    let expected = fs::read_to_string("tests/compatibility/golden/grep/basic_pattern.txt")?;
    assert_eq!(output, expected);
}
```

### Property-Based Tests

Use `proptest` to generate random inputs and verify invariants:

```rust
proptest! {
    #[test]
    fn test_grep_exit_code_matches_rg(
        pattern in any::<String>(),
        content in any::<String>()
    ) {
        let ff_exit = run_ff_exit_code("grep", &[&pattern, &content]);
        let rg_exit = run_rg_exit_code(&[&pattern, &content]);
        prop_assert_eq!(ff_exit, rg_exit);
    }
}
```

### Integration Tests

Test daemon lifecycle, auto-start, crash recovery:

```rust
#[test]
fn test_daemon_auto_start() {
    let output = run_ff("grep", &["TODO", "."]);
    assert!(daemon_is_running());
    assert_eq!(output.exit_code(), 0);
}
```

## Consequences

### Positive

- **Comprehensive coverage:** Golden files verify exact output, property tests cover edge cases
- **Regression detection:** Golden files catch any output change
- **Scalability:** Can add thousands of golden file tests
- **Automation:** All tests run in CI
- **Documentation:** Golden files serve as examples of expected behavior

### Negative

- **Maintenance:** Golden files need updating when output format changes
- **Storage:** Golden files take disk space (mitigated by compression)
- **Generation:** Need tooling to generate golden files from rg/fd/find

### Trade-offs Accepted

- **Golden files vs. snapshot tests:** We choose explicit golden files over implicit snapshots
- **Property tests vs. exhaustive tests:** We choose random sampling over exhaustive enumeration
- **Integration tests vs. unit tests:** We test end-to-end behavior, not just components

## Test Generation

### Golden File Generator

```bash
#!/bin/bash
# scripts/golden-gen.sh

# Generate golden files from rg/fd/find
rg "TODO" tests/fixtures/sample_tree > tests/compatibility/golden/grep/basic_pattern.txt
fd "\.rs$" tests/fixtures/sample_tree > tests/compatibility/golden/search/regex_pattern.txt
find tests/fixtures/sample_tree -name "*.rs" > tests/compatibility/golden/find/name_predicate.txt
```

### Test Fixture Tree

```
tests/fixtures/sample_tree/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   └── utils.rs
├── tests/
│   └── integration.rs
├── README.md
├── .gitignore
└── Cargo.toml
```

## Test Categories

### 1. Compatibility Tests

Verify ff output matches rg/fd/find:

```rust
#[test]
fn test_grep_matches_rg() {
    let ff_output = run_ff("grep", &["TODO", "."]);
    let rg_output = run_rg(&["TODO", "."]);
    assert_eq!(ff_output.stdout, rg_output.stdout);
    assert_eq!(ff_output.exit_code, rg_output.exit_code);
}
```

### 2. Edge Case Tests

Test special characters, binary content, large files:

```rust
#[test]
fn test_grep_binary_content() {
    let output = run_ff("grep", &["TODO", "tests/fixtures/binary_file"]);
    assert_eq!(output.exit_code, 1); // No match in binary
}
```

### 3. Performance Tests

Verify query latency meets PRD constraints:

```rust
#[test]
fn test_grep_latency_1m_files() {
    let start = Instant::now();
    run_ff("grep", &["function", "/nix/store"]);
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 250);
}
```

### 4. Property-Based Tests

Generate random inputs and verify invariants:

```rust
proptest! {
    #[test]
    fn test_search_always_finds_existing_file(
        filename in "[a-z]{1,10}\\.rs"
    ) {
        // Create file with random name
        // Run ff search
        // Verify file is found
    }
}
```

## CI Integration

```yaml
# .github/workflows/test.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo nextest run --workspace
      - run: cargo test --test compatibility
      - run: cargo bench --no-run
```

## References

- [proptest crate](https://crates.io/crates/proptest)
- [assert_cmd crate](https://crates.io/crates/assert_cmd)
- [Golden file testing pattern](https://junit.org/junit5/docs/current/user-guide/#writing-tests-built-in-assertions)
