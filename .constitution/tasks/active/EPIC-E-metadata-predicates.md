# Epic E: Metadata Predicates

**Status:** Active  
**Scope:** ff-query predicate types, expression parser, metadata engine  
**Dependencies:** Epic B (IPC & Daemon)  
**Story Points:** 21

## Overview

Implement the metadata query engine (find subcommand) predicate system. This includes all predicate types (name, path, type, size, time, perm, user, group, inode, links, empty, newer, samefile), the boolean expression parser with AND/OR/NOT and parenthetical grouping, and the predicate evaluator. This is the most complex epic due to the expression parser.

This epic includes one spike to de-risk the expression parser design.

---

#### METAP-E001 ff-query: Predicate Types and Evaluator
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** IPCD-B002, FOUND-A004
- **Description:** Implement all predicate types as an enum with evaluation logic against FileEntry metadata. Predicates: Name (glob match), Path (glob match), Type (file type check), Size (comparison with units), Mtime/Atime/Ctime (comparison with days/minutes), Perm (mode bit check), User/Group (uid/gid check), Inode, Links, Empty, Newer/Anewer/Cnewer (comparison with reference file), SameFile (device+inode check), Readable/Writable/Executable. Implement the recursive evaluator for boolean expression trees (And, Or, Not combinators).
- **Acceptance Criteria (Gherkin):**
```gherkin
Given a FileEntry for src/main.rs (1234 bytes, modified 2 days ago)
When evaluating predicate Name("*.rs")
Then it returns true

Given a FileEntry for a 2MB file
When evaluating predicate Size { op: GreaterThan, n: 1, unit: Megabyte }
Then it returns true

Given a FileEntry for a file with mode 0o755
When evaluating predicate Perm { mode: 0o111, kind: Any }
Then it returns true (executable bits match)

Given a boolean expression And(Name("*.rs"), Type(RegularFile))
When evaluating against a .rs file
Then it returns true
And when evaluating against a directory named "foo.rs"
Then it returns false
```

---

#### METAP-E002 Spike: Find Expression Parser
- **Type:** Spike
- **Effort:** 3
- **Dependencies:** METAP-E001
- **Description:** Design and prototype a parser for find-style boolean expressions. Evaluate parser approach (recursive descent vs. nom vs. lalrpop), implement operator precedence (NOT > AND > OR), handle implicit AND, parse predicate arguments (size specs, time specs, perm specs), and design error messages. Output: parser design document with prototype and test suite. See SPK-E001.md.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the spike is complete
When reading .constitution/spikes/SPK-E001.md
Then it contains a parser design with chosen approach and rationale
And it includes a grammar specification
And it documents operator precedence rules

Given the prototype parser
When parsing "-name *.rs -type f -size +1k"
Then it produces the correct predicate tree
And when parsing invalid expressions
Then it returns clear, actionable error messages
```

---

#### METAP-E003 ff-query: Expression Parser
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** METAP-E002
- **Description:** Implement the find expression parser per the spike recommendation. Parse find-style command-line arguments into a predicate tree. Handle operator precedence, implicit AND, parenthetical grouping, and all predicate argument formats. Produce clear error messages for invalid expressions with suggestions for corrections.
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the expression parser
When parsing "-name *.rs -type f"
Then it produces And(Name("*.rs"), Type(RegularFile))

Given the expression parser
When parsing "\\( -name *.rs -o -name *.ts \\) -type f"
Then it produces And(Or(Name("*.rs"), Name("*.ts")), Type(RegularFile))

Given the expression parser
When parsing "! -name *.log"
Then it produces Not(Name("*.log"))

Given the expression parser
When parsing "-name" (missing argument)
Then it returns an error: "predicate -name requires an argument"

Given the expression parser
When parsing "-foo" (unknown predicate)
Then it returns an error: "unknown predicate -foo" with hint suggesting similar predicates
```

---

#### METAP-E004 ff-query: Metadata Query Engine
- **Type:** Feature
- **Effort:** 5
- **Dependencies:** METAP-E003
- **Description:** Implement the MetadataQueryEngine in ff-query. Parse the expression string into a predicate tree using the expression parser. Iterate over Index entries, evaluate the predicate tree per file, and stream matching paths. Support depth constraints (minDepth, maxDepth), mount boundary (-xdev), and symlink following (-L).
- **Acceptance Criteria (Gherkin):**
```gherkin
Given a metadata query with expression "-name *.rs -type f"
When executing the query
Then it returns all regular .rs files in the index

Given a metadata query with maxDepth 2
When executing the query
Then only files at depth 0, 1, and 2 are evaluated

Given a metadata query with mountBoundary true
When executing the query
Then files on different filesystems are excluded

Given a metadata query with followSymlinks true
When executing the query
Then symlinks are followed and metadata is from the target
```

---

#### METAP-E005 Metadata Predicate Tests
- **Type:** Feature
- **Effort:** 3
- **Dependencies:** METAP-E004
- **Description:** Write comprehensive tests for the predicate system: unit tests for each predicate type, parser tests for expression syntax, integration tests for the metadata engine. Use property-based testing (proptest) for predicate evaluation invariants. Cover edge cases (empty expressions, nested parentheses, permission specifications).
- **Acceptance Criteria (Gherkin):**
```gherkin
Given the predicate test suite
When running `cargo nextest run -p ff-query`
Then all predicate unit tests pass
And all parser tests pass
And all metadata engine tests pass

Given a property-based test for predicate evaluation
When running with 1000 random inputs
Then all invariants hold (e.g., And(a,b) == a && b)

Given a test for the expression parser
When parsing 50 different valid expressions
Then all produce correct predicate trees
And when parsing 20 invalid expressions
Then all produce clear error messages
```
