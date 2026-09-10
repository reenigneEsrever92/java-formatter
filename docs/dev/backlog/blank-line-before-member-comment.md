---
type: ChangeRequest
kind: bug
title: Place the around-member blank lines before a member's leading comment
description: The BLANK_LINES_AROUND_* minimum for a member is emitted between the member's leading comment (e.g. its javadoc) and the declaration instead of before the comment, so a doc comment always ends up followed by a blank line.
state: done
priority: high
tags: [dev, bug, blank-lines]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-10T00:00:00Z
---

# Problem

A member's leading comment is not treated as part of the member for blank-line
spacing. In `crates/core/src/formatter.rs`, `class_body` emits comment nodes as
standalone lines with `blanks: 0` and explicitly excludes them from the spacing
options, then computes the next declaration's gap with `member_gap(prev, m, …)`
where `prev` is the previous _real_ member. The required minimum
(`BLANK_LINES_AROUND_METHOD`, default `1`) is therefore emitted on the
declaration's line — i.e. immediately after the comment — instead of before the
comment that leads the member.

Consequence: any doc comment preceding a method (or a nested class/initializer,
whose around-minimums also default to `1`) is always followed by an empty line,
even when the source has none. There is no option that controls this: the
`BLANK_LINES_AROUND_*` options are the cause, but setting one to `0` also
removes the intended blank lines between members. No javadoc-specific option
exists.

The same pattern appears in three places:

- `class_body` (the member loop at `crates/core/src/formatter.rs` L2877–2906);
- `enum_body`'s declaration loop (L2491–2504, a second copy of the pattern);
- the top-level `program` path, where comments between the package/imports and
  a top-level type are collected as if they were types (L1707–1717) and then
  each gets its own gap (L1794–1825), so the gap of
  `BLANK_LINES_AROUND_CLASS` (default `1`) lands between a top-level javadoc and
  its class.

Fields are unaffected under the built-in defaults only because
`BLANK_LINES_AROUND_FIELD` defaults to `0`; the defect appears for any
`BLANK_LINES_*` around-minimum that is non-zero.

# Reproduction

Default style, source with two methods and a doc comment (no blank lines in the
source):

```java
class A {
    void n() {
    }
    /** Doc. */
    void m() {
    }
}
```

Observed:

```java
class A {
    void n() {}
/** Doc. */

    void m() {}
}
```

Expected — the comment is leading trivia of `m`, so the minimum blank line goes
before the comment and the comment stays attached to the declaration:

```java
class A {
    void n() {}

/** Doc. */
    void m() {}
}
```

Confirmations: with `BLANK_LINES_AROUND_METHOD=0` the blank line disappears (and
with it the intended blank between `n()` and `m()`); with
`KEEP_BLANK_LINES_IN_DECLARATIONS=false` it remains, showing it is the inserted
minimum, not a preserved source blank. The same shape reproduces for an enum
member and for a top-level class with a javadoc after the imports.

# Proposal

Treat a member's leading comment run as part of the member: emit the member's
minimum gap before the **first** leading comment, and between the comments and
between the last comment and the declaration preserve only the source's blank
lines (no forced minimum). Apply this in all three places: `class_body`,
`enum_body`'s declaration loop, and the top-level `program` path (where comments
between the imports/package and a type — including a file-header comment when
there is no package or import — attach to the following type instead of being
spaced as if they were types). `member_gap` gains an explicit "measure to" byte
offset so the gap can be measured to the first leading comment rather than to
the declaration. Comments with no following member (trailing comments in a body
or at the end of the file) keep their current no-gap placement.

# Decisions

1. **It is a bug, not an option** (agreed with the user on 2026-09-10): no
   option keys on javadoc-adjacent blank lines; the `BLANK_LINES_AROUND_*`
   minimum is simply inserted on the wrong side of the comment.
2. **The minimum moves before the comment block** (agreed with the user): the
   comment stays attached to its declaration, matching IntelliJ's layout.
3. **Only the minimum moves** — blank lines the source already has between the
   comment block and the declaration are preserved (measured with a required
   minimum of `0`), so a deliberately separated comment is not reflowed.
4. **All three sites are fixed** (agreed with the user, "identical cases too"):
   `class_body`, `enum_body` and the top-level `program` path, so a comment
   before a method, an enum member or a top-level class behaves the same.
5. **Trailing comments are unchanged**: a comment with no following member keeps
   its current placement (no around-minimum applies).
6. **The comment's own column is out of scope**: the `/**` sitting in column 1 is
   the separate `BLOCK_COMMENT_AT_FIRST_COLUMN` default, not this defect.

# Acceptance criteria

- A doc comment immediately before a method (or nested class / initializer) with
  a non-zero `BLANK_LINES_AROUND_*` minimum is followed by no blank line; the
  required blank lines appear before the comment instead.
- `KEEP_BLANK_LINES_IN_DECLARATIONS=false` still caps those blanks to the
  configured minimum, and a source blank line between the comment and the
  declaration is still preserved.
- Enum members and top-level types with leading comments behave the same; a
  javadoc between the imports and a top-level class is attached to that class
  with no forced blank between them.
- Existing goldens that pinned the misplaced blank are corrected; new golden
  pairs cover a commented method (`blank_lines_around_method`), an enum member
  and a commented top-level class.
- `cargo test --workspace` is green with
  `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
  `cargo fmt --all -- --check` clean.
- A changelog entry is appended on delivery (`fawi-implement`).

# Implementation plan

## Approach

All changes are in `crates/core/src/formatter.rs`.

- **`member_gap` measures to an explicit offset.** Add an `end: usize`
  parameter (the byte to measure the source gap up to) so a caller can measure
  to the first leading comment rather than to the declaration; callers pass
  `m.start_byte()` when the member has no leading comment.
- **`class_body`: buffer each member's leading comments.** Instead of emitting a
  comment as its own `BodyLine` mid-iteration, collect the comment run preceding
  the next member, then: give the first comment the member's gap (measured from
  the previous member, minimum `member_around_min`), give later comments the
  source gap between comments (minimum `0`), and give the declaration the source
  gap after the last comment (minimum `0`). Always update `last` so the closing
  gap still measures from the final emitted node. Flush any trailing comment run
  after the loop with the current no-gap placement.
- **`enum_body`: same buffering** in its `enum_body_declarations` loop, which
  writes directly into `out`; the gap arithmetic mirrors `class_body`.
- **`program`: attach comments between the header and a type.** Keep
  `package_declaration` / `import_declaration` handling as is, but stop treating
  a comment as a pseudo-type: buffer comment children and attach the run to the
  following top-level type, placing the section minimum (`blank_lines_after_imports`
  / `blank_lines_after_package` / `blank_lines_around_class`) before the comment
  run and preserving only source blanks between the comments and the type. When
  the file has no package and no imports, a file-header comment is attached to
  the first type the same way (nothing precedes it, so no leading gap is
  emitted). A comment run with no following type keeps its current placement.
- **Tests.** New golden pairs and assertions: a commented method in
  `crates/core/tests/options/blank_lines_around_method.rs`; a commented enum
  member and a commented top-level class (the latter in
  `blank_lines_around_class.rs`, or `blank_lines_after_imports.rs` for the
  javadoc-after-imports case). Any pre-existing golden that pinned the misplaced
  blank is corrected. Per `.agents/AGENTS.md` every test is a golden pair.
- **Docs.** `README.md`'s blank-line behaviour note gains the leading-comment
  rule; `docs/dev/changelog.md` gets an entry on delivery.

## Steps

- [x] Add the explicit `end` offset to `member_gap` and update its callers.
- [x] Fix `class_body` to place the gap before the member's leading comment run.
- [x] Fix `enum_body`'s declaration loop the same way.
- [x] Fix `program` so comments attach to the following top-level type.
- [x] Correct any existing golden that pinned the misplaced blank and add the new
      golden pairs (commented method, enum member, top-level class).
- [x] Run `cargo test --workspace` — green, with only the intended goldens changed.
- [x] Run `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
      `cargo fmt --all -- --check`.
- [x] Update the README blank-line note and append a `docs/dev/changelog.md` entry.
- [x] Mark the request `done` with `verified`, and set its backlog index row to
      `done`.

## Closing

Shipped on 2026-09-10. The gap machinery ended up as `member_gap_bounds`
(returning the measure-from offset and the required minimum) plus `decl_gap`
(spacing over a byte range), used incrementally as each member's leading comment
run is emitted; `member_gap` itself was removed once both callers moved to the
new helpers. The enum-body loop's `last_content` and the class-body loop's
`last` are updated by the declaration (which follows its comments), so the
closing-brace gap still measures from the last emitted content. Only two
existing goldens pinned the misplaced blank (both the class-level javadoc in
`javadoc_formatting`); they were corrected, and commented-method,
commented-enum-member and commented-top-level-class golden pairs were added.
Verified with `cargo test --workspace` (815 core integration tests — three new —
plus nine core unit tests and six GUI tests),
`cargo clippy --workspace --lib --bins --tests -- -D warnings` (clean; the
two dead store assignments first reported were removed) and
`cargo fmt --all -- --check`. No commit was made as part of this work.
