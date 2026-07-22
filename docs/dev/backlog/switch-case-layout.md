---
type: ChangeRequest
kind: feature
title: Honour the switch/case indentation and wrapping options
description: Implement the case-label indentation options and SWITCH_EXPRESSIONS_WRAP on top of the shipped switch layout.
state: proposed
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

`INDENT_CASE_FROM_SWITCH`, `CASE_STATEMENT_ON_NEW_LINE`, `INDENT_BREAK_FROM_CASE`
and `SWITCH_EXPRESSIONS_WRAP` are valid IntelliJ options marked ❌ in
docs/settings/common.md ("Braces & indentation" and "Wrapping & braces") and
safely ignored per R7, so a scheme that sets them is only partially honoured
and output diverges from IntelliJ for the affected constructs. Switch
statements and expressions are already formatted (shipped switch-formatting
change request) with a fixed layout: `Fmt::switch_stmt` indents `case` /
`default` labels one level and their statements a further level, `switch_group`
keeps `break` / `continue` / `return` at the statement indent, and a switch
expression used as a value stays on one line when it fits the margin (README
formatting note) — none of that is configurable.

# Proposal

Parse each listed option into a `JavaStyle` field via the `OPTIONS` registry in
crates/core/src/config.rs (`Section::CodeStyleJava`, IntelliJ built-in defaults
from the tables: the three case options `true` / `true` / `true`,
`SWITCH_EXPRESSIONS_WRAP` `1` = wrap if long, a `WrapStyle`);
absent-from-scheme options keep the default. Apply them in
crates/core/src/formatter.rs at the constructs they govern:
`INDENT_CASE_FROM_SWITCH` controls whether labels are indented from the
`switch`, `CASE_STATEMENT_ON_NEW_LINE` whether the statement after a label
starts a new line, `INDENT_BREAK_FROM_CASE` the indentation of `break` /
`continue` / `return` relative to the label, and `SWITCH_EXPRESSIONS_WRAP`
governs when a switch expression used as a value wraps.

Docs touched: on delivery the implementation updates the docs/settings support
marks (❌ → ✅ for these rows), the README honoured-options table /
formatting-behaviour notes, docs/requirements.md (a new requirement row), and
docs/dev/changelog.md.

# Decisions

- **One family, one request.** Only the four listed options are added here;
  the other unimplemented wrapping rows stay out and are safely ignored (R7).
- **Defaults.** The registry records the IntelliJ built-ins from the tables;
  the shipped switch layout already matches them (labels indented, statements
  on a new line, `break` indented from the label, expressions wrap if long), so
  default and absent-from-scheme styles keep current byte-identical output and
  existing goldens stay green.
- **Semantics.** Whitespace/layout only (R5); unmodelled switch shapes still
  fall back to the verbatim echo (R4); formatting formatted output is a no-op
  (R6).
- **Encodings.** The three case options are plain bools;
  `SWITCH_EXPRESSIONS_WRAP` reuses the existing `OptionValue::Wrap` mapping
  (wrap codes per docs/settings/index.md) — no new registry types.

# Acceptance criteria

- A dedicated golden fixture + test file per option following the pattern in
  crates/core/tests/options/, each option tested at its interesting values plus
  an absent-option default check.
- `INDENT_CASE_FROM_SWITCH` = `false` puts labels at the switch indent;
  `CASE_STATEMENT_ON_NEW_LINE` = `false` keeps the first statement on the label
  line; `INDENT_BREAK_FROM_CASE` = `false` aligns `break` / `return` with the
  label.
- `SWITCH_EXPRESSIONS_WRAP` wraps a long switch expression used as a value per
  its wrap code (`0` never, `1` when long, `2` always, `5` chop down).
- Default/absent schemes produce today's layout and the suite stays green
  (`cargo test`); new goldens are idempotent.
- docs/settings marks flipped to ✅; README and docs/requirements.md updated.
