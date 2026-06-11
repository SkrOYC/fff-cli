# Epic G: CLI & Output

**Status:** Active  
**Scope:** ff-cli main dispatch, daemon client, one-shot mode, shell completions  
**Dependencies:** Epics C, D, F  
**Story Points:** 15

## Overview

Complete the ff-cli binary with main dispatch logic, daemon client for all subcommands, one-shot mode, and shell completions. This epic ties together all query engines into a unified CLI experience.

---

#### CLIO-G001 ff-cli: Main Dispatch and Daemon Client
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** CNTS-C004, NMSR-D004, META-F003
- **Description:** Implement the main dispatch logic in ff-cli. Parse global flags (--oneshot, --no-color, --json, --null, --quiet, --max-results, --timeout). Determine subcommand (grep, search, find, daemon, completions). For query subcommands: connect to daemon socket (or create one-shot index), send request, receive streamed results, format output, determine exit code. Handle daemon auto-start on connection failure.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the CLI binary
When running `ff grep TODO`
Then it dispatches to the grep subcommand handler

Given the CLI binary
When running `ff search "\.rs$"`
Then it dispatches to the search subcommand handler

Given the CLI binary
When running `ff find . -name "*.rs"`
Then it dispatches to the find subcommand handler

Given the CLI with --json global flag
When running `ff --json grep TODO`
Then all output is in JSON format

Given the CLI with --quiet global flag
When running `ff --quiet grep TODO`
Then no output is printed
And exit code is 0 if matches found, 1 if no matches
```

---

#### CLIO-G002 ff-cli: One-Shot Mode
- **Type:** Feature
- **Effort:** 3
- **Dependencies:** CLIO-G001, FOUND-A004
- **Description:** Implement one-shot mode (--oneshot flag). When set, the CLI creates a temporary Index (scans file tree), executes the query directly against the index (no daemon), formats output, and exits. The temporary index is dropped after the query completes. One-shot mode must work for all three subcommands (grep, search, find).
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the CLI with --oneshot flag
When running `ff --oneshot grep TODO`
Then it creates a temporary index
And executes the grep query without connecting to a daemon
And prints formatted results
And exits, freeing all index memory

Given the CLI with --oneshot flag
When running `ff --oneshot find . -name "*.rs" -exec wc -l {} +`
Then it creates a temporary index
And executes the find query with action execution
And prints the command output

Given no daemon is running
When running `ff --oneshot grep TODO`
Then it succeeds without attempting to start a daemon
```

---

#### CLIO-G003 ff-cli: Daemon Subcommand
- **Type:** Feature
- **Effort:** 3
- **Dependencies:** CLIO-G001, IPCD-B003
- **Description:** Implement the daemon subcommand for explicit lifecycle management: `ff daemon start` (start daemon for cwd), `ff daemon stop` (stop daemon), `ff daemon restart` (restart daemon), `ff daemon status` (show daemon status: running/stopped, PID, socket path, indexed files, uptime), `ff daemon logs` (tail daemon log file).
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the CLI with daemon subcommand
When running `ff daemon start`
Then the daemon is started for the current directory
And the socket and PID files are created

Given a running daemon
When running `ff daemon status`
Then it prints: running, PID, socket path, indexed file count, uptime

Given a running daemon
When running `ff daemon stop`
Then the daemon exits gracefully
And the socket and PID files are removed

Given no running daemon
When running `ff daemon status`
Then it prints: stopped
```

---

#### CLIO-G004 Shell Completions
- **Type:** Feature
- **Effort:** 2
- **Dependencies:** CLIO-G001
- **Description:** Generate shell completion scripts for bash, zsh, and fish using clap_complete. Implement `ff completions <shell>` subcommand that prints the completion script to stdout. Document installation instructions for each shell.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the CLI with completions subcommand
When running `ff completions bash`
Then it prints a valid bash completion script to stdout

Given the CLI with completions subcommand
When running `ff completions zsh`
Then it prints a valid zsh completion script to stdout

Given the CLI with completions subcommand
When running `ff completions fish`
Then it prints a valid fish completion script to stdout

Given bash completions are installed
When typing `ff gr<TAB>`
Then it completes to `ff grep`
And when typing `ff grep --<TAB>`
Then it shows all available grep flags
```

---

#### CLIO-G005 CLI Integration Tests
- **Type:** Feature
- **Effort:** 2
- **Dependencies:** CLIO-G001, CLIO-G002, CLIO-G003
- **Description:** Write end-to-end integration tests for the CLI: subcommand dispatch, global flags, one-shot mode, daemon subcommand, exit codes. Use assert_cmd for CLI testing. Test all three subcommands with various flag combinations. Verify exit code parity with rg/fd/find.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the CLI integration test suite
When running `cargo test --test cli`
Then all tests pass with zero failures

Given a test for exit codes
When running ff grep with a matching pattern
Then exit code is 0
And when running ff grep with a non-matching pattern
Then exit code is 1
And when running ff grep with an invalid pattern
Then exit code is 2

Given a test for one-shot mode
When running `ff --oneshot grep TODO`
Then it succeeds without a daemon running
And produces correct output
```
