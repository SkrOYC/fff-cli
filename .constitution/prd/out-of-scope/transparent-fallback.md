# Out of Scope: Transparent Fallback

## Context

During the PRD interview, transparent fallback (automatically shelling out to rg/fd/find when ff encounters an unsupported flag or feature) was considered and explicitly rejected in favor of error-with-suggestion.

## Reasoning

1. **Hidden dependency creation.** Transparent fallback would silently require rg, fd, or find to be installed on the system. Users who chose ff as a replacement would unknowingly depend on the very tools they are replacing, defeating the purpose.

2. **Inconsistent performance.** A transparently-fallback query would run at rg/fd/find speed without any indication to the user why. This creates confusing and unpredictable performance — sometimes 50ms, sometimes 30 seconds — with no explanation.

3. **Debugging opacity.** When output comes from a fallback tool, it may differ subtly from ff's native output format, breaking scripts or confusing users. The indirection makes it hard to diagnose whether a problem is in ff or in the fallback tool.

4. **Clean error is better.** An error message that says "this flag is not supported; run `rg --foo` instead" is explicit, honest, and actionable. The user can choose to install rg, adjust their command, or request the feature in ff.
