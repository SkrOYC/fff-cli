# Functional Capabilities

## Epic 1: Content Search (replaces rg)

**P0 — C-01: Regex content search**
The system must search indexed file contents using regular expressions and return matching lines with file path, line number, and column information.

**P0 — C-02: Case sensitivity control**
The system must support smart case (case-insensitive unless pattern contains uppercase), forced case-sensitive, and forced case-insensitive modes.

**P0 — C-03: Context lines**
The system must return configurable lines of context before and/or after each match.

**P0 — C-04: File filtering**
The system must filter content search scope by glob patterns, file type, extension, and respect for ignore files (.gitignore, .ignore, etc.).

**P0 — C-05: Hidden file control**
The system must allow searching hidden files/directories (dotfiles) via explicit flag, defaulting to respecting ignore conventions.

**P0 — C-06: Output format variants**
The system must support: default colored output, files-with-matches only (`-l`), match count per file (`-c`), JSON output, NUL-delimited output, and column display.

**P1 — C-07: Match replacement**
The system must support substituting matched text with a replacement pattern in output (`-r`).

**P1 — C-08: Only-matching mode**
The system must support outputting only the matched portion of each line (`-o`).

**P1 — C-09: Max match count**
The system must support limiting the number of matches per file and overall.

**P1 — C-10: Multiline matching**
The system must support matching patterns that span multiple lines (`-U`).

**P2 — C-11: PCRE2 pattern support**
The system must support Perl-compatible regular expressions with lookaround assertions, or clearly error with the equivalent `rg --pcre2` command.

## Epic 2: Name Search (replaces fd)

**P0 — N-01: Regex name search**
The system must search indexed file paths/names using regular expressions and return matching paths.

**P0 — N-02: Glob name search**
The system must support glob-pattern matching against file names as an alternative to regex.

**P0 — N-03: Fixed-string name search**
The system must support literal string matching against file names.

**P0 — N-04: Extension filter**
The system must filter results by file extension.

**P0 — N-05: File type filter**
The system must filter results by type: regular file, directory, symlink, socket, executable, empty.

**P0 — N-06: Depth control**
The system must support maximum and minimum depth constraints on search results.

**P0 — N-07: Exclude patterns**
The system must support excluding paths matching glob patterns from results.

**P0 — N-08: Hidden and ignored file control**
The system must support including hidden files and overriding ignore files via explicit flags.

**P1 — N-09: Absolute path output**
The system must support outputting absolute paths instead of relative paths.

**P1 — N-10: Detailed listing**
The system must support an ls-like detailed output mode showing size, permissions, and modification time.

**P1 — N-11: Format templates**
The system must support user-defined output format strings with placeholders for path components.

**P1 — N-12: Time-based filtering**
The system must filter results by modification time (changed within/before a duration or date).

**P1 — N-13: Size-based filtering**
The system must filter results by file size with comparison operators and unit suffixes.

**P1 — N-14: Command execution per result**
The system must execute a user-specified command for each result, substituting the file path.

**P1 — N-15: Batch command execution**
The system must execute a user-sific command once with all result paths as arguments.

**P1 — N-16: Exit after first match**
The system must support exiting immediately after finding the first match.

**P1 — N-17: NUL-delimited output**
The system must support NUL byte as output delimiter for safe piping of paths with special characters.

## Epic 3: Metadata Query (replaces find)

**P0 — M-01: Name predicate**
The system must support matching file paths against glob patterns.

**P0 — M-02: Type predicate**
The system must support filtering by file type (file, directory, symlink, socket, block, character, pipe).

**P0 — M-03: Size predicate**
The system must support filtering by file size with comparison operators (+/- prefix) and unit suffixes (c, k, M, G).

**P0 — M-04: Time predicates**
The system must support filtering by modification time, access time, and status change time with comparison operators and day/minute granularity.

**P0 — M-05: Boolean expression parser**
The system must parse and evaluate boolean expressions combining predicates with AND (implicit), OR (`-o`), NOT (`!`), and parenthetical grouping.

**P0 — M-06: Print action**
The system must support printing matching paths (`-print`, `-print0`) as the default action.

**P0 — M-07: Exec action**
The system must support executing a command with matched paths substituted (`-exec {} +`, `-exec {} ;`, `-execdir`).

**P0 — M-08: Delete action**
The system must support deleting matched files/directories (`-delete`).

**P0 — M-09: Depth control**
The system must support maximum and minimum depth constraints.

**P1 — M-10: Permission predicate**
The system must support filtering by file permission bits, including symbolic mode specifications (`/6000`, `-g+w`, `u=rwx`).

**P1 — M-11: Ownership predicates**
The system must support filtering by user name/ID and group name/ID.

**P1 — M-12: Inode predicate**
The system must support filtering by inode number.

**P1 — M-13: Link count predicate**
The system must support filtering by hard link count.

**P1 — M-14: Empty predicate**
The system must support matching empty files and directories.

**P1 — M-15: Newer predicate**
The system must support comparing modification/access/change time against a reference file.

**P1 — M-16: Same-file predicate**
The system must support matching files on the same device and inode as a reference path.

**P1 — M-17: Printf format action**
The system must support formatted output with find-style format specifiers (`-printf`).

**P1 — M-18: Ls format action**
The system must support ls-like formatted output (`-ls`, `-fls`).

**P1 — M-19: Prune support**
The system must support pruning directory branches from traversal.

**P1 — M-20: Mount/filesystem boundary**
The system must support restricting search to a single filesystem (`-mount`, `-xdev`).

**P1 — M-21: Symlink following**
The system must support following symbolic links (`-L`, `-follow`).

**P2 — M-22: Ok/prompt action**
The system must support prompting the user for confirmation before executing an action (`-ok`, `-okdir`).

## Epic 4: Daemon & Indexing

**P0 — D-01: Per-root daemon**
The system must run one daemon process per indexed root directory, identified by a hash of the root path.

**P0 — D-02: Full tree scan on startup**
The system must scan the entire file tree on daemon startup, collecting file metadata and optionally file content.

**P0 — D-03: Filesystem watching**
The system must watch the indexed tree for filesystem changes and update the index incrementally.

**P0 — D-04: Auto-start**
The system must automatically start the daemon on the first query if it is not already running.

**P0 — D-05: Idle timeout**
The system must automatically exit the daemon after a configurable period of inactivity.

**P0 — D-06: Socket-based IPC**
The system must communicate between CLI and daemon via JSON-RPC 2.0 over Unix domain sockets.

**P1 — D-07: Systemd user unit**
The system must provide a systemd user service unit for explicit daemon lifecycle management.

**P1 — D-08: Daemon lifecycle commands**
The system must support explicit daemon management subcommands: start, stop, restart, status.

**P1 — D-09: Content index budget**
The system must support a configurable memory budget for the content index, evicting least-recently-used content when exceeded.

**P1 — D-10: Ignore file awareness**
The system must respect .gitignore, .ignore, and similar ignore files during scanning, with configurable override.

**P1 — D-11: Hidden file scanning**
The system must support scanning hidden files/directories, with the daemon maintaining appropriate index state.

## Epic 5: CLI Interface

**P0 — I-01: Subcommand dispatch**
The system must dispatch `ff grep`, `ff search`, and `ff find` to the appropriate query engine.

**P0 — I-02: Daemon client**
The system must connect to the daemon socket, send queries, receive results, and format output.

**P0 — I-03: One-shot mode**
The system must support a mode where the CLI creates a temporary index, runs a single query, and exits without requiring a daemon.

**P0 — I-04: Exit codes**
The system must return exit code 0 on match found, 1 on no match, and 2 on error — consistent with rg/fd/find conventions.

**P0 — I-05: Daemon auto-start from CLI**
The system must detect a missing daemon and start it transparently before dispatching the query.

**P1 — I-06: Quiet mode**
The system must support suppressing output and relying solely on exit codes (`-q`).

**P1 — I-07: Version and help**
The system must display version information and comprehensive help text for each subcommand.

## Epic 6: Output Formatting

**P0 — F-01: Colored terminal output**
The system must produce ANSI-colored output when writing to a terminal, with match highlighting and path coloring.

**P0 — F-02: Plain output for pipes**
The system must automatically disable colors and formatting when output is not a terminal.

**P0 — F-03: JSON output**
The system must support structured JSON output for programmatic consumption.

**P0 — F-04: NUL-delimited output**
The system must support NUL byte-delimited output for safe handling of paths with special characters.

**P1 — F-05: Count output**
The system must support showing only match counts per file.

**P1 — F-06: Files-with-matches output**
The system must support showing only file paths that contain at least one match.
