---
type: ChangeRequest
kind: bug
title: A trailing comment is moved onto its own line instead of staying behind the code
description: A comment that trails code on the same source line — after a statement, a member, a type, a list element, or an opening brace — is always emitted on its own line; IntelliJ keeps it on the code's line, and no scheme option controls this.
state: done
priority: high
tags: [dev, bug, comments, formatter]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-11T22:05:00Z
---

# Problem

A comment that sits behind code on the same source line is always relocated onto
its own line. Every emit site treats a comment as standalone trivia:

- `block` (`crates/core/src/formatter.rs`) emits each extra (`s.is_extra()`)
  child as its own `BodyLine` (L5474–5485) with no "is it on the same line as
  the previous statement?" check, so `int x = 1; // c` becomes two lines.
- `class_body` / `enum_body` buffer a comment as _leading trivia of the next
  member_, so `int x = 0; // c` is moved to its own line and the following
  member's `BLANK_LINES_AROUND_*` minimum is inserted between the comment and
  the code.
- `program` does the same for a top-level type's trailing comment (and, out of
  scope here, for a `package …; // c` / `import …; // c` line).
- The comma-separated list renderers (`list_entries`) attach a comment to the
  following element, so `f(a, // c` moves the comment onto the line before `b`.

There is no scheme option for this: the six comment options
(`LINE_COMMENT_AT_FIRST_COLUMN`, `BLOCK_COMMENT_AT_FIRST_COLUMN`,
`KEEP_FIRST_COLUMN_COMMENT`, `LINE_COMMENT_ADD_SPACE_ON_REFORMAT`,
`LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION`, `WRAP_COMMENTS`) are about column,
the `//` space and wrapping. IntelliJ keeps a trailing comment on the code's
line, so this is a divergence. It is also a correctness problem, not only a
cosmetic one: tooling markers are line-scoped (`// $NON-NLS-1$`,
`// noinspection …`, `// NOPMD`, `// CHECKSTYLE:OFF`) and stop working once
moved off their line.

# Reproduction

Default style (indented comments are pinned to column 1 by the built-in
`*_AT_FIRST_COLUMN` defaults; a scheme with the toggles off shows the same
relocation at the code indent).

Input:

```java
class T {
    int x = 0; // field trailing
    void m() { // method brace trailing
        int y = 1; // local trailing
    } // method end trailing
} // class end trailing
```

Observed (every comment moved to its own line, with a blank line inserted before
the following member):

```java
class T {
    int x = 0;

// field trailing

    void m() {
// method brace trailing
        int y = 1;
// local trailing
    }
// method end trailing
}
// class end trailing
```

Expected: every comment stays behind the code on its own line:

```java
class T {
    int x = 0; // field trailing
    void m() { // method brace trailing
        int y = 1; // local trailing
    } // method end trailing
} // class end trailing
```

The same shape reproduces for `class C { // c`, `if (…) { // c`,
`} else { // c`, `} // class end`, and a comma-separated list element
(`f(a, // c`). A comment trailing a `package` / `import` line is **out of
scope** (the import engine reorders imports, so attaching a trailing comment to
one would need it carried through the engine) and keeps its current own-line
placement.

# Proposal

Wherever content is emitted, a comment whose start row equals the end row of the
content just emitted (or the row of an opening `{`) is appended to that line as
`" " + comment text` instead of being emitted as its own line. Single-line
comments (`//` and a `/* … */` with no newline) trail; a multi-line block
comment keeps its current own-line placement (it cannot share a line without
spanning). A trailing comment ignores
`*_AT_FIRST_COLUMN` and `KEEP_FIRST_COLUMN_COMMENT` and never wraps (it may be
long); the optional space after `//` (`LINE_COMMENT_ADD_SPACE_ON_REFORMAT` /
`…_IN_SUPPRESSION`) still applies to its text.

In a comma-separated list a trailing comment is emitted after the element and
**after** the separator comma (`a, // c`), so a `//` never swallows the comma or
the following element; a trailing comment on the last element forces the closing
delimiter onto its own line where the layout would otherwise glue it to that
line. A member's trailing comment belongs to that member and does not trigger the
following member's `BLANK_LINES_AROUND_*` minimum.

# Decisions

1. **It is a bug, not an option** (agreed with the user): IntelliJ keeps a
   trailing comment in place and no scheme option governs it.
2. **All confirmed contexts are fixed** (user choice, "all of them"): statement
   blocks, class / enum / interface members, top-level types, an opening `{`
   line, and comma-separated list elements.
3. **`package` / `import` trailing comments are out of scope.** A trailing
   comment on a `package` or `import` line would have to be carried through the
   import engine (which reorders imports), so it keeps its current own-line
   placement; a follow-up can add it with the engine support.
4. **Both `//` and single-line `/* … */` trail** (user choice); a multi-line
   block comment keeps its own-line placement.
5. **Trailing comments ignore the column options** (user choice):
   `LINE_COMMENT_AT_FIRST_COLUMN`, `BLOCK_COMMENT_AT_FIRST_COLUMN` and
   `KEEP_FIRST_COLUMN_COMMENT` are about comments on their own line.
6. **A trailing member comment belongs to the preceding member** (user choice)
   and does not trigger the following member's around-minimum.
7. **Lists place the trailing comment after the separator comma**, so the
   comment is valid and cannot swallow list syntax; a trailing line comment on
   the last element pushes the closing delimiter to its own line.
8. **No option is added**; the behaviour is gated on the comment's source row, so
   output without trailing comments is byte-identical (no existing golden
   contains a same-line trailing comment).

# Acceptance criteria

- A `//` or single-line `/* … */` comment behind a statement, a member, a type,
  an opening `{`, or a list element stays on that line; every reproduction above
  is fixed and idempotent (R6).
- A `package` / `import` trailing comment is unchanged (documented out of scope).
- A multi-line block comment trailing code keeps its own-line placement.
- A trailing comment ignores `LINE_COMMENT_AT_FIRST_COLUMN` /
  `BLOCK_COMMENT_AT_FIRST_COLUMN` / `KEEP_FIRST_COLUMN_COMMENT`; the `//`-space
  options still apply.
- A trailing comment on a list element sits after the separator comma; a
  trailing comment on the last element pushes the closing delimiter to its own
  line where needed, and the output re-parses.
- A trailing member comment does not trigger the next member's
  `BLANK_LINES_AROUND_*` minimum.
- New golden pairs under `tests/java/trailing_comments/` cover each context,
  wired in `tests/options/trailing_comments.rs`; existing goldens are unchanged
  and the full suite is green.
- `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
  `cargo fmt --all -- --check` are clean.
- Docs updated: README behaviour notes, `docs/requirements.md`, and a
  `docs/dev/changelog.md` entry on delivery.

# Implementation plan

## Approach

All formatter changes are in `crates/core/src/formatter.rs`; regression coverage
is new golden pairs under `crates/core/tests/`.

**Shared helpers.** Add `is_trailing_comment(c)` — a `//`, or a `/* … */`
with no newline — and `trailing_suffix(c)` returning `" " + text` with no
column placement and no wrapping. Factor the `//`-space decision out of
`comment()` into a shared `comment_body_text(c)` used by both, so a trailing
comment still honours `LINE_COMMENT_ADD_SPACE_* `.

**Statement blocks (`block`).** Track the row of the last emitted content. A
child that is a trailing comment starting on that row (or, for the first child,
on the block's `{` row) is appended to the last emitted line — or to the `{`
line — instead of becoming its own `BodyLine`; the measured end byte still
advances so the following gap is unchanged.

**Members (`class_body`, `enum_body`).** A comment starting on the last
member's end row (or on the body's `{` row) is appended to that member's line
(or the `{` line) instead of being buffered as leading trivia, so it does not
pick up the following member's `BLANK_LINES_AROUND_*` minimum.

**Top-level (`program`).** A comment starting on the previous top-level type's
end row is appended to that type's last line. A `package` / `import` trailing
comment is out of scope (the import engine reorders imports).

**Lists.** `list_entries` additionally returns each element's trailing comment
run (comments on that element's end row). `flat_entry` appends the single-line
block trailing comments (a `//` already forces the wrapped layout through
`list_forces_wrap`); the wrapped renderers own the separator comma and emit the
trailing run after it, so a `//` never swallows the comma; a trailing `//` on
the last element forces the closing delimiter onto its own line where the
layout would otherwise glue it.

No option is added; the behaviour is gated on the comment's source row, so
output without trailing comments is byte-identical.

Docs: the README behaviour notes gain the trailing-comment rule,
`docs/requirements.md` a requirement row, and `docs/dev/changelog.md` an entry.

## Steps

- [x] Add `is_trailing_comment` / `trailing_suffix` and factor the `//`-space
      logic into `comment_body_text` — AC: `//`-space options still apply; no
      column / wrap for a trailing comment.
- [x] `block`: keep a trailing comment on the statement's line and on the `{`
      line — AC: statement and brace-line reproductions fixed.
- [x] `class_body` / `enum_body`: keep a trailing comment on the member's line
      and the body's `{` line; do not apply the around-minimum — AC: member and
      brace-line reproductions fixed.
- [x] `program`: keep a trailing comment on a top-level type's line
      (`package` / `import` out of scope) — AC: type reproduction fixed.
- [x] Lists: per-element trailing runs in `list_entries`, comment-after-comma
      in the wrapped renderers, closing-delimiter guard — AC: list-element
      reproduction fixed and re-parses.
- [x] Tests: golden pairs under `tests/java/trailing_comments/` (statement,
      member, list, default-column) wired in
      `tests/options/trailing_comments.rs` — AC: every reproduction fixed and
      idempotent.
- [x] Docs: README behaviour note + `docs/requirements.md` requirement row.
- [x] Quality gates: `cargo test --workspace`,
      `cargo clippy --workspace --lib --bins --tests -- -D warnings`,
      `cargo fmt --all -- --check` — green, no existing golden changed.

## Closing

Shipped on 2026-09-11. `crates/core/src/formatter.rs` now keeps a `//` or
single-line `/* … */` comment that starts on the same source row as the content
it follows: the `//`-space logic moved into a shared `comment_body_text`, and
`is_trailing_comment` / `trailing_suffix` / `row_at` back the rule. `block`
tracks the last emitted row and appends a trailing comment to the statement's
line (or the block's `{` line); `class_body` and `enum_body` do the same for a
member and the body's `{` (so a member's trailing comment no longer picks up the
next member's `BLANK_LINES_AROUND_*` minimum); `program` appends a comment
trailing a top-level type to its closing line. `list_entries` now returns each
element's trailing run, `flat_entry` inlines a trailing block comment, and the
wrapped renderers (`record_components`, `formal_params`, `args_wrapped`,
`array_init`, `annotation_expanded`, `ann_array_elems`, `clause_list`) own the
separator comma and place a `//` after it, forcing the closing delimiter onto
its own line when the last element trails a `//`. A `package` / `import`
trailing comment is deliberately out of scope (the import engine reorders
imports). Four golden pairs under `tests/java/trailing_comments/` are wired
through `tests/options/trailing_comments.rs`. Verified with
`cargo test --workspace` (879 core integration tests — four new — plus 9 core
unit, 18 CLI and 6 GUI tests),
`cargo clippy --workspace --lib --bins --tests -- -D warnings` and
`cargo fmt --all -- --check`; no existing golden changed. The changes are
uncommitted in the worktree (no commit hash).
