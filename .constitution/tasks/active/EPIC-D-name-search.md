# Epic D: Name Search

**Status:** Active  
**Scope:** ff-query name engine, search output formatters, ff-cli search subcommand  
**Dependencies:** Epic B (IPC & Daemon)  
**Story Points:** 18

## Overview

Implement the name search engine (search subcommand) including regex/glob/fixed-string pattern matching, file filtering (extension, type, depth, exclude), and output formatting. This epic delivers the second end-to-end query path.

---

#### NMSR-D001 ff-query: Name Search Engine
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** IPCD-B002, FOUND-A004
- **Description:** Implement the NameQueryEngine in ff-query. Support three pattern modes: regex (compile and match against file name), glob (compile glob pattern and match), fixed-string (substring match). Apply file filters (extension, type, exclude, hidden, ignore, depth). Stream matching paths back to the caller.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given an index with files src/main.rs, src/lib.rs, README.md
When executing a name query with regex pattern "\.rs$"
Then it returns src/main.rs and src/lib.rs

Given a name query with glob pattern "*.rs"
When executing the query
Then it returns all .rs files at any depth

Given a name query with fixed-string pattern "main"
When executing the query
Then it returns all files with "main" in the name

Given a name query with max-depth 1
When executing the query
Then only files at depth 0 and 1 are returned
```

---

#### NMSR-D002 ff-query: Name Search Filters
- **Type:** Feature
- **Effort:** 3
- **Dependencies:** NMSR-D001
- **Description:** Implement advanced filters for name search: extension filter (-e, repeatable), file type filter (-t f/d/l/s/x/e), exclude patterns (-E, repeatable), size-based filtering (-S with comparison operators and units), and time-based filtering (--changed-within, --changed-before). Size and time filters use cached metadata from the Index.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given a name query with extension filter "rs" and "ts"
When executing the query
Then only .rs and .ts files are returned

Given a name query with type filter "f" (regular file)
When executing the query
Then directories and symlinks are excluded

Given a name query with size filter "+1M"
When executing the query
Then only files larger than 1MB are returned

Given a name query with --changed-within "1d"
When executing the query
Then only files modified in the last 24 hours are returned
```

---

#### NMSR-D003 Output Formatters: Name Search
- **Type:** Feature
- **Effort:** 3
- **Dependencies:** NMSR-D001
- **Description:** Implement output formatters for name search results: default colored output (path coloring by file type), plain output for pipes, JSON output (--json), NUL-delimited output (-0), absolute path output (-a), detailed listing (-l with size/permissions/mtime), and format templates (--format with placeholders). Detect TTY vs. pipe automatically.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given name search results written to a TTY
When formatting with default output
Then directories are colored in blue
And executables are colored in green
And regular files have no special coloring

Given name search results with -l flag
When formatting output
Then each line shows permissions, size, mtime, and path (ls-like format)

Given name search results with --format "{}/{.}" flag
When formatting output
Then each line shows directory/filename without extension

Given name search results with -a flag
When formatting output
Then paths are absolute instead of relative
```

---

#### NMSR-D004 ff-cli: search Subcommand
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** NMSR-D003, IPCD-B005
- **Description:** Implement the search subcommand in ff-cli using clap derive. Parse all fd-compatible flags (pattern, pattern mode, filters, depth, output formats, exec). Connect to daemon or use one-shot mode, send search request, receive streamed results, format output. Support command execution per result (-x) and batch execution (-X).
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the CLI with search subcommand
When running `ff search "\.rs$"`
Then it prints all .rs files
And exits with code 0 if matches found, 1 if no matches

Given the CLI with search subcommand and --glob flag
When running `ff search --glob "*.rs"`
Then it uses glob matching instead of regex

Given the CLI with search subcommand and -x flag
When running `ff search "\.rs$" -x wc -l`
Then it executes `wc -l` for each matching file
And prints the command output

Given the CLI with search subcommand and -1 flag
When running `ff search -1 "\.rs$"`
Then it exits after finding the first match
```

---

#### NMSR-D005 Name Search Golden File Tests
- **Type:** Feature
- **Effort:** 2
- **Dependencies:** NMSR-D004
- **Description:** Generate golden file tests comparing ff search output against fd output on identical inputs. Cover regex/glob/fixed-string patterns, extension filters, type filters, depth control, exclude patterns, output formats, and edge cases (special characters in filenames, symlinks). Store golden files under tests/compatibility/golden/search/.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the golden file test suite for search
When running `cargo test --test compatibility search`
Then all golden file comparisons pass
And ff output matches fd output for every test case

Given a test fixture directory with known content
When running ff search and fd with identical flags
Then both tools produce identical exit codes
And both tools produce identical output (modulo path prefixes)
```
