# IPC Protocol — JSON-RPC 2.0

## Overview

ff uses JSON-RPC 2.0 over Unix domain sockets for CLI ↔ Daemon communication. Messages are length-prefixed (4-byte big-endian u32 length + JSON payload).

## Transport

- **Socket path:** `/tmp/fffd-<username>/<sha256(cwd)>.sock`
- **Socket permissions:** 0600 (owner read/write only)
- **Message framing:** 4-byte big-endian length prefix + JSON payload
- **Max message size:** 100MB
- **Encoding:** UTF-8

## Methods

### grep — Content Search

**Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "grep",
  "params": {
    "pattern": "TODO",
    "caseMode": "smart",
    "contextLines": { "before": 2, "after": 2 },
    "filters": {
      "extensions": ["rs", "ts"],
      "globs": ["src/**"],
      "fileTypes": ["file"],
      "hidden": false,
      "noIgnore": false
    },
    "maxMatches": 10000
  }
}
```

**Streaming Notifications:**

```json
{
  "jsonrpc": "2.0",
  "method": "result",
  "params": {
    "items": [
      {
        "path": "src/main.rs",
        "lineNumber": 42,
        "column": 8,
        "lineContent": "    // TODO: implement this",
        "matchRanges": [[8, 12]],
        "gitStatus": "modified",
        "fileType": "file"
      }
    ]
  }
}
```

**Final Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "totalMatched": 183,
    "elapsedMs": 52
  }
}
```

### search — Name Search

**Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "search",
  "params": {
    "pattern": "\\.rs$",
    "patternMode": "regex",
    "filters": {
      "extensions": ["rs"],
      "fileTypes": ["file"],
      "excludes": ["target/**"],
      "hidden": false,
      "noIgnore": false
    },
    "maxDepth": 10,
    "minDepth": 0,
    "maxMatches": 10000
  }
}
```

**Streaming Notifications:**

```json
{
  "jsonrpc": "2.0",
  "method": "result",
  "params": {
    "items": [
      {
        "path": "src/main.rs",
        "fileType": "file",
        "size": 1234,
        "mtime": 1718100000
      }
    ]
  }
}
```

**Final Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "totalMatched": 42,
    "elapsedMs": 12
  }
}
```

### find — Metadata Query

**Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "find",
  "params": {
    "expression": "-name *.rs -type f -size +1k",
    "actions": [
      {
        "type": "exec",
        "command": "wc -l",
        "mode": "batch"
      }
    ],
    "maxDepth": 10,
    "minDepth": 0,
    "followSymlinks": false,
    "mountBoundary": false
  }
}
```

**Streaming Notifications:**

```json
{
  "jsonrpc": "2.0",
  "method": "result",
  "params": {
    "items": [
      {
        "path": "src/main.rs",
        "fileType": "file",
        "size": 1234,
        "mtime": 1718100000,
        "mode": 33188,
        "uid": 1000,
        "gid": 1000,
        "inode": 12345678,
        "device": 2049,
        "linkCount": 1
      }
    ]
  }
}
```

**Final Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "totalMatched": 15,
    "elapsedMs": 8,
    "actionResults": [
      {
        "type": "exec",
        "exitCode": 0,
        "stdout": "  42 src/main.rs\n  18 src/lib.rs\n...",
        "stderr": ""
      }
    ]
  }
}
```

### ping — Health Check

**Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "ping",
  "params": {}
}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "status": "ok",
    "version": "0.1.0",
    "rootPath": "/home/user/project",
    "indexedFiles": 54321,
    "uptimeSeconds": 3600,
    "indexStatus": "ready"
  }
}
```

### shutdown — Daemon Shutdown

**Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "shutdown",
  "params": {}
}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "status": "shutting_down"
  }
}
```

## Error Model

All errors follow JSON-RPC 2.0 error format:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "field": "pattern",
      "reason": "invalid regex: unclosed group"
    }
  }
}
```

### Error Codes

| Code | Meaning |
|------|---------|
| -32700 | Parse error (invalid JSON) |
| -32600 | Invalid request (missing jsonrpc, method, or id) |
| -32601 | Method not found |
| -32602 | Invalid params (validation error) |
| -32603 | Internal error |
| -1 | Index rebuilding (retry after delay) |
| -2 | Pattern too broad (too many matches) |
| -3 | Permission denied (unreadable file) |
| -4 | Feature not supported (with suggestion) |

### Feature Not Supported Error

When ff cannot handle a flag or feature natively:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -4,
    "message": "PCRE2 lookaround assertions are not supported",
    "data": {
      "suggestion": "rg --pcre2 '(?<=foo)bar' .",
      "originalFlag": "--pcre2"
    }
  }
}
```

## Type Definitions

### CaseMode

```json
"smart"   // Case-insensitive unless pattern contains uppercase
"sensitive" // Always case-sensitive
"insensitive" // Always case-insensitive
```

### PatternMode

```json
"regex"        // Regular expression
"glob"         // Glob pattern
"fixed-string" // Literal string
```

### FileType

```json
"file"
"directory"
"symlink"
"socket"
"block-device"
"character-device"
"fifo"
"executable"
"empty"
```

### ExecMode

```json
"per-match" // One subprocess per match (-exec {} ;)
"batch"     // One subprocess with all matches (-exec {} +)
"per-match-dir" // One subprocess per match in file's directory (-execdir {} ;)
```

## Versioning

The IPC protocol is versioned independently of the CLI. The `ping` response includes the daemon version. CLI and daemon must have compatible protocol versions.

**Compatibility policy:**
- Same major version: fully compatible
- Different major version: incompatible, CLI must error with version mismatch message
