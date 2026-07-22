---
type: ChangeRequest
kind: feature
title: Keep simple try/catch/finally and synchronized bodies on one line
description: Make KEEP_SIMPLE_BLOCKS_IN_ONE_LINE apply to try/catch/finally and synchronized blocks.
state: done
verified: { by: maintainer, at: 2026-09-03T09:32:00Z }
priority: low
tags: [dev, formatter]
owner: maintainer
---

# Problem

`KEEP_SIMPLE_BLOCKS_IN_ONE_LINE` currently applies to `if` / `else` / `for` /
`while` / `do`, but `try` / `catch` / `finally` and `synchronized` bodies are
always rendered multi-line (README limitation). A single-statement `try` or
`synchronized` body is therefore formatted differently from a single-statement
`if` body even though IntelliJ applies the same keep-simple-blocks preference
to them.

# Proposal

Extend the simple-block one-line handling to `try` / `catch` / `finally`
bodies and `synchronized` bodies: when `KEEP_SIMPLE_BLOCKS_IN_ONE_LINE` is
set and the body holds exactly one statement, render it on one line with the
block; otherwise keep the current multi-line layout. Where IntelliJ's own
behaviour differs for nested combinations (`try` with a one-line body but a
multi-line `catch`, and similar), IntelliJ's layout is the target.

Docs touched: `README.md` (honoured-options table and behaviour notes),
`docs/requirements.md` (R12), `docs/dev/changelog.md` on completion.

# Decisions

- **Follow the existing simple-block machinery.** The same option, the same
  one-statement test, and the same brace style are reused rather than adding a
  parallel mechanism.
- **Only simple bodies collapse.** Bodies with more than one statement, or
  containing comments, keep today's multi-line output.

# Acceptance criteria

- With `KEEP_SIMPLE_BLOCKS_IN_ONE_LINE` set, a `try { singleStatement }`
  renders on one line, as do single-statement `catch` and `finally` blocks
  and a single-statement `synchronized` body, matching the golden fixture
  output.
- With the option off (or a multi-statement body), output is unchanged from
  today's multi-line layout.
- Existing `control_flow` fixtures still pass (`cargo test` stays green).
- The README limitation paragraph on always-multi-line
  `try`/`catch`/`finally`/`synchronized` is removed or updated to describe the
  new behaviour.

# Implementation plan

## Approach

The existing simple-block machinery already does exactly this for `if`/`else`/
`for`/`while`/`do`: guard on `braces_style_inline() &&
keep_simple_blocks_in_one_line`, try `one_line_body` on the block, build a
candidate, accept it when `self.fits(c, &candidate)`. `one_line_body`
(src/formatter.rs L1191-1220) already rejects non-simple bodies, extra
(comment) nodes, and bodies whose rendering contains a newline — reuse it
as-is. The change is confined to `try_stmt` (L1537-1578) and `sync_stmt`
(L1580-1595).

For `try`: assemble the candidate for the whole statement —
`try { s }` plus each ` catch (P p) { h }` and ` finally { f }` clause —
using `one_line_body` on the try body and on every catch/finally body; if
_every_ body is simple and the assembled statement fits the column, return
it, otherwise fall through to today's always-multi-line rendering. This
all-or-nothing rule keeps one clean layout (a mix of one-line and multi-line
clauses is visually confusing and is not in the acceptance criteria; if real
IntelliJ output collapses clauses independently, that refinement can follow
in a later request). `try_with_resources_statement` is included: the
resources list is rendered verbatim (`self.txt`), so a one-line body simply
produces `try (r) { s } …`. For `synchronized`: candidate
`synchronized (lock) { s }` when the block is simple and fits.

The next-line-brace constraint already encoded in `braces_style_inline`
applies unchanged: with `other_brace_style` NextLine/NextLineIfWrapped the
option must not collapse these bodies, matching the existing
`keep_simple_blocks_ignores_next_line_brace_style` test.

## Steps

- [x] In `try_stmt`, compute the one-line candidate from the try body and
      each catch/finally body via `one_line_body`; return it when all bodies
      are simple and the whole fits, before the multi-line path (AC1).
- [x] In `sync_stmt`, add the analogous `synchronized (lock) { s }` candidate
      (AC1).
- [x] Fixtures under tests/java/control/: `try_sync_one_line.java` with the
      option on — single-statement try/catch/finally and synchronized bodies
      collapse; assert each rendered form in tests/control_flow.rs via
      `format_with` + `assert_contains` (AC1).
- [x] Regression tests: option off leaves today's multi-line output
      unchanged; option on with a multi-statement body stays multi-line;
      option on with `other_brace_style = BraceStyle::NextLine` does not
      collapse (AC2, AC3 — mirror the existing control_flow tests).
- [x] Run `cargo test`; confirm all control_flow and kitchen-sink goldens
      stay green (AC3).
- [x] Update the README behaviour notes (and options table wording if
      needed) and docs/requirements.md (R12); changelog on ship.

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
