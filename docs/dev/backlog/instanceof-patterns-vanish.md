---
type: ChangeRequest
kind: bug
title: instanceof pattern matching loses the pattern variable, final, or the whole record pattern
description: The `instanceof` renderers rebuild only `left instanceof right`, silently dropping the pattern variable name (`instanceof String value` → `instanceof String`), an optional `final` keyword, and a record deconstruction pattern (`instanceof Point(int x, int y)` → `instanceof )`) — turning valid Java into different, non-compiling output.
state: done
priority: high
tags: [dev, bug, expression]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-11T18:25:02Z
---

# Problem

Formatting an `instanceof` expression with pattern matching silently drops
part of the pattern. `if (field.getValue() instanceof String value)` formats
to `if (field.getValue() instanceof String)` — the pattern variable `value`
vanishes, and the body that uses it no longer compiles. The same happens for:

- `o instanceof final String s` → `o instanceof String` (`final` and `s` gone);
- `o instanceof Point p` → `o instanceof Point` (`p` gone);
- `o instanceof Point(int x, int y)` → `o instanceof )` — the whole record
  pattern collapses to a stray `)`.

This is silent data loss: the parse succeeds (tree-sitter-java 0.23.5 models
`instanceof_expression` with `left`, an anonymous `instanceof` keyword, an
optional anonymous `final`, and either a `right` type with an optional `name`
pattern variable or a `pattern` record pattern), so no `ParseDiagnostic` is
raised, and the never-corrupt contract ("anything the formatter does not
model — valid or not — is preserved verbatim, never dropped or invented") is
violated.

Root cause: the two `instanceof_expression` renderers in
`crates/core/src/formatter.rs` — `flat` (~L9578) and `expr_ac` (~L7662) — emit
only `left instanceof right` from the `left` and `right` fields. The `name`
field (type-pattern variable), the anonymous `final` token, and the `pattern`
field (record deconstruction pattern) are never rendered.

Switch pattern labels are **not** affected: `switch_label` nodes with type
patterns, guards or record patterns are echoed verbatim (or routed through the
modelled record-pattern shape), so `case String s`, `case Integer i when i > 0`
and `case Point(int x, int y)` all survive — verified, and pinned by the
existing `deconstruction_list_wrap/verbatim` goldens.

# Reproduction

Style: default (`JavaStyle::default()`).

Input:

```java
public class Test {
    void m(Object field) {
        if (field.getValue() instanceof String value) {
            System.out.println(value);
        }
    }
}
```

Observed output:

```java
public class Test {
    void m(Object field) {
        if (field.getValue() instanceof String) {
            System.out.println(value);
        }
    }
}
```

The pattern variable `value` is dropped; the output does not compile.

Also reproduced:

```java
// input                          // observed output
o instanceof final String s       o instanceof String
o instanceof Point p              o instanceof Point
o instanceof Point(int x, int y)  o instanceof )
```

Switch patterns are preserved correctly:

```java
switch (o) { case String s -> ...; case Integer i when i > 0 -> ...; }
// output keeps every case label byte-for-byte.
```

# Proposal

Make the `instanceof` tail renderer emit every token the grammar models, in
order: the optional `final` keyword, then either the type with its optional
pattern-variable name (`String value`, `Point p`) or the flat record pattern
(`Point(int x, int y)`). Reuse the record-pattern machinery already shipped
for switch labels (extract the parts helper from `deconstruction_parts` into a
bare-`record_pattern` variant) and render the record pattern flat with the
deconstruction-list spacing options (`SPACE_BEFORE_DECONSTRUCTION_LIST`,
`SPACE_WITHIN_DECONSTRUCTION_LIST`, `SPACE_AFTER_COMMA`), so `instanceof
Point(int x, int y)` respects the same options as `case Point(int x, int y)`.
Anything outside the modelled shapes echoes verbatim (R4). Both renderers
(`flat` and `expr_ac`) share one `instanceof_tail` helper, and the `left`
handling is unchanged. Switch labels are untouched.

Named record patterns (`instanceof Point(int x, int y) p`, `case Point(int x,
int y) p -> …`) are valid Java but a tree-sitter-java 0.23.5 grammar gap (the
parser emits `ERROR`, surfaced as a `ParseDiagnostic`), so they are
deliberately out of scope: the formatter cannot model what the grammar cannot
parse, and upgrading the grammar is a separate concern.

# Decisions

1. **Fix the renderers, not the grammar** (agreed with the user on
   2026-09-11): `flat` and `expr_ac` now emit the full pattern tail; the
   tree-sitter-java 0.23.5 grammar already parses `instanceof` patterns
   correctly, so only the formatter's output dropped them.
2. **Reuse the deconstruction machinery** (agreed with the user):
   `deconstruction_parts` is split so its core extraction works on a bare
   `record_pattern` node; the instance-of record pattern renders flat through
   the same spacing options as a switch label, so `instanceof` and `case`
   patterns stay consistent under the same scheme. A `where`-guard shape or
   anything else outside the two modelled branches keeps the verbatim echo
   (R4).
3. **Switch labels stay untouched** (agreed with the user): switch patterns
   already round-trip (they are echoed verbatim or modelled), confirmed by
   tests; the fix is scoped to `instanceof_expression`.
4. **Named record patterns deferred** (agreed with the user): the grammar gap
   (tree-sitter-java 0.23.5 cannot parse a name after a record pattern) is
   documented and out of scope; a parse diagnostic already surfaces it.
5. **Tests for the bug wherever they apply** (agreed with the user): an
   `instanceof_patterns` golden-pair suite pins the default-style preservation
   baseline and the spacing-option consistency.

# Acceptance criteria

- `instanceof String value` / `instanceof Point p` keep the pattern-variable
  name; `instanceof final String s` keeps `final` and the name; `instanceof
Point(int x, int y)` keeps the record pattern — all under the default style,
  byte-stable and idempotent.
- The record pattern in `instanceof` honours `SPACE_BEFORE_DECONSTRUCTION_LIST`
  and `SPACE_WITHIN_DECONSTRUCTION_LIST` identically to a switch-label record
  pattern under the same scheme; the absent-option default output is
  unchanged.
- `instanceof` in `if` / `while` / ternary / boolean contexts, with generic
  types (`List<String> list`), keeps every pattern token.
- Switch expression / statement labels with type patterns, guards and record
  patterns are byte-identical to today (existing
  `deconstruction_list_wrap/verbatim` goldens unchanged).
- New goldens: `tests/java/instanceof_patterns/` (type pattern, `final`
  pattern, record pattern, spaced record pattern, combined contexts), wired as
  `tests/options/instanceof_patterns.rs` per the per-option test layout.
- `cargo test --workspace` is green with `cargo clippy --workspace --lib
--bins --tests -- -D warnings` and `cargo fmt --all -- --check` clean.
- A changelog entry is appended on delivery (`fawi-implement`), and the named
  record-pattern grammar gap is noted in the README behaviour notes.

# Implementation plan

## Approach

The fix is confined to `crates/core/src/formatter.rs` and the golden test
suite; the two `instanceof_expression` renderers and the record-pattern
machinery share one tail renderer.

**`record_pattern_parts`** (~L7007): the record-type text and trimmed
component texts of a bare `record_pattern` node (`Type(` + components + `)`),
extracted by unwrapping `switch_label` → `pattern` → `record_pattern` in
`deconstruction_parts` (~L6982, which now delegates to it), and by the
`pattern` field of an `instanceof_expression` directly. Returns `None` for
anything that is not exactly the modelled shape, keeping the verbatim echo
(R4).

**`record_pattern_flat`** (~L7032): renders that shape flat as `Type(A, B)`
with `SPACE_BEFORE_DECONSTRUCTION_LIST` between the type and the `(` and
`SPACE_WITHIN_DECONSTRUCTION_LIST` just inside the parens — the
`deconstruction_flat_label` body without the `case ` prefix — so `instanceof`
and `case` record patterns stay consistent under the same scheme.

**`instanceof_tail`** (~L7058): the canonical tail after `left instanceof `.
Emits the optional anonymous `final` token, then either the `right` type via
`flat_type` with the optional pattern-variable `name` (`String value`,
`Point p`) or the `pattern` record pattern via `record_pattern_flat`
(`Point(int x, int y)`), falling back to the trimmed verbatim source echo (R4)
for anything outside the two modelled shapes. Both renderers — `flat`
(~L9642) and `expr_ac` (~L7730) — call it; the `left` handling and all switch
labels are unchanged.

**Tests**: new `crates/core/tests/options/instanceof_patterns.rs` (5 golden
tests, wired into `tests/options.rs`) with fixtures under
`tests/java/instanceof_patterns/`: `type_pattern` (pattern variable survives),
`final_pattern` (`final` and name survive), `record_pattern` (record
deconstruction survives flat), `record_pattern_spaced` (deconstruction spacing
options apply identically to `instanceof`), `combined` (boolean `if` / `while`
/ ternary contexts with generic types).

**Docs**: the README behaviour notes (~L720) document the preservation and
the named-record-pattern grammar gap; the changelog entry is appended on
delivery.

## Steps

- [x] Split `record_pattern_parts` out of `deconstruction_parts` and delegate
      (`deconstruction_parts` → `record_pattern_parts`); the existing
      switch-label goldens are unchanged.
- [x] Add `record_pattern_flat`, reusing
      `SPACE_BEFORE_DECONSTRUCTION_LIST` / `SPACE_WITHIN_DECONSTRUCTION_LIST`
      / `SPACE_AFTER_COMMA`.
- [x] Add `instanceof_tail` (optional `final` + `right`/`name` or `pattern`,
      verbatim R4 fallback) and call it from both the `flat` and `expr_ac`
      `instanceof_expression` arms.
- [x] Add `tests/options/instanceof_patterns.rs` (5 golden tests) with
      fixtures under `tests/java/instanceof_patterns/` and wire it into
      `tests/options.rs`.
- [x] README: document `instanceof` pattern preservation plus the
      named-record-pattern grammar gap in the behaviour notes.
- [x] Quality gates: `cargo test --workspace`, `cargo clippy --workspace
      --lib --bins --tests -- -D warnings`, `cargo fmt --all -- --check` —
      all green.

## Closing

Shipped on 2026-09-11. `instanceof_tail` now renders the full pattern tail
behind `left instanceof ` in both the `flat` (~L9642) and `expr_ac` (~L7730)
renderers: the optional `final` keyword, the type with its pattern-variable
name (`String value`, `Point p`), or the record pattern rendered flat through
the deconstruction-list spacing options (`Point(int x, int y)`).
`deconstruction_parts` was split so its extraction core lives in
`record_pattern_parts` (~L7007), shared with the new `record_pattern_flat`
(~L7032); switch labels are untouched and every other shape keeps the verbatim
echo (R4). Five golden tests under `tests/java/instanceof_patterns/` pin the
type, `final`, record, spaced-record and combined contexts; the README notes
the preservation and the tree-sitter-java 0.23.5 named-record-pattern grammar
gap. Verified with `cargo test --workspace` (867 core integration tests — five
new — plus 9 core unit and 18 CLI + 6 GUI tests), `cargo clippy --workspace
--lib --bins --tests -- -D warnings`, and `cargo fmt --all -- --check`. The
changes are uncommitted in the worktree (no commit hash); the changelog entry
was added on 2026-09-11.
