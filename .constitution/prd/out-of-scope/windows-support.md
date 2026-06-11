# Out of Scope: Windows Support

## Context

During the PRD interview, Windows support was considered and explicitly rejected.

## Reasoning

1. **Unix-specific architecture.** ff relies on Unix domain sockets for IPC, POSIX file metadata (uid, gid, inode, device, mode bits), inotify for filesystem watching, and systemd for service management. None of these exist natively on Windows.

2. **Target user base.** The primary users — developers on Linux and macOS, particularly NixOS users — do not use Windows as their primary development platform. The performance problem ff solves (slow search on large trees) is most acute on the filesystems these users work with (Nix store, large monorepos on ext4/APFS).

3. **Metadata model incompatibility.** The find-compatible metadata query engine depends on POSIX concepts (permissions, ownership, inode numbers) that have no direct Windows equivalents. Emulating these via Windows ACLs and NTFS attributes would be a significant additional effort with questionable fidelity.

4. **Scope containment.** Supporting Windows would require a parallel IPC mechanism (named pipes), a parallel filesystem watcher (ReadDirectoryChangesW), a parallel service management approach, and extensive testing on a platform the primary developer does not use.
