# Flow: Daemon Auto-Start

**Maps to PRD capabilities:** D-04 (Auto-start), I-05 (Daemon auto-start from CLI)

## Sequence Diagram

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant FS as Filesystem
    participant Daemon

    User->>CLI: ff grep "TODO"
    CLI->>CLI: Determine socket path: /tmp/fffd-<user>/<sha256(cwd)>.sock
    CLI->>FS: Check if socket exists
    
    alt Socket exists
        CLI->>Daemon: Attempt connection
        alt Connection successful
            Daemon-->>CLI: Connected
            CLI->>Daemon: Send query
        else Connection refused (daemon crashed)
            CLI->>FS: Check PID file
            CLI->>CLI: Verify PID is stale (process not running)
            CLI->>FS: Delete stale socket and PID files
            CLI->>CLI: Trigger auto-start
        end
    else Socket does not exist
        CLI->>CLI: Trigger auto-start
    end
    
    CLI->>CLI: Auto-start sequence
    CLI->>FS: Create /tmp/fffd-<user>/ directory (mode 0700)
    CLI->>Daemon: Fork and exec daemon process
    Daemon->>Daemon: Initialize: scan file tree, build index
    Daemon->>FS: Create socket (mode 0600)
    Daemon->>FS: Create PID file
    
    loop Wait for daemon ready (timeout 2s)
        CLI->>FS: Check if socket exists
        alt Socket exists
            CLI->>Daemon: Attempt connection
            alt Connection successful
                Daemon-->>CLI: Connected
                CLI->>Daemon: Send query
            else Connection failed
                CLI->>CLI: Retry after 100ms
            end
        else Socket not yet created
            CLI->>CLI: Wait 100ms
        end
    end
    
    alt Daemon started successfully
        CLI->>Daemon: Send query
        Daemon-->>CLI: Stream results
    else Daemon failed to start (timeout or error)
        CLI-->>User: Error: failed to start daemon
        CLI-->>User: Exit with code 2
    end
```

## Key Steps

1. **CLI determines socket path:** Computes socket path from username and current working directory: `/tmp/fffd-<username>/<sha256(cwd)>.sock`.

2. **CLI checks if socket exists:** If socket file exists, CLI attempts to connect.

3. **CLI handles connection failure:**
   - If connection refused, daemon may have crashed
   - CLI checks PID file to verify daemon is not running
   - CLI deletes stale socket and PID files
   - CLI triggers auto-start

4. **CLI triggers auto-start:**
   - Creates socket directory `/tmp/fffd-<username>/` with mode 0700 (if not exists)
   - Forks and execs daemon process with root path = current working directory
   - Daemon runs in background (detached from CLI)

5. **Daemon initializes:**
   - Scans entire file tree, builds index (~10s for 1M files)
   - Creates socket at `/tmp/fffd-<username>/<sha256(cwd)>.sock` with mode 0600
   - Creates PID file at `/tmp/fffd-<username>/<sha256(cwd)>.pid`
   - Starts filesystem watcher
   - Starts idle timeout timer

6. **CLI waits for daemon ready:** CLI polls socket existence and attempts connection every 100ms, up to 2s timeout.

7. **CLI sends query:** Once connected, CLI sends original query to daemon.

## Error Paths

- **Socket directory creation failed:** CLI returns error (permission denied, disk full), exits 2.
- **Daemon fork/exec failed:** CLI returns error (out of memory, binary not found), exits 2.
- **Daemon startup timeout:** CLI waits 2s for socket to appear. If timeout, CLI returns error, exits 2.
- **Daemon crashed during startup:** CLI detects connection refused, returns error, exits 2.

## Security Considerations

- Socket directory is created with mode 0700 (owner only)
- Socket file is created with mode 0600 (owner read/write only)
- Daemon verifies peer credentials on connection (SO_PEERCRED on Linux)
- PID file prevents multiple daemons for same root
