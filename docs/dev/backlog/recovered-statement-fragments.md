---
type: ChangeRequest
kind: bug
title: A statement recovered as an ERROR fragment in a block is split into separate statements
description: When tree-sitter-java recovers a statement it cannot finish as a parsed prefix plus an `ERROR` fragment (and often a parsed tail) directly in the block, the block renders the pieces as separate lines — moving the fragment out of its statement and emitting invalid Java.
state: done
priority: high
tags: [dev, bug, expression]
owner: maintainer
verified: { by: Zed coding agent, at: 2026-09-14 }
---

# Problem

The previous fix ([instanceof with a qualified-type record pattern loses the
deconstruction](scoped-record-pattern-vanishes.md)) made a *statement* whose
subtree contains a tree-sitter `ERROR` node render verbatim. But tree-sitter
does not always keep the recovered fragment inside the statement: for a
statement it cannot finish it emits the parsed prefix as one node, the unparsed
remainder as a sibling `ERROR` node, and often the parsed tail as a further
sibling — all direct children of the enclosing `block`. The `block` renderer
walks its children and renders each as its own statement, so a ternary over a
qualified-type record pattern is torn into three lines:

```java
// input
int r = o instanceof Outer.Inner.Record(var value) ? 1 : 2;
```

```java
// observed
int r = o instanceof Outer.Inner.Record; (var value) ? 1 :
        2;
```

The fragment and the tail are moved out of their statement, the `;` lands in
the wrong place, and the output no longer compiles — a never-corrupt violation
(R4). The same tear happens for a plain declaration
(`boolean b = …;`), a `return`, and an assignment; only an `ERROR` nested
*inside* a statement (e.g. inside a call argument) was already handled by the
statement guard.

# Reproduction

Style: default (`JavaStyle::default()`).

Input:

```java
class C {
    void m(Object o) {
        int r = o instanceof Outer.Inner.Record(var value) ? 1 : 2;
    }
}
```

Observed output — the statement is split:

```java
class C {
    void m(Object o) {
        int r = o instanceof Outer.Inner.Record; (var value) ? 1 :
        2;
    }
}
```

Expected output — the broken statement is preserved verbatim as one line:

```java
class C {
    void m(Object o) {
        int r = o instanceof Outer.Inner.Record(var value) ? 1 : 2;
    }
}
```

The tree for the input is
`block[ { , local_variable_declaration("int r = o instanceof Outer.Inner.Record", MISSING ";"), ERROR("(var value) ? 1 :"), expression_statement("2;"), } ]`
— three named children of the block that make up one statement.

# Proposal

In `block`, when a named child is followed by an `ERROR` child, the two (and any
further sibling that begins on the same source line — the parsed tail) are one
broken statement, not separate statements. Emit the whole run as one verbatim
line (R4): the text runs from the first child's start byte to the start of the
next sibling (or the block's `}` when the run is last), trimmed. The surrounding
statements keep their normal formatting, so only the broken statement is echoed.
The run is detected by the same direct-child `ERROR` test as before, so valid
input — no `ERROR` child — is untouched and pays nothing.

# Decisions

1. **Merge the run in `block`, not a whole-block verbatim echo** (agreed with the
   user on 2026-09-14): echoing the whole block would also stop formatting every
   other statement of the method; merging only the broken run keeps the
   untouched statements formatted. The block-with-a-direct-`ERROR` case is
   exactly the one the earlier statement-level guard cannot see.
2. **The run boundary is the same-source-line rule** (agreed with the user): the
   run continues while the next sibling starts on the same row as the previous
   sibling's end, which captures the parsed tail (`2;`) of a torn ternary
   without swallowing the following line's statements. A statement on the same
   line as the fragment would be merged too — still verbatim, so still lossless.
3. **The statement-level guard stays** (agreed with the user): it covers an
   `ERROR` nested inside a statement (e.g. a call argument), which the block
   never sees; this request adds only the direct-child case.

# Acceptance criteria

- A ternary over a qualified-type record pattern in a declaration
  (`int r = o instanceof Outer.Inner.Record(var value) ? 1 : 2;`) formats as one
  verbatim statement — the fragment is not moved out and the `;` stays put.
- The same holds for a plain declaration, a `return o instanceof … ? 1 : 2;`,
  and an assignment statement.
- Statements around the broken one in the same block keep their normal
  formatting (a mixed fixture pins this).
- Output is byte-stable and idempotent (formatting it again is a no-op).
- Valid input is unaffected: every existing fixture's golden stays
  byte-identical (`cargo test --workspace` green).
- New goldens: `tests/java/instanceof_patterns/scoped_record_pattern_in_expression.java`
  + `.out.java`, wired as a test in `tests/options/instanceof_patterns.rs`.
- README behaviour notes describe the block-fragment recovery;
  `docs/requirements.md` gains an R48 row; a changelog entry is appended on
  delivery.

# Implementation plan

## Approach

The change is confined to `Fmt::block` in `crates/core/src/formatter.rs`, plus
the golden fixture and docs.

**`Fmt::block`** (~L5806): in the child loop, after the protected-run and
trailing-comment branches, if `stmts[i + 1]` is an `ERROR` node then children
`i..=j` form one broken statement, where `j` extends while the next child is an
`ERROR` or starts on the same row as the previous child's end. The run is pushed
as a single `BodyLine` whose text is
`src[stmts[i].start_byte()..end_byte].trim_end()`, with `end_byte` the next
sibling's start byte or `node.end_byte() - 1` (the closing `}`) when the run is
last; `prev_end` / `prev_row` move to the run's last child and `i` skips past
`j`. The normal path is unchanged, so valid blocks behave exactly as before.

**Tests**: `tests/java/instanceof_patterns/scoped_record_pattern_in_expression.java`
+ `.out.java` — a class with a plain declaration, a ternary declaration, a call
argument, and a `return`, all over qualified-type record patterns — plus the
golden assertion in `tests/options/instanceof_patterns.rs`.

**Docs**: the README `instanceof` behaviour bullet gains the block-fragment
case; `docs/requirements.md` gains R48; `docs/dev/changelog.md` is appended.

## Steps

- [x] Merge a statement-plus-`ERROR` run in `Fmt::block` into one verbatim line
      (same-source-line boundary; normal path untouched).
- [x] Add `tests/java/instanceof_patterns/scoped_record_pattern_in_expression.java`
      + `.out.java` and the golden assertion in
      `tests/options/instanceof_patterns.rs`.
- [x] README: extend the `instanceof` grammar-gap note to the block-fragment
      recovery.
- [x] `docs/requirements.md`: add the R48 row.
- [x] Quality gates: `cargo test --workspace`, `cargo clippy --workspace --lib
      --bins --tests -- -D warnings`, `cargo fmt --all -- --check` — all green.
- [x] `docs/dev/backlog/index.md`: add the new row; append the changelog entry.

## Closing

Shipped on 2026-09-14. `Fmt::block` (`crates/core/src/formatter.rs`) now detects
a statement the parser split into a parsed prefix plus an `ERROR` fragment
(and its parsed tail) directly in the block and emits the whole run verbatim as
one line, using a same-source-line boundary; the surrounding statements keep
their formatting. A ternary, a declaration, a `return` and an assignment over a
qualified-type record pattern therefore survive verbatim instead of being torn
into invalid statements. A new golden pair
`tests/java/instanceof_patterns/scoped_record_pattern_in_expression.{java,out.java}`
plus its assertion in `tests/options/instanceof_patterns.rs` pins the
behaviour; the README behaviour notes and `docs/requirements.md` (R48) document
it. Verified with `cargo test --workspace` (893 core integration tests — one new
— plus 9 core unit, 18 CLI and 6 GUI tests), `cargo clippy --workspace --lib
--bins --tests -- -D warnings`, and `cargo fmt --all -- --check`. The changes are
uncommitted in the worktree (no commit hash); the changelog entry was added on
2026-09-14.
