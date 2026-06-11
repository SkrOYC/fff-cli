# Epic F: Metadata Actions

**Status:** Active  
**Scope:** ff-query action executor, find output formatters, ff-cli find subcommand  
**Dependencies:** Epic E (Metadata Predicates)  
**Story Points:** 16

## Overview

Implement the action executor for metadata queries (print, exec, delete, printf, ls) and the find subcommand in ff-cli. This epic completes the third end-to-end query path and delivers full find compatibility.

---

#### META-F001 ff-query: Action Executor
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** METAP-E004
- **Description:** Implement the ActionExecutor in ff-query. Actions: Print (output paths with format variants: plain, -print0), Exec (spawn subprocess per match with -exec {} ;, batch with -exec {} +, or per-match-dir with -execdir {} ;), Delete (unlink files or recursively delete empty directories). Handle action errors gracefully (permission denied, command not found) and continue with remaining matches.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given a metadata query with action Print
When executing the query
Then matching paths are printed to stdout, one per line

Given a metadata query with action Exec "wc -l" in batch mode
When 10 files match
Then `wc -l` is executed once with all 10 paths as arguments
And the command output is printed to stdout

Given a metadata query with action Exec "wc -l" in per-match mode
When 10 files match
Then `wc -l` is executed 10 times, once per file
And each command output is printed to stdout

Given a metadata query with action Delete
When a matching file exists
Then the file is deleted
And when a matching directory is empty
Then the directory is deleted
And when a matching directory is not empty
Then an error is reported and the directory is not deleted
```

---

#### META-F002 Output Formatters: Metadata Query
- **Type:** Feature
- **Effort:** 3
- **Dependencies:** METAP-E004
- **Description:** Implement output formatters for metadata query results: printf format (-printf with find-style format specifiers like %p, %s, %m, %u, %g, %T+), ls format (-ls with ls -dils-like output), and NUL-delimited output (-print0). Parse format strings and evaluate specifiers against FileEntry metadata.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given a metadata query with -printf "%p %s\n"
When a 1234-byte file matches
Then it prints "src/main.rs 1234"

Given a metadata query with -printf "%m %u %g %p\n"
When a file with mode 0o755, uid 1000, gid 1000 matches
Then it prints "755 1000 1000 src/main.rs"

Given a metadata query with -ls
When a file matches
Then it prints ls -dils-like output with inode, blocks, permissions, links, owner, group, size, date, path

Given a metadata query with -print0
When files match
Then paths are printed separated by NUL bytes
```

---

#### META-F003 ff-cli: find Subcommand
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** META-F001, META-F002, IPCD-B005
- **Description:** Implement the find subcommand in ff-cli. Parse find-style arguments: paths, predicates, combinators, actions, and options (-maxdepth, -mindepth, -depth, -mount, -L). Detect first non-flag argument as path vs. expression start. Connect to daemon or use one-shot mode, send find request with expression and actions, receive streamed results, execute actions, and determine exit code.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the CLI with find subcommand
When running `ff find . -name "*.rs" -type f`
Then it prints all regular .rs files
And exits with code 0 if matches found, 1 if no matches

Given the CLI with find subcommand
When running `ff find . -name "*.rs" -exec wc -l {} +`
Then it executes `wc -l` once with all matching paths
And prints the command output

Given the CLI with find subcommand
When running `ff find . -name "*.log" -delete`
Then all matching .log files are deleted
And the command exits with code 0

Given the CLI with find subcommand and no action
When running `ff find . -name "*.rs"`
Then -print is used as the default action
And matching paths are printed
```

---

#### META-F004 Find Golden File Tests
- **Type:** Feature
- **Effort:** 3
- **Dependencies:** META-F003
- **Description:** Generate golden file tests comparing ff find output against GNU find output on identical inputs. Cover name/type/size/time predicates, boolean expressions, exec/delete actions, printf/ls formats, depth control, and edge cases (special characters, symlinks, empty directories). Store golden files under tests/compatibility/golden/find/.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the golden file test suite for find
When running `cargo test --test compatibility find`
Then all golden file comparisons pass
And ff output matches find output for every test case

Given a test fixture directory with known content
When running ff find and find with identical expressions
Then both tools produce identical exit codes
And both tools produce identical output (modulo path prefixes)

Given a test for -exec action
When running `ff find . -name "*.rs" -exec wc -l {} +`
And `find . -name "*.rs" -exec wc -l {} +`
Then both produce identical output
```
