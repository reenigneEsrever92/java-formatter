---
type: ChangeRequest
kind: bug
title: instanceof with a qualified-type record pattern loses the deconstruction
description: Formatting `o instanceof Outer.Inner.Record(var value)` drops the `(var value)` deconstruction — tree-sitter-java 0.23.5 cannot parse a record pattern whose type is a qualified name, and the field-based statement renderers silently discard the resulting `ERROR` node, emitting output that no longer compiles.
state: done
priority: high
tags: [dev, bug, expression]
owner: maintainer
verified: { by: Zed coding agent, at: 2026-09-14 }
---

# Problem

Formatting an `instanceof` whose record pattern has a **qualified** type name
silently drops the deconstruction list. `obj instanceof Outer.Inner.Record(var
value)` formats to `obj instanceof Outer.Inner.Record` — the `(var value)`
deconstruction is gone, the body that uses `value` no longer compiles, and the
drop happens without any sign in the output. It violates the never-corrupt
contract (R4: anything the formatter does not model is preserved verbatim,
never dropped or invented) that the README states as a guarantee.

There are two layers:

1. **Grammar gap.** tree-sitter-java 0.23.5's `record_pattern` production
   accepts only `identifier` / `_reserved_identifier` / `generic_type` before
   the component list, never a `scoped_type_identifier`
   (`record_pattern: $ => seq(choice($.identifier, $._reserved_identifier, $.generic_type), $.record_pattern_body)`).
   A qualified record type — valid Java (`ReferenceType ( ComponentPatternList )`,
   JLS §14.30.1) — therefore does not parse as a record pattern. The parser
   recovers with `right: A.B.C` (a `scoped_type_identifier`), a zero-width
   `MISSING ")"`, and an `ERROR` node carrying `(var value)`.
2. **Formatter drop.** The `if_statement` / `while_statement` renderers rebuild
   their output from the modelled fields (`condition`, `consequence`,
   `alternative`, …) and never emit the sibling `ERROR` child, so its text is
   silently discarded.

No released tree-sitter-java fixes the grammar gap: 0.23.5 is the newest
release and upstream `master`'s `record_pattern` production is identical, so
the fix belongs in the formatter.

# Reproduction

Style: default (`JavaStyle::default()`).

Input:

```java
public class Test {
    void m(Object o) {
        if (o instanceof Outer.Inner.Record(var value)) {
            handle(value);
        }
    }
}
```

Observed output — the deconstruction is dropped:

```java
public class Test {
    void m(Object o) {
        if (o instanceof Outer.Inner.Record) {
            handle(value);
        }
    }
}
```

Expected output — the whole statement is preserved verbatim:

```java
public class Test {
    void m(Object o) {
        if (o instanceof Outer.Inner.Record(var value)) {
            handle(value);
        }
    }
}
```

The trigger is a _qualified_ record-pattern type. With the default style:

| Input (inside the `if`)     | Observed                                |
| --------------------------- | --------------------------------------- |
| `o instanceof Foo(var n)`   | unchanged (simple name parses)          |
| `o instanceof A.B(var n)`   | `o instanceof A.B` — `(var n)` lost     |
| `o instanceof A.B.C(var n)` | `o instanceof A.B.C` — `(var n)` lost   |
| `o instanceof A.B(var n) p` | `o instanceof A.B` — list and name lost |

Switch labels (`case A.B(var n) -> …`) are unaffected — they already echo their
source verbatim. Some non-statement contexts (local variable initialiser,
ternary, `return`, field, call argument) keep the `ERROR` text through the
`stmt` catch-all / member verbatim echoes, though its placement can be odd; the
token loss is specific to the field-based statement renderers.

# Proposal

Make the formatter honour the never-corrupt contract for statements the parser
could only partially recover: in the statement dispatcher (`Fmt::stmt`), a
statement whose subtree contains an unparsed `ERROR` node is emitted verbatim
instead of being rebuilt from its fields. Because every statement (`if` /
`else if` / `while` / `do` / `for` / `try` / `synchronized` / `switch` / …) is
dispatched through `stmt`, one guard covers the whole data-loss class rather
than the single reported shape. Whitespace inside such a statement is not
reformatted — an acceptable trade for never dropping content (R4).

The guard predicate is "the subtree contains an `ERROR` node", **not**
`Node::has_error()`: `has_error()` is also true for a zero-width `MISSING`
token (the grammar inserts a `MISSING ")"` for this bug), and the enclosing
`parenthesized_expression`'s own source text is then incomplete
(`(o instanceof A.B`), so echoing it verbatim would drop the closing paren.
Only an `ERROR` node's range is guaranteed to sit inside the node being echoed.

# Decisions

1. **Fix the formatter, not the grammar** (agreed with the user on
   2026-09-14): tree-sitter-java 0.23.5 is the newest release and upstream
   `master` has the same `record_pattern` production, so no version bump fixes
   the parse; a vendored/generated grammar is a separate concern and is not
   taken on here.
2. **A statement-level verbatim guard, not an `instanceof` repair** (agreed with
   the user): the `ERROR` node is a sibling of the `instanceof_expression`
   (inside the enclosing statement), so it cannot be seen or reconstructed from
   `instanceof_tail`; splicing it back into `left instanceof right` output would
   duplicate the recovered tokens. Emitting the recovered statement verbatim is
   the minimal change that guarantees nothing is dropped.
3. **The predicate is "contains an `ERROR` node"** (agreed with the user):
   `has_error()` would fire on the zero-width `MISSING` token and truncate the
   verbatim echo, so an explicit `ERROR`-only walk is used, behind a
   `has_error()` fast path so valid input pays nothing.
4. **Scope is statements** (agreed with the user): the dropped-content cases are
   the field-based statement renderers; contexts that already preserve the
   `ERROR` text (local variable, ternary, `return`, field, argument) are left
   as they are.
5. **Tests in the existing `instanceof_patterns` suite** (agreed with the user):
   `AGENTS.md` forbids new topic / parse-error suites, and the suite is already
   the home of `instanceof` pattern preservation, so the grammar-gap case is
   added there as a golden pair.

# Acceptance criteria

- The reported input formats with the whole statement preserved: the
  deconstruction `(var value)` and the qualified type survive verbatim, and the
  output is byte-stable and idempotent (formatting it again is a no-op).
- The same holds for a qualified-type record pattern in a `while`, a
  `do … while` and an `else if` alternative.
- Valid input is unaffected: every existing fixture's golden stays
  byte-identical (`cargo test --workspace` green), including the simple-type
  `instanceof` record patterns under the deconstruction spacing options.
- New goldens: `tests/java/instanceof_patterns/scoped_record_pattern.java` +
  `.out.java`, wired as a test in `tests/options/instanceof_patterns.rs`.
- `cargo test --workspace` is green with `cargo clippy --workspace --lib --bins
--tests -- -D warnings` and `cargo fmt --all -- --check` clean.
- The README behaviour notes state that a record pattern with a qualified type
  name is a tree-sitter-java 0.23.5 grammar gap and that the affected statement
  is preserved verbatim (never dropped); `docs/requirements.md` gains an R47 row
  for the verbatim guarantee; a changelog entry is appended on delivery.

# Implementation plan

## Approach

The change is confined to `crates/core/src/formatter.rs` (one helper, one guard,
one `else if` routing fix) plus the golden fixture, tests, README, requirements
and changelog.

**`contains_error_node`** (module-level, beside the parse-diagnostic helpers
~L119): returns false immediately for a node whose `has_error()` is false (the
valid-input fast path), otherwise walks the subtree for an `is_error()` node.

**`Fmt::stmt`** (~L6024): as the first statement of the function, return
`self.txt(node).to_string()` when `contains_error_node(node)` — a recovered
statement is echoed verbatim (R4). The verbatim text is the statement's own
source range, so nothing in it can be dropped; the `block` / `switch_rule`
callers keep prefixing the first line with the indent and gluing `-> body`
after it.

**`Fmt::if_stmt`** (~L6300, ~L6306): the `else if` alternatives are routed
through `self.stmt(...)` instead of `self.if_stmt(...)` so an errored `else if`
also takes the guard (equivalent dispatch for an `if_statement`).

**Tests**: `tests/java/instanceof_patterns/scoped_record_pattern.java` +
`.out.java` — a class whose method holds an `if`, an `else if`, a `while` and a
`do … while` over qualified-type record patterns — plus one golden assertion in
`tests/options/instanceof_patterns.rs`.

**Docs**: the README `instanceof` behaviour bullet (~L720) is corrected to
describe the qualified-type grammar gap and the verbatim preservation;
`docs/requirements.md` gains R47 and a milestones mention;
`docs/dev/changelog.md` is appended on delivery.

## Steps

- [x] Add `contains_error_node` and the verbatim guard at the top of
      `Fmt::stmt`; route the `else if` alternatives through `Fmt::stmt`.
- [x] Add `tests/java/instanceof_patterns/scoped_record_pattern.java` +
      `.out.java` and the golden assertion in
      `tests/options/instanceof_patterns.rs`.
- [x] README: correct the `instanceof` grammar-gap note to state the affected
      statement is preserved verbatim.
- [x] `docs/requirements.md`: add the R47 row and a milestones mention.
- [x] Quality gates: `cargo test --workspace`, `cargo clippy --workspace --lib
      --bins --tests -- -D warnings`, `cargo fmt --all -- --check` — all green.
- [x] `docs/dev/backlog/index.md`: add the new row; append the changelog entry.

## Closing

Shipped on 2026-09-14. `Fmt::stmt` (`crates/core/src/formatter.rs`) now echoes a
statement verbatim when its subtree contains a tree-sitter `ERROR` node — the
new `contains_error_node` helper walks the subtree for an `is_error()` node
behind a `has_error` fast path, so the zero-width `MISSING ")"` a grammar gap
inserts does not trigger the echo — and the `else if` alternatives in
`Fmt::if_stmt` route through `stmt` so they take the guard too. A record pattern
whose type is a qualified name (`o instanceof Outer.Inner.Record(var v)`) —
valid Java that tree-sitter-java 0.23.5 cannot parse — therefore survives
verbatim in an `if` / `else if` / `while` / `do … while` / `for` / `try` /
`synchronized` / `switch` statement instead of losing its deconstruction. A new
golden pair `tests/java/instanceof_patterns/scoped_record_pattern.{java,out.java}`
plus its assertion in `tests/options/instanceof_patterns.rs` pins the
preservation; the README behaviour notes and `docs/requirements.md` (R47)
document it. Verified with `cargo test --workspace` (892 core integration tests —
one new — plus 9 core unit, 18 CLI and 6 GUI tests), `cargo clippy --workspace
--lib --bins --tests -- -D warnings`, and `cargo fmt --all -- --check`. The
changes are uncommitted in the worktree (no commit hash); the changelog entry
was added on 2026-09-14.
