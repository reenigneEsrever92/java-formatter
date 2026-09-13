---
type: ChangeRequest
kind: bug
title: A comment inside an if/while/do/synchronized/switch condition or a parenthesized expression is taken for the whole inner expression — the real condition vanishes and a `//` comment swallows the closing paren
description: When a comment sits inside a keyword condition (if, while, do-while, synchronized, switch) or a parenthesized expression, the formatter renders the comment as the entire inner expression: the actual condition is dropped, the closing `)` and the following `{`/`;` land inside the `//` comment and are swallowed, and the output is invalid Java; comments in trailing or mid-expression positions are silently deleted.
state: done
verified:
  by: maintainer
  at: 2026-09-13T18:30:00Z
priority: high
tags: [dev, bug, comments, formatter]
owner: maintainer
---

# Problem

The keyword-condition renderers in `crates/core/src/formatter.rs` —
`keyword_cond` and `flat_keyword_cond` — destructure a condition's
`parenthesized_expression` and render `named_child(0)` as "the inner
expression". Tree-sitter attaches comments as named extras _inside_ the parens,
so when a comment precedes the real expression, `named_child(0)` is the
**comment**, not the expression. The renderer then emits only the comment,
wraps it in `(…)`, and glues the closing `)` (and the statement's following `{`
or `;`) directly after the comment text. For a `//` line comment the `)` and
everything after it on that line are swallowed into the comment: the real
condition silently disappears and valid Java becomes invalid (`missing ')'`,
reported by the formatter's own parse-error detection on a second pass) with no
warning on the first — a direct violation of the never-corrupt contract (R5/R6).

The same `named_child(0)` pattern serves `if`, `while`, `do-while`,
`synchronized`, `switch` (both the multi-line `switch_stmt` and the flat
`switch_one_line`) and the generic `parenthesized_expression` arms of `expr_ac`
and `flat`. Comments in the _other_ positions inside a condition are not
corrupted but silently dropped: a leading `/* … */` block comment yields
`if (/* c */)` (the condition still vanishes — an empty condition is invalid
Java), a trailing comment yields `if (s.contains(a))`, a mid-expression comment
yields `(x + y)`, and `(x + // c\n y)` renders `(x + y)`. R4/R5 (comments echoed
verbatim, only whitespace/layout may change) are violated in every variant.

# Reproduction

Style: the repository's `codestyle.xml` (`LINE_COMMENT_AT_FIRST_COLUMN=false`)
or the built-in default.

Input (the report, rewritten with neutral names):

```java
if (
    // the entry may already be in the set we are rebuilding
    // and we only want to add it once
    entries.stream()
        .noneMatch(entry -> entry.id() == id)
) {
    entries.add(entry);
}
```

Observed output: the condition and the second comment vanish; the first comment
is glued after `if (` (indented to the condition column) and the closing `)` +
`{` land inside it:

```java
if (        // the entry may already be in the set we are rebuilding) {
    entries.add(entry);
}
```

The reported case shows the same shape one nesting level deeper (12 spaces).
Formatting the output a second time reports `missing ')' at …` and `parse
error` — the emitted text is not valid Java. The siblings reproduce with the
built-in defaults too:

| Construct                                     | Observed                                      |
| --------------------------------------------- | --------------------------------------------- |
| `if` / `while` / `synchronized`, leading `//` | condition vanishes, `) {` swallowed → invalid |
| `switch`, leading `//`                        | condition vanishes, `) {` swallowed → invalid |
| `do`–`while`, leading `//`                    | condition vanishes, `);` swallowed → invalid  |
| `if (/* c */ x)`, leading block comment       | `if (/* c */)` — empty condition → invalid    |
| `if (x /* trailing */)`                       | `if (x)` — comment dropped                    |
| `if (x > /* mid */ y)`                        | `if (x > y)` — comment dropped                |
| `(x + // c\n y)` parenthesized                | `(x + y)` — comment dropped                   |

Expected: both comments preserved on their own lines at the continuation
indent, the real condition rendered intact after them, the `)` alone on its own
line, and the output a fixed point:

```java
if (// the entry may already be in the set we are rebuilding
    // and we only want to add it once
    entries.stream()
        .noneMatch(entry -> entry.id() == id)
) {
    entries.add(entry);
}
```

# Proposal

Root-cause fix confined to `crates/core/src/formatter.rs`: the condition
renderers stop treating `named_child(0)` of the `parenthesized_expression` as
the whole inner expression and become comment-aware, following the design the
comma-separated-lists fix established (`comment_text`, `comment_forces_break`,
own-line comment placement):

- A shared comment-aware keyword-condition renderer separates the paren's
  comment extras from the real expression (the first non-comment named child),
  renders the expression normally, and lays each comment out safely: a `//`
  line comment or multi-line `/* … */` goes on its own line at the
  continuation indent and forces the wrapped condition layout; a single-line
  `/* … */` may stay inline before the expression. A `//` comment in the
  condition disables the flat / one-line collapse candidates (`if_one_line`,
  the `while` / `do` one-line bodies, `switch_one_line`) the way
  `list_forces_wrap` forces the wrapped list form.
- The generic `parenthesized_expression` arms of `expr_ac` and `flat` get the
  same treatment.
- Where no safe layout exists — a comment nested deeper inside the condition
  (e.g. between operands of a binary expression, which the expression
  renderers do not model) or any flat context that cannot carry an own-line
  comment — the construct echoes its source verbatim (R4), the escape hatch
  already applied to commented flat lists, type arguments, lambda parameters
  and multi-declarator lists.
- Gated on comment presence: uncommented conditions render byte-identically to
  today, so existing goldens stay green.

Docs touched on delivery: `docs/requirements.md` gains a requirement row for
comment-in-condition preservation (as the list fix added for comment-in-list),
the README formatting-behaviour notes describe the new comment layout, and
`docs/dev/changelog.md` records the fix.

# Decisions

1. **Whole family in scope (user choice, 2026-09-13).** All five keyword
   conditions — `if`, `while`, `do-while`, `synchronized`, `switch` (statement
   and one-line forms) — plus generic parenthesized expressions share the
   `named_child(0)`-picks-a-comment root cause; all are fixed together.
2. **Root-cause comment-aware rendering, not a blanket R4 fallback (user
   choice).** A leading `//` comment gets its own line at the continuation
   indent and forces the wrapped condition layout; the reported case is laid
   out, not echoed verbatim.
3. **Expected layout confirmed (user).** Comment lines at the continuation
   indent, the condition after them, the closing `)` alone on its own line.
4. **R4 verbatim fallback only where no safe layout exists.** Comments nested
   deeper inside the condition (the expression renderers cannot carry them)
   and flat contexts (no own-line comment possible) echo the construct's
   source verbatim instead of dropping the comment or corrupting the output —
   the established contract for unmodelled / flat constructs.
5. **No option change.** The fix is gated on comment presence only, so default
   and absent schemes keep today's output byte-for-byte wherever no condition
   comment exists.

# Acceptance criteria

- The reported reproduction formats to valid Java with both comments preserved
  on their own lines at the continuation indent, the real condition intact
  after them, and the `)` on its own line; a second pass parses cleanly and
  re-formats to itself (R6).
- One golden pair per affected construct under `tests/java/` asserting the
  comment is preserved, the condition intact, and the output valid and
  idempotent: `if`, `while`, `do-while`, `synchronized`, `switch`, and a
  generic parenthesized expression — wired via `tests/options/`.
- A `//` comment in a keyword condition forces the wrapped / own-line layout
  even when the statement would fit the margin; a single-line `/* … */` may
  stay inline before the expression.
- Comments in trailing and mid-expression positions are preserved (by layout
  or the R4 verbatim fallback), never silently dropped.
- No existing golden changes; `cargo test --workspace`, `cargo clippy
--workspace --lib --bins --tests -- -D warnings` and
  `cargo fmt --all -- --check` are clean; `docs/requirements.md`, the README
  and `docs/dev/changelog.md` are updated.

# Implementation plan

## Approach

All changes are confined to `crates/core/src/formatter.rs` plus golden
fixtures and tests. The condition renderers stop treating
`named_child(0)` of the `parenthesized_expression` as the whole inner
expression and become comment-aware, reusing the comment helpers the
comma-separated-lists fix established (`comment_text`,
`comment_forces_break`, `is_comment_node`).

**Helpers** (next to the list-comment helpers):

- `subtree_has_comment(node)` — recursive child walk; true when any
  comment node appears anywhere under `node` (comments nested inside the
  condition expression, which the expression renderers do not model).
- `paren_has_comment(node)` — a `parenthesized_expression` carries a
  comment as a direct extra (before or after the inner expression) or
  nested inside its inner expression.
- `commented_parens(leading, trailing, inner, indent, pad)` — wrapped
  layout: the first leading comment glued right after `(` (the layout the
  report confirmed), every other comment and the expression each on their
  own line at `cont(indent)`, trailing comments after the expression, and
  the closing `)` alone at `ind(indent)` — never a `//` next to following
  code, so nothing is swallowed. When only single-line `/* … */` comments
  are present they stay inline (joined with spaces, passed through
  `within` so the `SPACE_WITHIN_*` pad toggle still applies).

**`keyword_cond`** (if / while / do-while / synchronized / switch):
no comment (direct or deep) → byte-identical to today; a comment nested
inside the expression → the whole condition echoes its source verbatim
(R4 — the expression renderers cannot carry it); direct comments →
`commented_parens`.

**`flat_keyword_cond`** (one-line collapse candidates — `if_one_line`,
the while / do / synchronized one-line bodies, `switch_one_line`): a
comment present → return the condition's verbatim source (multi-line
conditions bail via the callers' `contains('\n')` checks; a single-line
`/* … */` condition still collapses safely with the comment inline). No
call-site changes needed.

**`parenthesized_expression`**: the `expr_ac` arm gets the same
comment-aware treatment as `keyword_cond` (existing align / lparen /
rparen logic untouched for uncommented parens); the `flat` arm echoes
the paren verbatim (R4) when a comment is present.

Gated on comment presence throughout: uncommented conditions and parens
render byte-identically to today, so the existing goldens stay green.

## Steps

- [x] `formatter.rs`: add `subtree_has_comment`, `paren_has_comment` and
      `commented_parens` next to the list-comment helpers.
- [x] `keyword_cond`: comment-aware — direct comments via
      `commented_parens`, deep comment → verbatim (R4), else unchanged.
- [x] `flat_keyword_cond`: comment guard → verbatim (single-line `/* */`
      still collapses safely).
- [x] `expr_ac` / `flat` `parenthesized_expression` arms: comment-aware
      rendering / R4 echo.
- [x] Tests: new `tests/java/commented_conditions/` golden suite (if,
      while, do-while, synchronized, switch, parenthesized expression,
      trailing and mid-expression comments) + `tests/options/
  commented_conditions.rs` registered in `options.rs`; goldens
      generated with the CLI, byte-checked by eye, idempotency asserted
      in-test.
- [x] Docs: README formatting-behaviour note, `docs/requirements.md`
      requirement row, `docs/dev/changelog.md` entry.
- [x] Quality gates: `cargo test --workspace`, `cargo clippy --workspace`,
      `cargo fmt --all -- --check` — all green.

## Closing

Shipped on 2026-09-13. The condition renderers (`keyword_cond`,
`flat_keyword_cond`) and the `parenthesized_expression` arms of `expr_ac` /
`flat` no longer take `named_child(0)` for the whole inner expression: a
break-forcing comment gets its own line at the continuation indent (the first
one glued right after `(`), the real condition follows, the `)` sits alone on
its line, a single-line `/* … */` stays inline, a comment nested inside the
condition's expression keeps the whole paren verbatim (R4), and the one-line
collapses bail on a commented condition. Fixtures: the new
`tests/java/commented_conditions/` suite (if / while / do-while /
synchronized / switch leading comments, a mid-expression parenthesized
expression, a trailing block comment, a binary-operand comment) pins each
symptom as an idempotent golden, wired through
`tests/options/commented_conditions.rs`. Verified with `cargo test --workspace`
(891 core integration tests — eight new — plus nine core unit and 18 CLI + 6
GUI tests), `cargo clippy --workspace --lib --bins --tests -- -D warnings`, and
`cargo fmt --all -- --check`. The changes are uncommitted in the worktree (no
commit hash); the changelog entry was added on 2026-09-13.
