---
type: ChangeRequest
kind: bug
title: A comment in a declaration's modifier area is joined onto the code or silently dropped
description: A comment that sits inside a declaration's modifiers (or as another extra child of the declaration) is treated as a keyword modifier — a line comment then comments out the rest of the declaration — or is ignored outright, so a comment is corrupted or lost and the output is neither valid nor stable.
state: done
priority: high
tags: [dev, bug, comments, formatter]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-11T21:10:00Z
---

# Problem

A `//` or `/* */` comment placed among a declaration's modifiers is not
recognised as a comment. Tree-sitter attaches it either **inside the
`modifiers` node** (when a keyword modifier such as `private` / `public` /
`static` follows it) or as a **direct extra child of the declaration** (when it
is the last thing before the type). The formatter handles neither case:

- **Treating a comment as a modifier.** `mods_parts`
  (`crates/core/src/formatter.rs` L4330) iterates _all_ children of `modifiers`
  and pushes every non-annotation child into the keyword-modifier list,
  including the `line_comment` / `block_comment`. `mods_per_line` (L4379) and
  `mods_inline` (L4359) then join it with the real keyword modifiers, so a line
  comment ends up on the same line as the following code and comments it out.
  `flat_mods` (L5021) has the same defect for the flat parameter form.
- **Ignoring a comment child.** The declaration renderers read only `modifiers`
  and their named fields (`method_decl` L3979, `constructor_decl` L4063,
  `compact_constructor_decl` L4134, the type declarations, the parameter
  renderers), so a comment that is a direct child of the declaration is never
  emitted and disappears. `field_decl` (L4200) is the only one that survives,
  and only by accident: its guard at L4204 echoes the whole node verbatim when
  it has a direct comment child.

Both outcomes violate the never-corrupt contract (a comment is content and must
be preserved, R4) and the idempotency contract (R6): the corrupted formatter
output puts `private int revision = 0;` _inside_ a `//` comment, so formatting
it a second time sees a bare annotation and the field is gone. This is the same
family as the shipped
[comments in comma-separated lists](comments-in-comma-separated-lists.md) bug,
but a distinct location (comments in a `modifiers` node / declaration extras,
not between list elements), so it is a new request rather than a duplicate.

# Reproduction

Default style (the comment-joining cases are independent of the style; the repo
`codestyle.xml` reproduces the same shapes).

The reported case:

```java
public abstract class Repro<ID> {

    @Version
    @Column(name = "revision")
    // TODO change to long (incl. respective db columns)
    private int revision = 0;

    @OverridingMethodsMustInvokeSuper
    protected <T extends AggregateRoot<ID>> T copyNonModifiableFields(T target) {
        return target;
    }
}
```

Observed (the `//` comment is joined with the field and comments it out):

```java
    @Version
    @Column(name = "revision")
    // TODO change to long (incl. respective db columns) private int revision = 0;
```

All reproduced shapes:

| Input                                      | Observed                                      | Failure     |
| ------------------------------------------ | --------------------------------------------- | ----------- |
| `@Ann` `//c` `private int a = 0;`          | `@Ann` / `//c private int a = 0;`             | corruption  |
| `@Ann` `//c` `public void b() {}`          | `@Ann` / `//c public void b() {}`             | corruption  |
| `@Ann` `//c` `static class Inner {}`       | `@Ann` / `//c static class Inner {}`          | corruption  |
| `public` `//c` `void c() {}`               | `//c` **dropped**                             | silent loss |
| `@Ann` `//c` `void m() {}`                 | `//c` **dropped**                             | silent loss |
| `@Ann` `//c` `enum E {}`                   | `//c` **dropped**                             | silent loss |
| `@Ann` `/*c*/ int p` (parameter)           | `/*c*/` **dropped**                           | silent loss |
| `@Ann` `//c` `int a = 0;` (no keyword mod) | preserved (verbatim echo via the L4204 guard) | accidental  |

Expected: the comment is preserved at its source position, each comment on its
own line, and the declaration's code still parses on the next line — e.g. the
reported case keeps

```java
    @Version
    @Column(name = "revision")
    // TODO change to long (incl. respective db columns)
    private int revision = 0;
```

and formatting that output again is a no-op (R6). A comment that trails a
modifier or another statement stays where it is (see Decisions).

# Proposal

Stop treating comments as modifiers and emit them at their source position.

- **Never a modifier.** `mods_parts` and `flat_mods` skip `line_comment` /
  `block_comment` children instead of collecting them as keyword modifiers.
- **Emit comments in place.** Collect the comment children of a declaration and
  its `modifiers` node, in source order, into the declaration's layout. Each
  comment is emitted on its own line at the declaration's indent (the comment
  column options — `LINE_COMMENT_AT_FIRST_COLUMN`, `BLOCK_COMMENT_AT_FIRST_COLUMN`,
  `KEEP_FIRST_COLUMN_COMMENT` — still apply), and a break-forcing comment
  (`//`, or a block comment spanning rows) forces the declaration out of any
  single-line / inline modifier form so no code ever shares its line. A comment
  that follows a modifier or a statement stays in that position, per the user's
  decision. Source blank lines around the comment are preserved; the member's
  `BLANK_LINES_AROUND_*` minimum stays before the first leading comment, as
  established by
  [blank-line-before-member-comment](blank-line-before-member-comment.md).
- **Cover the family.** Apply this in `field_decl`, `method_decl`,
  `constructor_decl`, `compact_constructor_decl`, `local_var`, the type
  declarations (`class_decl`, `iface_decl`, `enum_decl`, `record_decl`,
  annotation types), record components, and the parameter renderers
  (`flat_param` / `wrapped_param` / `formal_params`).
- **R4 fallback where no layout can carry a comment.** Constructs with no
  multi-line form of their own — a flat parameter list inside a nested flat
  expression, type arguments rendered flat, a construct collapsed to one line by
  `KEEP_SIMPLE_*` — echo their verbatim source when a comment is present,
  matching the approach already taken in `comments-in-comma-separated-lists`.
- **No option change.** The behaviour is gated on the presence of a comment, so
  declarations without comments render byte-identically to today and no
  `OptionDef` is added.

Docs touched on delivery: the README formatting-behaviour notes gain the
modifier-area comment rule, `docs/requirements.md` gets a requirement row for
comment preservation in declarations, and `docs/dev/changelog.md` an entry.

# Decisions

1. **Fix the whole family (user choice).** The corruption and the silent losses
   share two adjacent mistakes in the same code paths, and the corruption turns
   valid Java into un-compilable output, so every declaration kind and both
   attachment positions are in scope.
2. **A comment is never a modifier (agreed with the user).** `mods_parts` /
   `flat_mods` exclude comment children, which removes the inline-join
   corruption at its source.
3. **Own-line placement, position preserved (user choice).** A comment in the
   modifier area is emitted on its own line before its declaration, and a
   comment that sits behind a modifier or another statement stays in that
   position — the formatter reorders neither the comment nor the code around
   it.
4. **Verbatim R4 fallback where no layout can carry a comment.** A construct
   whose only form is flat (a flat parameter list, flat type arguments, a
   one-line `KEEP_SIMPLE_*` collapse) echoes its source rather than corrupting
   or dropping the comment, reusing the disclosed fallback pattern from
   `comments-in-comma-separated-lists`.
5. **No option change.** The fix is gated on comment presence; absent-comment
   output is unchanged and no `OptionDef` is added.
6. **Reported-merge caveat.** The exact cross-member merge in the report (the
   method's annotation hoisted above the field and the field and method joined
   on one line) was not reproducible from the pasted snippet on the current
   `main`; the reproducible corruption and losses above are the authoritative
   reproduction. If the reporter can supply the original file, the merge is to
   be added to the reproduction (it is expected to be another manifestation of
   the same root cause).

# Acceptance criteria

- A field, method, constructor, compact constructor, local variable, type
  declaration, record component or parameter carrying a comment in its
  modifier area keeps the comment on its own line; no code shares a line with a
  `//` comment, and no comment is dropped.
- The reported field reproduces to `@Version` / `@Column(name = "revision")` /
  `// TODO …` / `private int revision = 0;`, and formatting that output again is
  a no-op (R6).
- A comment behind a modifier (`public` `//c` `void c()`) or another statement
  stays in that position; its source blank lines are preserved and the member's
  `BLANK_LINES_AROUND_*` minimum stays before the first leading comment.
- The comment column options still apply (`LINE_COMMENT_AT_FIRST_COLUMN`,
  `BLOCK_COMMENT_AT_FIRST_COLUMN`, `KEEP_FIRST_COLUMN_COMMENT`).
- Constructs with no multi-line form fall back to the verbatim source (R4)
  rather than corrupting or dropping the comment.
- Declaration output with no comment child is byte-identical to today; no
  existing golden changes.
- Golden pairs under `tests/java/comment_in_modifiers/` cover the field,
  method, class/enum, parameter, local-variable and behind-a-modifier shapes,
  are wired in `tests/options/comment_in_modifiers.rs` per the per-option test
  layout, and are idempotent.
- `cargo test --workspace` is green with
  `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
  `cargo fmt --all -- --check` clean.
- `docs/requirements.md`, the README behaviour notes and a
  `docs/dev/changelog.md` entry are updated on delivery.

# Implementation plan

## Approach

All formatter changes are in `crates/core/src/formatter.rs`; regression coverage
is new golden pairs under `crates/core/tests/`.

**A comment-aware modifier-part model.** Add
`enum ModPart<'s> { Annotation(Node<'s>), Keyword(String), Comment(Node<'s>) }`
and `mods_parts_ordered(&self, mods) -> Vec<ModPart<'s>>`, which walks `all_ch`
and classifies each child as an annotation, a comment
(`is_comment_node`), or a keyword modifier. Reimplement the existing
`mods_parts` (`(anns, keywords)`) and `flat_mods` on it so both **skip comment
children** — a comment is never a keyword modifier, which removes the
inline-join corruption in `field_decl` / `method_decl` / `class_decl` /
`enum_constant` / `flat_param` at its source. `mods_parts`' signature is
unchanged for its other callers (`has_annotation`, `mods_single_annotation`,
`record_component_block`, `formal_params`).

**Declaration modifier area.** Add
`decl_leading_comments(&self, node) -> Vec<Node<'s>>`: the declaration's own
comment children that appear before its first non-modifier child (the type or
keyword) — the comments tree-sitter attaches after the `modifiers` node and
that the declaration renderers currently ignore. `decl_mod_parts(&self, node)`
concatenates the `modifiers` node's ordered parts with a `Comment` part per
leading comment. Two renderers consume it:

- `decl_mods_per_line(&self, node, indent) -> String` builds the block
  line-by-line: each annotation and each comment occupies its own line, keyword
  modifiers between breaks are joined with single spaces, and no `//` is ever
  followed by code on its line. Comments render through the existing
  `comment(node, indent)` helper, so the comment column / space / wrap options
  still apply. The block's **first** line is returned unindented (the callers —
  `class_body`, `program` — add the member indent), matching `mods_per_line`'s
  existing contract; a keyword-terminated block ends without a trailing break so
  `mods_tail` still inserts the pre-type gap, and an annotation/comment-
  terminated block ends with a fresh `ind(indent)` line.
- `decl_mods_inline(&self, node, indent) -> Option<String>` returns `None` when
  any part is a comment (a comment forces the own-line layout), else the current
  annotation + keyword join. Because it is `None`, `mods_inline_decision`
  selects the own-line form (`single_ann_exempts` cannot override it).

**Wire the declaration renderers.** `field_decl`, `method_decl`,
`constructor_decl`, `compact_constructor_decl`, `class_decl`, `iface_decl`,
`enum_decl`, `record_decl` and `local_var` switch from
`mods_per_line`/`mods_inline` to `decl_mods_per_line`/`decl_mods_inline`, and
`has_mods` becomes `!decl_mod_parts(node).is_empty()`. The declaration header
closes with `mods_tail` unchanged (the modifiers-only default output is
byte-identical).

**Keep the R4 verbatim fallback for comments the layout cannot carry.** The
`field_decl` / `local_var` guard currently echoes the whole node verbatim for
_any_ direct comment child; refine it to fire only for a comment **after the
type** (the multi-declarator case pinned by
`comments_in_lists/multi_declarator`), leaving the modifier-area comments to the
new own-line layout. `enum_constant`, `record_component_block` and `flat_param`
echo their node verbatim when it carries a modifier-area comment (no own-line
form of their own).

**Parameter / record-component lists.** `formal_params` and `record_components`
add "an entry carries a comment" to their wrap decision so the verbatim entry
lands on its own list line; the existing `list_entries` / `list_forces_wrap`
machinery keeps handling comments _between_ entries.

No option is added: the behaviour is gated on a comment child, so declarations
without comments render byte-for-byte as today.

Docs: the README formatting-behaviour notes gain the modifier-area comment rule;
`docs/requirements.md` a requirement row for comment preservation in a
declaration's modifier area; `docs/dev/changelog.md` an entry on delivery.

## Steps

- [x] Add `ModPart` and `mods_parts_ordered`; reimplement `mods_parts` /
      `flat_mods` on it so comments are skipped — AC: no comment is ever joined
      as a modifier; no-comment output unchanged.
- [x] Add `decl_leading_comments` + `decl_mod_parts` and the
      `decl_mods_per_line` / `decl_mods_inline` renderers — AC: comments on
      their own line, column options honoured, first line unindented.
- [x] Wire the nine declaration renderers onto the new renderers and set
      `has_mods` from `decl_mod_parts` — AC: field / method / constructor /
      compact constructor / class / interface / enum / record / local variable.
- [x] Refine the `field_decl` / `local_var` verbatim guard to the post-type
      comment only; keep the `multi_declarator` golden byte-identical.
- [x] Verbatim fallbacks for `enum_constant`, `record_component_block`,
      `flat_param`; force wrap in `formal_params` / `record_components` when an
      entry carries a comment — AC: parameter / component / enum-constant
      comment preserved.
- [x] Tests: golden pairs under `tests/java/comment_in_modifiers/` (field,
      method, class/enum, parameter, local variable, behind-a-modifier) wired in
      `tests/options/comment_in_modifiers.rs` via `#[path]` in `options.rs`; each
      pair asserts equality and idempotency — AC: reproduced shapes fixed.
- [x] Docs: README behaviour note + `docs/requirements.md` requirement row.
- [x] Quality gates: `cargo test --workspace`,
      `cargo clippy --workspace --lib --bins --tests -- -D warnings`,
      `cargo fmt --all -- --check` — all green, no existing golden changed.
- [x] Append the `docs/dev/changelog.md` entry (on delivery, via
      `fawi-implement`).

## Closing

Shipped on 2026-09-11. `crates/core/src/formatter.rs` now models a
declaration's modifier area as an ordered `ModPart` sequence (annotation /
keyword / comment): `mods_parts` and `flat_mods` skip comment children so a
comment is never rendered as a keyword modifier, and the new
`decl_leading_comments` / `decl_mod_parts` / `decl_mods_per_line` /
`decl_mods_inline` helpers lay each comment out on its own line at its source
position. The nine declaration renderers (`field_decl`, `method_decl`,
`constructor_decl`, `compact_constructor_decl`, `class_decl`, `iface_decl`,
`enum_decl`, `record_decl`, `local_var`) use the new renderers; the
`field_decl` / `local_var` verbatim guard now fires only for a comment after the
type (the multi-declarator case, unchanged), `local_var` keeps its declaration
flat when a comment is present (matching `field_decl`), and `wrapped_param` lays
a commented parameter's annotation / comment lines out at the element prefix.
`enum_constant`, `record_component_block` and `flat_param` keep their source
verbatim (R4) when they carry a comment, and `formal_params` / `record_components`
force the wrapped layout for such an entry. Seven golden pairs under
`tests/java/comment_in_modifiers/` are wired through
`tests/options/comment_in_modifiers.rs`. Verified with `cargo test --workspace`
(874 core integration tests — seven new — plus 9 core unit, 18 CLI and 6 GUI
tests), `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
`cargo fmt --all -- --check`; no existing golden changed. The changes are
uncommitted in the worktree (no commit hash).
