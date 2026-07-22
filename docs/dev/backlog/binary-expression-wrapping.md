---
type: ChangeRequest
kind: feature
title: Wrap binary expressions per BINARY_OPERATION_WRAP
description: Implement binary-expression wrapping so long right-hand sides respect the configured margin.
state: done
verified: { by: maintainer, at: 2026-09-02T19:13:02Z }
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

Binary expressions are not wrapped: there is no `BINARY_OPERATION_WRAP`
handling, so a long right-hand side can exceed the configured margin even
when other constructs (calls, assignments, record components) wrap. This is
the first documented limitation in the README and the most visible fidelity
gap against IntelliJ output.

# Proposal

Parse the IntelliJ `BINARY_OPERATION_WRAP` option into `config::JavaStyle`
and teach the formatter to lay out a binary expression that does not fit
within the margin according to the wrap code (0 = do not wrap, 1 = wrap if
long, 2/5 = chop down if long, 3 = wrap always), placing the operator and
breaking the expression at its top-level operators, with continuation
indentation. Binary sub-options beyond the wrap code (for example operator
placement variants) are deliberately out of scope for this request and can be
added later.

Docs touched: `README.md` (honoured-options table, limitations),
`docs/requirements.md` (R10), `docs/dev/changelog.md` on completion.

# Decisions

- **Margin-driven like the other wraps.** Wrapping engages when the whole
  expression exceeds the margin, matching how calls and assignments already
  wrap — not line-by-line greedy breaking.
- **One option, one request.** Only `BINARY_OPERATION_WRAP` is added here;
  IntelliJ's finer binary-layout sub-options stay unimplemented and are
  ignored safely (R7).
- **R4/R5 hold.** The reformatted expression must stay semantically identical;
  where a construct is not (yet) modelled it is still echoed verbatim.

# Acceptance criteria

- A fixture whose right-hand side exceeds the margin with
  `BINARY_OPERATION_WRAP` = wrap-if-long is wrapped at its operators within
  the margin, matching the expected `*.out.java` golden.
- Wrap codes behave as documented: `0` never wraps, `1` wraps only when long,
  `2`/`5` chop down when long, `3` wraps always.
- Wrapped lines use continuation indentation and the operators are placed
  consistently with the other wrapped constructs.
- Do-not-wrap and default schemes produce unchanged (single-line) layout for
  expressions that currently stay on one line — no regression for existing
  fixtures (`cargo test` stays green).
- The README's "Binary expressions are not wrapped" limitation is removed and
  `BINARY_OPERATION_WRAP` appears in the honoured-options table.

# Implementation plan

## Approach

Two sides: configuration and rendering. In src/config.rs add a
`binary_operation_wrap: WrapStyle` field to `JavaStyle` (constructed only via
`Default`, so a new field needs no literal-site changes), default
`WrapStyle::DoNotWrap`, and parse the JAVA `codeStyleSettings` block option
`BINARY_OPERATION_WRAP` with the existing `OptionMap::get_wrap` helper
alongside `ASSIGNMENT_WRAP`. Extend tests/config.rs following its inline-XML
pattern (values 1, 2/5, 3; absent → default; non-JAVA block ignored).

Rendering: today `Fmt::binary` (src/formatter.rs L1991-2005) flattens
`left op right` and ignores its `indent`/`c` arguments, and the `flat`
machinery has its own inline binary arm (L2180-2194) used where an inline
expression is required (e.g. assignment left-hand sides). The wrapped path is
reached through `expr` → `binary` only. Rewrite `binary` on the pattern of
`ternary` (L2007-2037): render the full flat text; if `self.fits(c, &flat)`
return it; otherwise, when `binary_operation_wrap != DoNotWrap`, walk the
binary spine (a left-associative chain of `left op right` nodes) and emit one
operand per line at `self.cont(indent)`, preserving the exact token order so
semantic equivalence (R5) is guaranteed by construction — only whitespace is
inserted at operator boundaries. WrapIfLong breaks the spine only when the
segment does not fit; ChopDownIfLong (codes 2/5) additionally recurses into
an operand that is itself a binary expression when its own line overflows;
WrapAlways forces the spine break. DoNotWrap (and the default style) keep
today's single-line output, so no existing golden changes.

The operator-placement convention for wrapped lines must be pinned by a
golden: follow the codebase's existing continuation convention (operator at
the start of the continuation line, as `ternary` does for `?`/`:`) and, if an
IntelliJ installation is available to the implementer, verify the golden
against real IntelliJ output and adjust if it differs — IntelliJ's finer
`BINARY_OPERATION_SIGN_ON_NEXT_LINE` sub-option stays out of scope per the
request decisions.

## Steps

- [x] src/config.rs: add `binary_operation_wrap` field + default + parse;
      extend tests/config.rs with inline-scheme cases for values 1, 2, 5, 3
      and a missing-option default check (AC: config mapping).
- [x] Rewrite `Fmt::binary` to flatten-then-wrap per the approach; keep
      `flat`'s inline binary arm unchanged for the no-wrap default (AC4).
- [x] Add fixtures under tests/java/binary/: a long sum wrapped at its
      operators under wrap-if-long (`long_sum.java` + `long_sum.out.java`
      golden at a small `right_margin`), a do-not-wrap fixture asserted
      unchanged, a chop-down fixture with a nested binary operand, and a
      wrap-always fixture (AC1, AC2, AC3).
- [x] Register the fixtures in a new tests/binary.rs using the common
      helpers (`format_with`, `assert_idempotent`) with `WrapStyle` styles
      (AC1-AC3, AC5 idempotency).
- [x] Verify no existing suite regresses — run `cargo test` and inspect the
      diff of any changed goldens (AC4).
- [x] If IntelliJ is available, format the long-sum snippet there and align
      the operator-placement golden; record the outcome in the changelog.
- [x] Update the README (honoured-options table + remove the limitation) and
      docs/requirements.md (R10); append the changelog entry when shipped.

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
