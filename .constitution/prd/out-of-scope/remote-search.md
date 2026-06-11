# Out of Scope: Remote / Network Search

## Context

During the PRD interview, remote search (SSH integration, network filesystems, client-server across machines) was considered and explicitly rejected.

## Reasoning

1. **Core value proposition is local speed.** ff's performance advantage comes from keeping a full file tree in local RAM. Network latency fundamentally undermines this model — remote stat calls and content fetches would reintroduce the exact bottleneck ff exists to eliminate.

2. **Daemon architecture assumes local filesystem.** The daemon watches a local filesystem tree via inotify/FSEvents. Extending this to remote mounts or SSH tunnels would require a fundamentally different architecture (distributed index, network protocol, auth).

3. **Scope containment.** ff targets interactive developers searching local repos and directories. Remote search is a different product with different constraints, different users, and different failure modes.

4. **Existing tools cover this.** For remote search, users can SSH into the target machine and run ff there, or use tools like `rg --ssh` or `codesearch`.
