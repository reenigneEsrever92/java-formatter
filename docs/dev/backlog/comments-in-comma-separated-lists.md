---
type: ChangeRequest
kind: bug
title: Comments in comma-separated lists get a list separator, corrupt the construct, or vanish
description: A comment between two elements of a comma-separated list is treated as an element, so it receives a separator comma and can swallow the following code on a flat layout; in the enum-constant and multi-declarator lists it is dropped outright.
state: done
verified: { by: Zed coding agent, at: 2026-09-11T00:00:00Z }
priority: high
tags: [dev, bug, comments, formatter]
owner: maintainer
---

# Problem

Every comma-separated list renderer in `crates/core/src/formatter.rs` builds its
element list with `self.named(node)`. Tree-sitter attaches comments as named
extras, so a `line_comment` / `block_comment` between two elements is collected
as if it were an element and then receives the list's separator comma. That
produces three failure modes:

1. **Corruption.** A line comment that ends up on a line shared with following
   code swallows the rest of the construct. On a flat list the whole header /
   argument list / type-argument list can be commented out, so the formatted
   output is not valid Java and a second format pass fails to parse. Affects
   record headers, annotation arguments, type arguments, lambda parameters,
   `throws` clauses, deconstruction patterns, and the flat form of call
   arguments and array initializers.
2. **Stray separator.** In a wrapped list the comment sits on its own line but
   still gets a trailing `,` (inside the comment for a line comment, after `*/`
   for a block comment). Valid, but wrong.
3. **Silent loss.** Lists whose renderer ignores non-matching children drop the
   comment entirely: an enum constant list (`enum_body` matches only
   `enum_constant` / `enum_body_declarations`) and a multi-declarator field /
   local declaration lose every comment between the constants / declarators.

The comment-layout request (`comment-layout.md`) established that comments are
echoed verbatim (R4) and only whitespace/layout may change (R5); this bug
violates that contract for comments that sit inside a list.

# Reproduction

Given:

```java
@Builder
public record CriterionDto(
        UUID importId,
        UUID criterionId,
        UUID persistentId,
        String name,
        // …
        boolean reviewRequired,
// TODO fm - support it!?
//    TableModel tableModel,
        String importDocumentType,
        // …
        List<FileKindTypeDto> fileKindTypes) {
}
```

`java-formatter --style codestyle.xml` emits the record header on one line with
the comments as components:

```java
public record CriterionDto(UUID importId, // …, boolean reviewRequired, // TODO fm - support it!?, //    TableModel tableModel,, String importDocumentType, // …) {}
```

The `//` comments swallow the rest of the header → invalid Java; re-formatting
the output reports `parse error`.

The same root cause is reachable in ten more contexts (all reproduced):

| Context                        | Observed                                                       |
| ------------------------------ | -------------------------------------------------------------- |
| record header                  | flat header, comment swallows the rest → invalid               |
| annotation arguments           | `@Ann(a = "x", // TODO b, b = 2)` → invalid                    |
| type arguments                 | `Map<// TODO type arg, String, Integer>` → invalid             |
| lambda parameters              | `(String s, // c, String t) -> s` → invalid                    |
| `throws` clause                | `throws IOException, // c, SQLException {}` → invalid          |
| deconstruction pattern         | comment lines kept inside a one-line switch collapse → invalid |
| call arguments (flat)          | `consume(aaa, bbb, // c, ccc)` → invalid                       |
| array initializer (flat)       | `{aaa(), // c, bbb()}` → invalid                               |
| method / constructor params    | own-line comment with a stray trailing `,` — valid but wrong   |
| enum constants                 | `enum Color {RED, GREEN, BLUE}` — comment **dropped**          |
| multi-declarator field / local | `int field1, field2, field3;` — comment **dropped**            |

Expected: a comment between list elements is preserved verbatim and laid out as
its own item — the element it precedes, or the list's closing delimiter, follows
on the next line at the element indent — and it never receives a separator
comma. A line comment (which cannot share its line with code) forces the list to
its wrapped layout; a single-line block comment may stay inline before its
element. Existing comment text is never rewritten (R4/R5) and the result
re-formats to itself (R6).

# Proposal

Add one shared comment-aware list mechanism to `Fmt` and route every affected
renderer through it.

Configuration: none — this is a layout/correctness fix, not an option. The
behaviour is gated on the presence of a comment child, so lists without comments
render byte-identically to today and the existing goldens stay green (no current
fixture has a comment inside a list).

Helpers (in `crates/core/src/formatter.rs`):

- `comment_text(node)` — the comment's source text with a trailing line break
  trimmed (a `line_comment` token may carry it).
- `comment_forces_break(node)` — true for a `line_comment` or a block comment
  spanning source rows.
- `list_entries(node)` — partitions a list node's named children into
  `(leading comments, element)` entries plus the trailing comment run after the
  last element.
- `list_forces_wrap(node)` — true when any comment child forces a break.
- `flat_entry(comments, elem)` / `flat_list_items(node, render)` — inline the
  single-line block comments before their element; `None` when a break-forcing
  comment is present (the caller must wrap).
- `wrapped_entry(comments, elem, prefix)` / `wrapped_list_items(node, render,
prefix)` — a break-forcing comment on its own line at `prefix`, a single-line
  block comment inlined before the element, each element prefixed.

Renderers updated: `record_components`, `formal_params`, `flat_args` +
`args_wrapped`, `flat_arr_init` + `array_init`, `flat_ann_args` +
`annotation` / `annotation_expanded`, `clause_list` (+ `type_list_items`), and
`enum_body` (constants), `enum_one_line_body`, `field_decl`, `local_var`, and the
deconstruction / switch one-line collapse.

Wrap-forcing: each renderer with a wrapped form (record header, params, call
args, array initializer, annotation, clause list) includes `list_forces_wrap` in
its wrap decision, so a line comment reaches the wrapped path. Flat list
renderers used inside a nested flat expression fall back to the list node's
verbatim source text (R4) when a break-forcing comment is present — this keeps
nested cases valid without a wrapped form of their own.

No wrapped form exists for **type arguments** and **lambda parameters**
(`GENERIC_TYPE_PARAMETER_INDENT` is inert; type arguments always render flat),
so those two fall back to the verbatim list source (R4) whenever a comment is
present. Multi-declarator lists have no per-declarator break layout, so a
declaration carrying a comment is echoed verbatim (R4) rather than split —
this fixes the silent loss without inventing a new layout.

Docs touched: `docs/requirements.md` (a new requirement row for comment-in-list
preservation), the README formatting-behaviour notes, and
`docs/dev/changelog.md` on delivery.

# Decisions

1. **Fix the family, not the reported case (user choice).** The root cause is
   shared by every `self.named` + comma-join renderer, and the reproduction
   corrupts valid Java in eight contexts and silently loses comments in two, so
   all eleven are in scope.
2. **Comment attached to the following element, on its own line (user choice).**
   A break-forcing comment is emitted at the element indent before its element
   (or before the closing delimiter for a trailing comment run), with no
   separator of its own. A single-line block comment stays inline before its
   element. This mirrors the existing `class_body` / `enum_body` member-comment
   handling.
3. **A line comment forces the wrapped layout.** A `line_comment` cannot share
   its line with following code, so the list breaks even when it would otherwise
   fit the margin; a multi-line block comment is treated the same. Comment text
   is preserved (R4/R5) and the output re-formats to itself (R6).
4. **Deliberate R4 verbatim fallbacks where no wrapped form exists.**
   Type arguments, lambda parameters and multi-declarator lists have no layout
   that can carry an own-line comment, so a comment there echoes the construct's
   source (R4) instead of corrupting it. This is the same contract the engine
   already applies to unmodelled constructs.
5. **No option change.** The fix is gated on comment presence only, so no
   `OptionDef` is added and default/absent schemes keep today's output
   byte-for-byte wherever no list comment exists.

# Acceptance criteria

- The reproduction formats to valid Java with the two header comments preserved
  on their own lines and no stray commas, and re-formats to itself (R6).
- One golden pair per affected context under `tests/java/comments_in_lists/`
  asserts a comment is preserved, receives no separator, and yields valid,
  idempotent output: record header, method/ctor params, call arguments, array
  initializer, annotation arguments, type arguments, lambda parameters, `throws`
  clause, deconstruction pattern, enum constants, multi-declarator
  field/local.
- A line comment forces a wrapped/own-line layout in every context that has a
  wrapped form; a single-line block comment stays inline.
- No existing golden changes and `cargo test --workspace` is green; the new
  goldens each format to themselves on a second pass (R6).
- `cargo fmt --all -- --check` and
  `cargo clippy --workspace --lib --bins --tests -- -D warnings` are clean;
  `docs/requirements.md`, the README and `docs/dev/changelog.md` are updated.

# Implementation plan

## Approach

Shared helpers plus per-renderer wiring, with wrap-forcing on the renderers that
have a wrapped form and R4 verbatim fallbacks where they do not.

## Steps

- [x] `formatter.rs`: add the shared helpers (`comment_text`,
      `comment_forces_break`, `list_entries`, `list_forces_wrap`, `flat_entry`,
      `wrapped_entry`, `flat_trailing`, `wrapped_trailing`) next to the comment
      helpers.
- [x] `record_components`: comment-aware flat + wrapped, `list_forces_wrap`
      forces wrapping.
- [x] `formal_params`: same for method / constructor parameter lists.
- [x] `flat_args` (verbatim fallback) + `args_wrapped` (comment-aware, forces
      wrap) and the `method_inv_ac` / `new_expr` / `overflowing_args` decisions.
- [x] `flat_arr_init` (verbatim fallback) + `array_init` (comment-aware, forces
      wrap).
- [x] `flat_ann_args` / `annotation` / `annotation_expanded` (plus the
      annotation array-element lists): comment-aware, forces expansion.
- [x] `clause_list` + `type_list_node` / `append_type_clause`: take the list
      node so comments survive; comment-aware and forces wrap.
- [x] `flat_type_args` (verbatim fallback) and `flat_formal_params` /
      `lambda` / `flat_lambda` inferred parameters (verbatim fallback).
- [x] `enum_body` + `enum_one_line_body`: preserve constant-list comments and
      force the expanded layout.
- [x] `field_decl` / `local_var`: preserve multi-declarator comments (verbatim
      fallback).
- [x] Deconstruction / `one_line_label`: a label carrying a comment must not
      collapse onto one line.
- [x] Tests: `tests/java/comments_in_lists/` golden pairs + a
      `tests/options/comments_in_lists.rs` module registered in `options.rs`.
- [x] Docs: README behaviour note and changelog entry; the three quality gates
      run green.

## Closing

Reproduction fixed (verified against the built binary): the record header and
all ten sibling contexts keep their comments on their own line with no stray
separator, and a second pass parses cleanly and re-formats to itself. Eleven
new golden pairs (`comments_in_lists`) registered in `options.rs`;
`cargo test --workspace` is green (831 core + 18 cli, plus the GUI suite),
clippy `-D warnings` and `cargo fmt --check` clean. Type arguments, lambda
parameters, multi-declarator lists and a commented record pattern use the
disclosed R4 verbatim fallback (no wrapped form). Changes uncommitted in the
worktree (no commit hash); changelog entry added 2026-09-11.
