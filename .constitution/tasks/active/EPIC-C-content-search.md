# Epic C: Content Search

**Status:** Active  
**Scope:** ff-query content engine, grep output formatters, ff-cli grep subcommand  
**Dependencies:** Epic B (IPC & Daemon)  
**Story Points:** 18

## Overview

Implement the content search engine (grep subcommand) including regex matching with case sensitivity modes, context lines, file filtering, and output formatting. This epic delivers the first end-to-end query path: CLI → daemon → content engine → index → formatted output.

---

#### CNTS-C001 ff-query: Content Search Engine
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** IPCD-B002, FOUND-A004
- **Description:** Implement the ContentQueryEngine in ff-query. Compile regex patterns with case sensitivity modes (smart, sensitive, insensitive). Iterate over Index entries filtered by file type/extension/glob. For each file, retrieve content from Index, apply regex, extract match ranges and context lines. Stream MatchResult objects back to the caller.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given an index with files containing "TODO" on various lines
When executing a content query with pattern "TODO"
Then it returns all matching lines with correct path, line number, and column

Given a content query with case mode "smart" and pattern "todo"
When executing the query
Then it matches "TODO", "Todo", and "todo" (case-insensitive)

Given a content query with case mode "smart" and pattern "TODO"
When executing the query
Then it matches only "TODO" (case-sensitive, pattern has uppercase)

Given a content query with context lines before=2, after=2
When a match is found on line 10
Then the result includes lines 8-12 with correct line numbers
```

---

#### CNTS-C002 ff-query: Content File Filtering
- **Type:** Feature
- **Effort:** 3
- **Dependencies:** CNTS-C001
- **Description:** Implement file filtering for content search: glob patterns (-g), file type filters (-t), extension filters (--extension), hidden file control (--hidden), and ignore file awareness (--no-ignore, --no-ignore-vcs). Filters are applied before content scanning to avoid unnecessary work.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given a content query with glob filter "*.rs"
When executing the query
Then only files matching the glob are scanned

Given a content query with extension filter "rs"
When executing the query
Then only .rs files are scanned

Given a content query with hidden flag enabled
When executing the query
Then hidden files (dotfiles) are included in the search

Given a content query with no-ignore flag
When executing the query
Then .gitignore rules are not respected
And all files are scanned
```

---

#### CNTS-C003 Output Formatters: Content Search
- **Type:** Feature
- **Effort:** 3
- **Dependencies:** CNTS-C001
- **Description:** Implement output formatters for content search results: default colored output (match highlighting, path coloring), plain output for pipes (no colors), JSON output (--json), NUL-delimited output (--null), files-with-matches (-l), count per file (-c), column display (--column). Detect TTY vs. pipe for automatic color selection.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given content search results written to a TTY
When formatting with default output
Then matches are highlighted in red
And file paths are colored in magenta
And line numbers are colored in green

Given content search results written to a pipe
When formatting with default output
Then no ANSI color codes are present

Given content search results with --json flag
When formatting output
Then each result is a valid JSON object on its own line
And the final line is a JSON summary with totalMatched and elapsedMs

Given content search results with -l flag
When formatting output
Then only file paths are printed (one per line)
And each file appears at most once
```

---

#### CNTS-C004 ff-cli: grep Subcommand
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** CNTS-C003, IPCD-B005
- **Description:** Implement the grep subcommand in ff-cli using clap derive. Parse all rg-compatible flags (pattern, case modes, context, filters, output formats). Connect to daemon socket (or create one-shot index), send grep request, receive streamed results, format output, and determine exit code (0=match, 1=no match, 2=error).
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the CLI with grep subcommand
When running `ff grep TODO`
Then it connects to the daemon
And sends a grep request with pattern "TODO"
And prints formatted results
And exits with code 0 if matches found, 1 if no matches

Given the CLI with grep subcommand and --oneshot flag
When running `ff grep --oneshot TODO`
Then it creates a temporary index
And executes the query without a daemon
And exits after printing results

Given the CLI with grep subcommand and invalid regex
When running `ff grep "[invalid"`
Then it prints a clear error message
And exits with code 2
```

---

#### CNTS-C005 Content Search Golden File Tests
- **Type:** Feature
- **Effort:** 2
- **Dependencies:** CNTS-C004
- **Description:** Generate golden file tests comparing ff grep output against rg output on identical inputs. Cover basic patterns, case sensitivity modes, context lines, file filters, output formats, and edge cases (binary files, empty files, very long lines). Store golden files under tests/compatibility/golden/grep/.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the golden file test suite for grep
When running `cargo test --test compatibility grep`
Then all golden file comparisons pass
And ff output matches rg output for every test case

Given a test fixture directory with known content
When running ff grep and rg with identical flags
Then both tools produce identical exit codes
And both tools produce identical output (modulo path prefixes)
```
