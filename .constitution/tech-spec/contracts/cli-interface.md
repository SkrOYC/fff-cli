# CLI Interface

## Overview

ff is a unified CLI with three primary subcommands: `grep`, `search`, and `find`. Each subcommand replaces a specific tool (rg, fd, find respectively) and supports its full flag surface.

## Global Flags

```
ff [GLOBAL FLAGS] <SUBCOMMAND> [SUBCOMMAND FLAGS]
```

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--oneshot` | | Run without daemon (build temporary index) | false |
| `--no-color` | | Disable colored output | auto-detect TTY |
| `--json` | | Output results as JSON | false |
| `--null` | `-0` | Use NUL delimiter instead of newline | false |
| `--quiet` | `-q` | Suppress output, use exit code only | false |
| `--max-results` | | Maximum number of results | 10000 |
| `--timeout` | | Query timeout in seconds | 30 |
| `--version` | `-V` | Print version information | |
| `--help` | `-h` | Print help information | |

## Subcommand: grep

Replaces `rg` (ripgrep). Searches file contents using regular expressions.

```
ff grep [FLAGS] <PATTERN> [PATH...]
```

### Flags

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--ignore-case` | `-i` | Case-insensitive search | false |
| `--case-sensitive` | `-s` | Force case-sensitive | false |
| `--smart-case` | `-S` | Smart case (default) | true |
| `--context` | `-C` | Lines of context before and after | 0 |
| `--before-context` | `-B` | Lines of context before | 0 |
| `--after-context` | `-A` | Lines of context after | 0 |
| `--glob` | `-g` | Glob pattern filter (repeatable) | |
| `--type` | `-t` | File type filter (repeatable) | |
| `--type-not` | `-T` | File type exclusion (repeatable) | |
| `--extension` | | Extension filter (repeatable) | |
| `--hidden` | `-.` | Search hidden files | false |
| `--no-ignore` | | Don't respect ignore files | false |
| `--no-ignore-vcs` | | Don't respect .gitignore | false |
| `--files-with-matches` | `-l` | Only print file paths with matches | false |
| `--count` | `-c` | Only print match count per file | false |
| `--count-matches` | | Only print match count per file (including multiple per line) | false |
| `--only-matching` | `-o` | Only print matching part | false |
| `--replace` | `-r` | Replace matches in output | |
| `--multiline` | `-U` | Enable multiline matching | false |
| `--pcre2` | `-P` | Use PCRE2 regex (not supported, errors with suggestion) | |
| `--max-depth` | | Maximum directory depth | |
| `--min-depth` | | Minimum directory depth | 0 |
| `--max-count` | `-m` | Maximum matches per file | |
| `--column` | | Show column numbers | false |
| `--line-number` | `-n` | Show line numbers (default for TTY) | auto |
| `--no-line-number` | | Hide line numbers | |
| `--no-filename` | | Don't show filenames | false |
| `--with-filename` | `-H` | Show filenames (default for multiple files) | auto |
| `--null-data` | | Use NUL as line delimiter | false |
| `--fixed-strings` | `-F` | Treat pattern as literal string | false |

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `PATTERN` | Regular expression pattern | yes |
| `PATH` | Paths to search (default: cwd) | no |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | At least one match found |
| 1 | No matches found |
| 2 | Error (invalid pattern, I/O error, etc.) |

### Examples

```bash
# Search for TODO in current directory
ff grep TODO

# Case-insensitive search for function definitions
ff grep -i "def test_" src/

# Search with context
ff grep -C 3 "error" src/

# Search only Rust files
ff grep -g "*.rs" "fn main"

# Files with matches only
ff grep -l "TODO" src/

# Count matches per file
ff grep -c "TODO" src/

# JSON output
ff grep --json "TODO" src/
```

## Subcommand: search

Replaces `fd`. Searches file names using regex, glob, or fixed-string patterns.

```
ff search [FLAGS] <PATTERN> [PATH...]
```

### Flags

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--glob` | `-g` | Use glob pattern (instead of regex) | false |
| `--fixed-strings` | `-F` | Use literal string (instead of regex) | false |
| `--ignore-case` | `-i` | Case-insensitive search | false |
| `--extension` | `-e` | Extension filter (repeatable) | |
| `--type` | `-t` | File type filter: f, d, l, s, x, e (repeatable) | |
| `--exclude` | `-E` | Exclude pattern (repeatable) | |
| `--hidden` | `-H` | Include hidden files | false |
| `--no-ignore` | | Don't respect ignore files | false |
| `--no-ignore-vcs` | | Don't respect .gitignore | false |
| `--max-depth` | `-d` | Maximum directory depth | |
| `--min-depth` | | Minimum directory depth | 0 |
| `--absolute-path` | `-a` | Show absolute paths | false |
| `--list-details` | `-l` | Show detailed listing (ls-like) | false |
| `--format` | | Custom output format string | |
| `--changed-within` | | Filter by modification time (e.g., 1d, 1h) | |
| `--changed-before` | | Filter by modification time (before date) | |
| `--size` | `-S` | Filter by file size (e.g., +1M, -1k) | |
| `--exec` | `-x` | Execute command for each result | |
| `--batch-exec` | `-X` | Execute command once with all results | |
| `--max-results` | | Maximum number of results | 10000 |
| `--max-results-one` | `-1` | Exit after first match | false |

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `PATTERN` | Search pattern (regex, glob, or fixed string) | yes |
| `PATH` | Paths to search (default: cwd) | no |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | At least one match found |
| 1 | No matches found |
| 2 | Error |

### Examples

```bash
# Find all Rust files
ff search "\.rs$"

# Find files with glob pattern
ff search -g "*.rs"

# Find directories only
ff search -t d "src"

# Find files larger than 1MB
ff search -S +1M

# Find files modified in last day
ff search --changed-within 1d

# Execute command for each result
ff search "\.rs$" -x wc -l

# Find with absolute paths
ff search -a "\.rs$"
```

## Subcommand: find

Replaces `find`. Evaluates boolean expressions against file metadata.

```
ff find [PATH...] [EXPRESSION]
```

### Predicates

| Predicate | Description |
|-----------|-------------|
| `-name PATTERN` | File name matches glob pattern |
| `-path PATTERN` | Full path matches glob pattern |
| `-regex PATTERN` | Full path matches regex |
| `-type TYPE` | File type: f, d, l, s, b, c, p |
| `-size SPEC` | File size: +1M, -1k, 100c |
| `-mtime N` | Modification time: +7, -1, 0 |
| `-atime N` | Access time |
| `-ctime N` | Status change time |
| `-perm SPEC` | Permission bits: /6000, -g+w, u=rwx |
| `-user NAME` | Owner user name or ID |
| `-group NAME` | Owner group name or ID |
| `-inum N` | Inode number |
| `-links N` | Hard link count |
| `-empty` | Empty file or directory |
| `-newer FILE` | Newer than reference file |
| `-anewer FILE` | Access time newer than reference |
| `-cnewer FILE` | Status change time newer than reference |
| `-samefile FILE` | Same device and inode as reference |
| `-readable` | File is readable |
| `-writable` | File is writable |
| `-executable` | File is executable |

### Combinators

| Combinator | Description |
|------------|-------------|
| `EXPR -a EXPR` | AND (implicit between predicates) |
| `EXPR -o EXPR` | OR |
| `! EXPR` | NOT |
| `( EXPR )` | Grouping |

### Actions

| Action | Description |
|--------|-------------|
| `-print` | Print path (default action) |
| `-print0` | Print path with NUL delimiter |
| `-printf FORMAT` | Print with format string |
| `-ls` | Print ls-like detailed listing |
| `-exec CMD {} ;` | Execute command per match |
| `-exec CMD {} +` | Execute command with all matches |
| `-execdir CMD {} ;` | Execute command in file's directory |
| `-delete` | Delete file or empty directory |
| `-ok CMD {} ;` | Prompt before executing |

### Options

| Option | Description |
|--------|-------------|
| `-maxdepth N` | Maximum directory depth |
| `-mindepth N` | Minimum directory depth |
| `-depth` | Process directory contents before directory |
| `-mount`, `-xdev` | Don't descend into other filesystems |
| `-L`, `-follow` | Follow symbolic links |
| `-prune` | Don't descend into matched directory |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | At least one match found |
| 1 | No matches found |
| 2 | Error |

### Examples

```bash
# Find all Rust files
ff find . -name "*.rs"

# Find files larger than 1MB
ff find . -type f -size +1M

# Find files modified in last 7 days
ff find . -mtime -7

# Find and delete empty directories
ff find . -type d -empty -delete

# Find and execute command
ff find . -name "*.rs" -exec wc -l {} +

# Complex expression
ff find . \( -name "*.rs" -o -name "*.ts" \) -type f -size +1k

# Find by permissions
ff find . -perm /6000
```

## Subcommand: daemon

Manage the ff daemon lifecycle.

```
ff daemon [SUBCOMMAND]
```

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `start` | Start daemon for current directory |
| `stop` | Stop daemon for current directory |
| `restart` | Restart daemon for current directory |
| `status` | Show daemon status |
| `logs` | Tail daemon logs |

### Examples

```bash
# Start daemon
ff daemon start

# Check status
ff daemon status

# Stop daemon
ff daemon stop

# View logs
ff daemon logs
```

## Shell Completions

```bash
# Generate completions
ff completions bash > /etc/bash_completion.d/ff
ff completions zsh > ~/.zfunc/_ff
ff completions fish > ~/.config/fish/completions/ff.fish
```
