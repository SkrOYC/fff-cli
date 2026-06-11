# Out of Scope: IDE / Editor Integrations

## Context

During the PRD interview, IDE and editor integrations (VS Code extension, LSP server, Emacs/Vim plugins) were considered and explicitly rejected.

## Reasoning

1. **CLI-first product.** ff is a terminal tool. Its output formats, interaction model, and daemon lifecycle are designed for human-in-terminal and script consumption. IDE integration would require a different interface contract (incremental results, cancellation, position mapping).

2. **Existing editor tooling already wraps CLI tools.** VS Code's search, vim's `:grep`, Emacs's `consult-ripgrep` — these invoke CLI tools and parse their output. ff's rg-compatible output format means existing editor integrations that wrap rg can be pointed at ff with zero changes.

3. **Scope containment.** Building and maintaining editor plugins multiplies the surface area significantly (different extension APIs, update cycles, user expectations). The CLI is the product; editor compatibility comes for free via output parity.

4. **No unique value-add.** An ff-specific editor plugin would not offer capabilities that the CLI + existing editor wrappers cannot already provide.
