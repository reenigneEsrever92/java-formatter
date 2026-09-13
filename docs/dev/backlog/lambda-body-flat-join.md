---
type: ChangeRequest
kind: bug
title: A `//` comment or multi-line statement in a lambda body swallows or corrupts the following statement when formatted
description: When a lambda with a block body is rendered as a call argument, the flat/one-line rendering joins every body child onto one line — a `//` line comment then comments out the rest of the line (silently deleting the following statement's header and unbalancing the braces), multi-line control-flow statements leak their raw source into the "flat" call text, multi-statement bodies gain doubled semicolons, and KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE collapses multi-statement bodies its own contract says are never collapsed.
state: done
priority: high
tags: [dev, bug, lambda, comments]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-13T13:48:01Z
---

# Problem

A lambda with a block body passed as a method-call argument is rendered
through the "flat" machinery, which joins every block child onto one line.
When the block holds a `//` comment, the comment ends up on the same line as
the following statement — and because `//` comments out the rest of its line,
the statement's header silently disappears from the output. A comment
followed by an `if` formats as `{ // keep new items; if (!cond) { … } }`: the
`if` header is swallowed, the body runs unconditionally, and the braces no
longer balance — valid Java becomes invalid, corrupted Java with no warning,
violating the never-corrupt contract.

The same flat join corrupts in two further ways. `flat()` falls through to the
raw source text for statement kinds it does not model (`if_statement`,
`for_statement`, …), so a multi-line control-flow statement leaks its raw
source (with the source's own indentation) into the "flat" call line — and
`args_wrapped`'s `fits()` check measures only the *last* line, so the
multi-line text is emitted unchanged. And the `"; "` join doubles the
semicolon of every self-terminating statement (`{ use(x);; more(x); }`).

Root cause: `flat_block` (~L10294 in `crates/core/src/formatter.rs`) joins all
named children with `"; "`; `flat()` returns the raw token text for comments
and unmodelled statement kinds; the `lambda()` one-line collapse
(`KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`, ~L9450) joins the same way and collapses
bodies of any statement count; and `args_wrapped` / `chain_link_args` emit the
multi-line "flat" text without a newline guard. The `one_line_body` helper for
statement blocks (`if`/`for`/`while`) already required exactly one statement —
the lambda collapse was the inconsistent outlier.

# Reproduction

Style: default (`JavaStyle::default()`).

Input:

```java
list.forEach(item -> {
    // keep only new items
    if (!persisted.contains(item.getId())) {
        add(item);
    }
});
```

Observed output (before the fix):

```java
list.forEach(item -> { // keep only new items; if (!persisted.contains(item.getId())) {
        add(item);
    } });
```

The `//` swallows the `if` header, `add(item)` runs unconditionally, and the
braces no longer balance. Without the comment, a multi-line `if` body renders
as `{ if (x) {` with the body at the source indent and a stray `} });`, and a
two-statement body renders as `{ use(x);; more(x); }` (doubled semicolon).
With `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE` on and a wide margin, a multi-statement
body collapses to one line even though the option's own description says
"single-statement" and the option test suite documents "multi-statement block
bodies are never collapsed".

Expected output — the comment on its own line, the `if` intact, the lambda
glued on the call line:

```java
list.forEach(item -> {
    // keep only new items
    if (!persisted.contains(item.getId())) {
        add(item);
    }
});
```

# Proposal

Give a block no safe one-line form when it holds a `//` comment or a
statement whose flat text carries a newline; keep such blocks multi-line in
every flat / one-line context, and render a single multi-line lambda argument
glued on the call line:

- `flat_block` returns the block's source verbatim (R4) when any child is
  unflattenable (`flat_unflattenable`: a `line_comment` or flat text with a
  newline), and joins statements with a single space — statements are
  self-terminating (`;` or `}`), so `"; "` only doubled semicolons — when it
  does flatten.
- The `lambda()` one-line collapse collapses only a single simple statement,
  matching `one_line_body` and the option's own description; commented,
  multi-line and multi-statement bodies stay multi-line.
- `args_wrapped` never emits a "flat" argument list that contains a newline; a
  single lambda argument whose body cannot flatten stays glued after `(`
  (`list.forEach(item -> { … })`, as IntelliJ lays it out) instead of being
  pushed onto its own line by the wrapped-argument layout — only when nothing
  trails the argument, so a `//` cannot swallow the glued `)`.
- Chain links (`chain_link_args`) route multi-line argument lists through
  `args_wrapped` under `CALL_PARAMETERS_WRAP = DoNotWrap` too, so a chain
  link's lambda is re-rendered at the chain's indent instead of pasted from
  the source.

# Decisions

1. **Whole class, not just the report** (agreed with the user on 2026-09-13):
   comments, control-flow statements and multi-statement bodies in lambda
   block bodies — in call-argument and chain-link positions — plus the
   `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE` collapse share one root cause (the
   flat/one-line join), so one fix covers all of them.
2. **Single multi-line lambda argument stays glued after `(`** (agreed with
   the user): IntelliJ keeps `forEach(item -> { … })` on the call line with
   the body multi-line; the wrapped-argument `(\n … \n)` layout is reserved
   for multi-argument lists. The glue only applies when nothing trails the
   argument so a `//` comment cannot swallow the glued `)`.
3. **`KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE` collapses only single-statement
   bodies** (raised by the user while reviewing the fixture): matches the
   option description ("single-statement"), the documented "multi-statement
   block bodies are never collapsed", and the sibling `one_line_body` guard
   for statement blocks.
4. **Verbatim fallback for unflattenable blocks (R4)**: a comment or
   multi-line-statement block has no flat form; echoing its source is
   consistent with `flat_args`'s existing comment-forced raw fallback and
   keeps flat contexts byte-preserving.
5. **Only `//` line comments are unflattenable, not single-line `/* */`
   block comments**: a one-line block comment joins safely (`{ /* c */ use(x);
   }`); multi-line block comments bail via the newline check.

# Acceptance criteria

- The reproduction above formats with the comment on its own line and the
  `if` header, condition and body intact; reformatting the output is a no-op
  (fixed point).
- A multi-line control-flow statement inside a lambda body in call-argument
  position keeps its multi-line layout at the correct indent — no raw source
  text, no stray braces.
- A two-statement lambda body that fits renders `{ use(x); more(x); }` (no
  doubled semicolon) and stays multi-line when it does not fit.
- Under `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`, a single-statement body collapses
  while a commented body and a multi-statement body stay multi-line; the
  output is a fixed point.
- A chain link whose argument is a comment lambda re-renders the lambda at
  the chain's indent; `list.forEach(item -> { … })` stays glued on the call
  line.
- New goldens under `tests/java/lambda_body_comments/` (comment-before-`if`,
  multi-statement / control-flow / block-comment bodies, chain-link lambda,
  one-line collapse) wired via `tests/options/lambda_body_comments.rs`; the
  existing 879 tests stay green.
- `cargo test --workspace`, `cargo clippy --workspace` and
  `cargo fmt --all -- --check` are all clean.

# Implementation plan

## Approach

All changes are confined to `crates/core/src/formatter.rs` plus golden
fixtures and tests.

**`flat_unflattenable`** (new helper, next to `flat_block`): a block child has
no safe flat rendering when it is a `line_comment` (it would comment out
everything joined after it, including the closing `}`) or its flat text
contains a newline (control flow, wrapped text). Single-line `/* */` block
comments stay flattenable.

**`flat_block`**: when any child is unflattenable, return `self.txt(node)`
verbatim (R4); otherwise join the statements' flat texts with a single space
instead of `"; "`.

**`lambda()` collapse**: the one-line branch now requires `stmts.len() == 1`
and a flattenable statement; everything else falls to the multi-line `block()`
layout.

**`args_wrapped`**: the flat shortcuts (`!keep && !forces && fits(...)` and
the `DoNotWrap` return) additionally require the flat text to contain no
newline (`flat_ok`). A new single-argument branch glues a lambda whose flat
text is multi-line right after `(` — `expr(arg, indent, c + 1 + pad)` wrapped
in the call parens — guarded by `!keep && !forces`, no trailing comments on
the argument, and no list-level trailing comments.

**`chain_link_args`**: the `DoNotWrap` fast path now only applies when the
flat argument list contains no newline; multi-line argument lists route
through `args_wrapped` so the lambda is re-rendered at the chain indent.

**Tests**: a new `lambda_body_comments` golden suite pinning the reproduction
and each symptom — `comment_if` (comment before `if`), `multi_statement`
(multi-statement collapse, control flow, block comment), `chain` (chain-link
lambda), `simple_collapse` (single-statement collapse under
`KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`) — each a fixed point (reformatting the
output is a no-op).

## Steps

- [x] `flat_unflattenable`: line-comment / newline-in-flat predicate.
- [x] `flat_block`: verbatim fallback for unflattenable blocks; `" "` join.
- [x] `lambda()` collapse: single simple statement only.
- [x] `args_wrapped`: newline guard on the flat shortcuts; single-multi-line-
      lambda-argument glued after `(`.
- [x] `chain_link_args`: route multi-line argument lists through
      `args_wrapped` under `DoNotWrap`.
- [x] New `lambda_body_comments` suite (option file + fixtures + `options.rs`
      wiring), goldens generated with the CLI and byte-checked by eye,
      idempotency asserted in-test.
- [x] Quality gates: `cargo test --workspace` (883 core integration tests —
      four new — plus nine core unit, 18 CLI, 6 GUI), `cargo clippy
      --workspace`, `cargo fmt --all -- --check` — all green.

## Closing

Shipped on 2026-09-13. Lambda block bodies with a `//` comment or a
multi-line statement are no longer flat-joined: `flat_block` echoes them
verbatim in flat contexts and joins flattenable statements with a single
space; the `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE` collapse requires exactly one
simple statement; `args_wrapped` never emits multi-line "flat" text and glues
a single multi-line lambda argument after `(` (`list.forEach(item -> { … })`);
and `chain_link_args` re-renders multi-line chain-link arguments at the
chain's indent under `CALL_PARAMETERS_WRAP = DoNotWrap`. Fixtures: the new
`tests/java/lambda_body_comments/` suite (comment-before-`if`, multi-statement
/ control-flow / block-comment bodies, chain-link lambda, one-line collapse)
pins each symptom as an idempotent golden, wired through
`tests/options/lambda_body_comments.rs`. Verified with `cargo test --workspace`
(883 core integration tests — four new — plus nine core unit and 18 CLI + 6
GUI tests), `cargo clippy --workspace`, and `cargo fmt --all -- --check`. The
changes are uncommitted in the worktree (no commit hash); the changelog entry
was added on 2026-09-13.
