# Non-Functional Constraints

## Performance

- **Hot query latency:** Content search, name search, and metadata queries must return results in under 250ms on indexed trees containing 1 million or more files, measured from CLI dispatch to output completion.
- **Cold query latency:** The first query after daemon startup (while index is still being built) must return partial or full results within 5 seconds on trees with 1 million files.
- **Index build time:** Full tree scan and index construction must complete within 15 seconds for 1 million files on typical developer hardware (SSD, 4-core CPU).
- **Memory ceiling:** The daemon must not exceed 500MB resident memory for an indexed tree of 1 million files with content indexing enabled. Without content indexing, the ceiling is 150MB.
- **Filesystem watcher overhead:** The filesystem watcher must not consume more than 5% CPU during steady-state idle periods on trees with 1 million files.

## Compatibility

- **Flag coverage:** ff must support 100% of the commonly-used flags for rg (content search), fd (name search), and find (metadata query). "Commonly-used" is defined as flags documented in the respective tool's man page that are not platform-specific or deprecated.
- **Exit code parity:** ff must return exit codes identical to the tool it is replacing for each subcommand (0 = match found, 1 = no match, 2 = error).
- **Output parity:** Default output format for each subcommand must be visually and structurally identical to the corresponding tool's default output, including color codes, path separators, and line formatting.
- **Error messages:** When a flag or feature is not supported, ff must output an error message that includes the equivalent command using the original tool (rg, fd, or find).

## Reliability

- **Daemon crash recovery:** If the daemon process crashes, the CLI must detect the stale socket/PID file and automatically restart the daemon on the next query.
- **Index consistency:** After a filesystem event, the index must reflect the current state of the tree within 2 seconds. Queries during this window may return stale results but must not error.
- **Graceful degradation under memory pressure:** If system memory becomes critically low, the daemon must shed load by dropping the content index (retaining metadata-only) rather than crashing.

## Platform

- **Supported platforms:** Linux (x86_64, aarch64) and macOS (x86_64, aarch64) only.
- **Minimum OS versions:** Linux kernel 4.15+, macOS 12 (Monterey)+.
- **No Windows support.** Any Windows-specific request is out of scope.

## Operability

- **Zero-config default:** ff must work correctly with no configuration file for all common use cases. All configuration must be optional and overridable via CLI flags.
- **Daemon transparency:** The interactive developer must never need to manually manage the daemon for typical usage. Auto-start and idle timeout must handle lifecycle automatically.
- **Socket isolation:** Each daemon's socket and PID file must be isolated by root path hash to prevent cross-project interference.
- **Log accessibility:** The daemon must write structured logs to a predictable location (`~/.local/state/ff/` or `$XDG_STATE_HOME/ff/`) for debugging.

## Security

- **No remote access:** The Unix socket must be accessible only to the user who started the daemon. No network exposure is permitted.
- **No secret indexing:** ff must not index or cache file contents for files with restrictive permissions (e.g., mode 0600 owned by another user) that the daemon process cannot read.
- **Action safety:** Destructive actions (`-delete`, `-exec` with write commands) must require explicit user opt-in and must not be triggered by default.
