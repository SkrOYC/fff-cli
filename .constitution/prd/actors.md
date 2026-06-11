# Actors

## Primary Actor: Interactive Developer

**Role:** Software developer, systems engineer, or power user who works primarily in a terminal emulator on Linux or macOS.

**Operating Context:**
- Works in repositories and directory trees ranging from a few hundred to several million files
- Runs dozens to hundreds of search queries per day during active development
- Uses shell aliases and custom scripts to streamline workflows
- Expects instantaneous feedback from CLI tools — any delay over ~500ms breaks flow
- May work on NixOS or other non-standard Linux distributions where traditional package managers are not ideal
- Already uses rg, fd, and/or find and is motivated by speed, not novelty

**Goals:**
- Search file contents and locate files without perceptible delay, even on very large trees
- Use a single tool with a consistent interface instead of mentally switching between rg, fd, and find syntax
- Have the tool "just work" — auto-starting daemon, no manual lifecycle management, no configuration for common cases
- Retain full expressive power of existing tools (all flags, output formats, exit codes)

**Frictions:**
- rg times out or takes 30+ seconds on large trees (e.g., /nix/store, monorepos, vendor directories)
- find is slow and has arcane syntax; fd is faster but still re-traverses on every call
- Switching mental models between three different tools with different flag conventions
- Forgetting exact flag names and having to re-read man pages
- Scripts that hardcode rg/fd/find invocations breaking when tools are unavailable

## Secondary Actor: Script Author

**Role:** Developer who writes shell scripts, Makefiles, or CI pipelines that invoke file search as part of automation.

**Operating Context:**
- Runs ff from non-interactive contexts: CI jobs, cron tasks, build scripts
- Cannot rely on a pre-running daemon; each invocation may be isolated
- Values deterministic exit codes, machine-parseable output formats, and POSIX-compatible behavior
- May pipe ff output into other commands or redirect to files

**Goals:**
- Use one-shot mode to run ff without a persistent daemon
- Get identical output and exit codes to rg/fd/find for drop-in script compatibility
- Access machine-readable output formats (JSON, NUL-delimited) for downstream processing

**Frictions:**
- One-shot mode must pay the index-build cost on every invocation, partially negating the speed advantage
- Scripts that depend on obscure rg/fd/find flags may hit unsupported edge cases
- CI environments may have memory constraints that conflict with daemon memory usage
