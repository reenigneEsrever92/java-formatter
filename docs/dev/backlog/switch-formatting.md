---
type: ChangeRequest
kind: feature
title: Format switch statements and switch expressions
description: Lay out switch statements and switch expressions instead of emitting their original source text.
state: done
verified: { by: maintainer, at: 2026-09-03T08:57:07Z }
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

`switch` statements and switch expressions are emitted as their original
source text (README limitation). Inside a reformatted method this leaves the
switch body with whatever indentation and spacing it had in the input, which
breaks formatting consistency (R3) even though the tool is technically safe
(R4): indentation around the construct is often wrong after the surrounding
code is reformatted.

# Proposal

Model switch statements and switch expressions in the formatter: emit the
header (`switch (selector)`), lay out the body per the configured brace and
indentation style, indent `case`/`default` labels and their statements
consistently, and preserve `case` labels, colon versus arrow forms, and all
contained expressions and statements — which are formatted recursively by the
existing machinery. Only whitespace/layout decisions are made by this change.

Docs touched: `README.md` (limitations), `docs/requirements.md` (R11),
`docs/dev/changelog.md` on completion.

# Decisions

- **Layout, not analysis.** The change normalises whitespace and indentation
  only; it never reorders, merges, or analyses cases, and keeps colon and
  arrow (`case x -> ...`) forms exactly as written.
- **Safe default stays.** Until the construct is modelled, and for any
  switch-shaped input the parser still flags as an error, the verbatim-echo
  path (R4) remains the fallback.

# Acceptance criteria

- A `switch` statement inside a reformatted method is indented consistently
  with its surrounding code and its case bodies are formatted (statements
  laid out per style), matching the `*.out.java` golden in the fixture suite.
- A switch expression (including one used as an assignment RHS or return
  value) is formatted likewise.
- `case` labels, `default`, colon forms, and arrow forms are preserved
  verbatim apart from whitespace around them.
- No existing fixture regresses (`cargo test` stays green) and a switch
  fixture that is not yet fully modelled still round-trips without losing
  tokens.
- The README's "switch statements and switch expressions are emitted as their
  original source text" limitation is removed.

# Implementation plan

## Approach

Today both switch forms are verbatim echoes: `Fmt::switch_stmt`
(src/formatter.rs L1610-1614) and the `"switch_expression"` arm of `Fmt::expr`
(L1679). The work is to replace the echo with a layout pass that formats the
header, then the case groups, indenting consistently with the surrounding
code. Before implementing, the exact tree-sitter-java node/field names for
switch constructs must be confirmed — the grammar distinguishes colon-style
`switch_block_statement_group` nodes from arrow-style `switch_rule` nodes and
labels from statements. Add a temporary scratch test (or `eprintln!` in a
throwaway unit test) that parses a representative switch and prints the CST
node kinds and field names, then delete it; the codebase's `sync_stmt`
(L1580-1595) is the precedent for locating children by kind when field names
are unavailable, and anything not modelled falls back to `self.txt` (R4).

Rendering plan: `switch (selector)` on the header line with the selector
rendered through `expr`; body braces follow the statement-block convention.
Inside the body, indent `case`/`default` labels one level and their statements
a further level, matching IntelliJ's default layout; colon groups get their
statements formatted via `self.stmt` (blank-line preservation between
statements is a nice-to-have and can wait); arrow rules render as
`case X ->` followed by the single statement/block. Switch _expressions_
(statement position, assignment RHS, return value, argument) render on one
line when the whole expression fits the current column (reusing `fits`), else
fall back to the same multi-line body layout; when a switch expression is
embedded in a larger flat context that cannot contain newlines, keep the
current verbatim fallback rather than risk corrupt output (R4/R5).

Decisions recorded: layout-only change — labels, colon vs arrow form,
`default`, and case expressions are preserved except for whitespace; no
case analysis, merging, or reordering; `case`-indentation sub-options of
IntelliJ (`INDENT_CASE_FROM_SWITCH`, etc.) are out of scope for this request.

## Steps

- [x] Confirm the CST shape: temporary scratch test parses a sample switch
      (colon + arrow + expression) and prints node kinds/fields; record the
      names in the final implementation comments and delete the scratch test.
- [x] Implement `switch_stmt` layout: header, brace placement, label/body
      indentation, colon groups via `stmt`, arrow rules, `default`; unknown
      shapes fall back to `self.txt` (R4).
- [x] Route `switch_expression` through the layout, one-line when it fits its
      context column, multi-line otherwise (R5 preserved).
- [x] Fixtures under tests/java/control/: `switch_basic.java` (canonical
      colon switch, must format unchanged), `switch_messy.java` +
      `switch_messy.out.java` (bad indentation normalised), and a
      `switch_expression.java` fixture covering expression positions (AC1,
      AC2). Assert `case`/`default`, colon and arrow forms survive verbatim
      apart from whitespace (AC3).
- [x] Add assertions to tests/control_flow.rs wiring the fixtures through
      `format`/`format_with`; add an idempotency assertion on the messy
      fixture (AC3, AC5).
- [x] Run `cargo test`; all existing goldens must stay green, confirming the
      verbatim fallback still covers any unmodelled switch shape (AC4).
- [x] Update the README (remove the limitation, note switch layout in
      behaviour notes) and docs/requirements.md (R11); changelog on ship.

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
