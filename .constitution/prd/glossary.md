# Glossary — Ubiquitous Language

| Term | Definition | Do Not Use |
| :--- | :--- | :--- |
| Daemon | A long-running background process that maintains an in-memory index of a file tree and responds to queries via a Unix socket. One daemon exists per indexed root directory. | server, service, background worker, fffd |
| Index | The in-memory data structure holding file metadata and optionally file content for all entries in a scanned tree. Built once on daemon startup and kept current via filesystem watchers. | cache, database, store |
| Query | A single user-issued search request (content, name, or metadata) dispatched from the CLI to the daemon for evaluation against the index. | request, command, search |
| Content Search | A query that matches a pattern against the indexed file contents, returning matching lines with context. Replaces rg functionality. | grep, text search, full-text search |
| Name Search | A query that matches a pattern against indexed file paths/names, returning matching file paths. Replaces fd functionality. | file search, glob search, find by name |
| Metadata Query | A query that evaluates boolean predicate expressions against cached file metadata (size, time, permissions, ownership, inode). Replaces find functionality. | find, stat query, predicate evaluation |
| Predicate | A single test within a metadata query expression (e.g., name matches glob, size exceeds threshold, type is directory). Predicates combine via boolean operators. | filter, condition, test |
| Action | A side-effect operation triggered by a metadata query result, such as executing a command per match, deleting files, or printing formatted output. | handler, callback, operation |
| Expression | A tree of predicates combined with boolean operators (AND, OR, NOT) and optional grouping, forming a complete metadata query filter. | query, filter chain, predicate list |
| One-shot Mode | An execution mode where the CLI creates a temporary index, runs a single query, and exits — used for scripts, CI, or environments where a persistent daemon is impractical. | standalone mode, direct mode, no-daemon mode |
| Idle Timeout | A configurable duration after which the daemon automatically exits if no queries have been received. | inactivity timer, TTL, expiry |
| File Tree | The complete hierarchical set of files and directories under a given root path that the daemon has scanned and indexed. | directory tree, filesystem tree, repo |
| CLI | The command-line binary (`ff`) that parses user input, dispatches to the appropriate subcommand, communicates with the daemon via socket, and formats output. | client, frontend, wrapper |
| Subcommand | One of the three primary modes of operation: `grep` (content search), `search` (name search), or `find` (metadata query + actions). | mode, command, verb |
| Fallback | Not a product concept. When ff cannot handle a request natively, it returns an error with the equivalent command in the original tool. There is no automatic delegation. | delegation, passthrough, proxy |
