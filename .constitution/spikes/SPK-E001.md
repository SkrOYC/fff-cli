# Spike: Find Expression Parser

**Spike ID:** SPK-E001  
**Related Epic:** EPIC-E-metadata-predicates  
**Related Ticket:** METAP-E002  
**Timebox:** 3 days

## Objective

Design and prototype a parser for find-style boolean expressions, evaluating complexity, edge cases, and implementation approach.

## Background

The `find` subcommand must support boolean expressions combining predicates with AND, OR, NOT, and parenthetical grouping:

```bash
ff find . \( -name "*.rs" -o -name "*.ts" \) -type f -size +1k
ff find . ! -name "*.log" -mtime -7
ff find . -name "*.rs" -exec wc -l {} +
```

The expression grammar is:

```
expression := predicate
            | expression '-a' expression    # implicit AND
            | expression '-o' expression    # OR
            | '!' expression                # NOT
            | '(' expression ')'            # grouping
            | predicate expression          # implicit AND

predicate := '-name' PATTERN
           | '-path' PATTERN
           | '-type' TYPE
           | '-size' SPEC
           | '-mtime' N
           | '-atime' N
           | '-ctime' N
           | '-perm' SPEC
           | '-user' NAME
           | '-group' NAME
           | '-inum' N
           | '-links' N
           | '-empty'
           | '-newer' FILE
           | '-anewer' FILE
           | '-cnewer' FILE
           | '-samefile' FILE
           | '-readable'
           | '-writable'
           | '-executable'
```

This is a non-trivial parser with operator precedence and associativity rules.

## Investigation Areas

### 1. Parser Approach Options

**Option A: Hand-written recursive descent parser**
- Full control over error messages
- No external dependencies
- More code to write and maintain

**Option B: Parser combinator library (nom, combine)**
- Declarative grammar definition
- Good error messages with nom
- Adds dependency

**Option C: Parser generator (lalrpop)**
- Formal grammar specification
- Efficient generated parser
- Adds build-time dependency

**Questions to answer:**
- Which approach is simplest for this grammar?
- How do we handle operator precedence (AND binds tighter than OR)?
- How do we handle implicit AND (two predicates in a row)?

### 2. Operator Precedence

The `find` command has specific precedence rules:
- NOT (`!`) has highest precedence
- AND (`-a`, implicit) has middle precedence
- OR (`-o`) has lowest precedence

Example: `-name "*.rs" -o -name "*.ts" -type f` parses as:
```
(-name "*.rs") OR ((-name "*.ts") AND (-type f))
```

**Questions to answer:**
- How do we implement precedence in the parser?
- Should we use precedence climbing or Pratt parsing?
- How do we handle ambiguous expressions?

### 3. Implicit AND

In `find`, two predicates in a row are implicitly ANDed:
```bash
find . -name "*.rs" -type f
# Equivalent to: find . -name "*.rs" -a -type f
```

But actions break the implicit AND:
```bash
find . -name "*.rs" -print -type f  # Error: -print is an action, not a predicate
```

**Questions to answer:**
- How do we distinguish predicates from actions in the parser?
- Should we parse the entire expression first, then separate predicates/actions?
- How do we handle errors like `-print -name "*.rs"` (action before predicate)?

### 4. Predicate Argument Parsing

Each predicate has different argument types:
- `-name PATTERN`: glob pattern (string)
- `-size SPEC`: comparison operator + number + unit (e.g., `+1M`, `-1k`)
- `-mtime N`: comparison operator + number (e.g., `+7`, `-1`)
- `-perm SPEC`: mode specification (e.g., `/6000`, `-g+w`, `u=rwx`)

**Questions to answer:**
- How do we parse comparison operators (+, -, exact)?
- How do we parse size units (c, k, M, G)?
- How do we parse permission specifications (octal, symbolic)?

### 5. Error Messages

`find` error messages are notoriously cryptic:
```
find: unknown predicate `-foo'
find: invalid argument `+abc' to `-mtime'
```

We need clear, actionable error messages:
```
error: unknown predicate `-foo'
  hint: did you mean `-name'?
error: invalid argument `+abc' to `-mtime'
  hint: expected a number, optionally prefixed with + or -
```

**Questions to answer:**
- How do we provide good error messages with our chosen parser?
- Should we suggest corrections for typos?
- How do we handle missing arguments (e.g., `-name` without pattern)?

### 6. Edge Cases

- Empty expression: `ff find .`
- Unmatched parenthesis: `ff find . \( -name "*.rs"`
- Nested parentheses: `ff find . \( \( -name "*.rs" \) \)`
- Multiple NOT: `ff find . ! ! -name "*.rs"`
- Empty parentheses: `ff find . \( \)`
- Action in wrong position: `ff find . -print -name "*.rs"`

**Questions to answer:**
- How do we handle each edge case?
- Should we be strict (error) or lenient (warn and continue)?
- How do we match `find`'s behavior for compatibility?

## Deliverables

1. **Parser design document** with:
   - Chosen parser approach (with rationale)
   - Grammar specification (BNF or similar)
   - Operator precedence rules
   - Error handling strategy

2. **Prototype parser** (in a separate test file) demonstrating:
   - Parsing simple expressions
   - Parsing complex expressions with parentheses
   - Error messages for invalid expressions

3. **Predicate argument parsers** for:
   - Size specifications (+1M, -1k)
   - Time specifications (+7, -1)
   - Permission specifications (/6000, -g+w)

4. **Test suite** covering:
   - Simple predicates
   - Boolean combinations
   - Parenthetical grouping
   - Edge cases
   - Error cases

## Success Criteria

- Parser handles all predicates defined in PRD (M-01 through M-21)
- Operator precedence matches `find` behavior
- Error messages are clear and actionable
- Parser performance: <1ms for typical expressions
- No external parser dependencies (or justified dependency if using nom/lalrpop)

## References

- PRD: capabilities.md (Epic 3: Metadata Query)
- TechSpec: contracts/cli-interface.md (find subcommand)
- Architecture: flows/flow-metadata-query.md
