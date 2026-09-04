---
type: ChangeRequest
kind: feature
title: Honour the switch/case indentation and wrapping options
description: Implement the case-label indentation options and SWITCH_EXPRESSIONS_WRAP on top of the shipped switch layout.
state: planned
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

# Implementation plan

## Approach

**Configuration (crates/core/src/config.rs).** Add four fields to `JavaStyle`
with their `Default` values — three plain bools (`indent_case_from_switch`,
`case_statement_on_new_line`, `indent_break_from_case`, all `true`) and
`switch_expressions_wrap: WrapStyle` defaulting to `WrapStyle::WrapIfLong`
(IntelliJ code `1`) — and declare the four `OptionDef` entries in the `OPTIONS`
registry, all in `Section::CodeStyleJava` (the
`<codeStyleSettings language="JAVA">` block the docs tables document). The
bools use `OptionValue::Bool`, the wrap reuses `OptionValue::Wrap` (Decision:
Encodings); the `get`/`set` closures clone the neighbouring entries and parsing
needs no new code because `parse_codestyle` / `serialize_codestyle` and the GUI
are registry-driven, so the existing round-trip coverage extends to the new
entries automatically. Because the recorded defaults equal the IntelliJ
built-ins the shipped switch layout already matches (Decision: Defaults),
absent-from-scheme options and the default style keep today's byte-identical
output and no existing golden moves.

**Rendering — case layout (crates/core/src/formatter.rs).** `switch_stmt`
(L1813) currently hardcodes `inner = indent + 1` and passes it to
`switch_group` / `switch_rule` / the comment fallback. Replace that with two
computed levels — `label_level = indent + 1 if indent_case_from_switch else
indent`, `statement_level = label_level + 1` — threaded through in place of the
single `inner`. `switch_group` (L1852) renders labels at `label_level` and
statements at `statement_level`; when `case_statement_on_new_line` is false the
group's first statement renders on the label line (`case 1: foo();`, following
statements still on their own lines at `statement_level`);
`break_statement` / `continue_statement` / `return_statement` children (node
kind match in the group loop, before the `stmt` call) render at `label_level`
when `indent_break_from_case` is false. `switch_rule` (L1871) labels move to
`label_level` as well; arrow rules are unaffected by the other two options
(their bodies are expressions / blocks / throws, never label-line statements),
and `switch_stmt`'s comment/stray-node fallback follows `label_level`. All
changes are whitespace-only (R5); unmodelled shapes keep the verbatim echo
(R4); with the defaults (`true` / `true` / `true`) every branch renders exactly
as today.

**Rendering — expression wrapping.** `switch_expr` (L1913) replaces its
hardcoded fits-based choice with a `switch_expressions_wrap` decision:
`DoNotWrap` (0) always uses the one-line form when `switch_one_line` produces
one (multi-line fallback only when it cannot); `WrapIfLong` (1, the default)
keeps the current fits-based choice — byte-identical to the shipped behaviour;
`WrapAlways` (2) always uses the multi-line `switch_stmt` layout; `ChopDownIfLong`
(5) wraps when long and additionally breaks an overflowing nested construct in
the body (e.g. a nested switch expression reached through the recursive
`stmt` / `expr` machinery) — the distinguishing chop-down output is pinned by a
golden and, per the binary-expression-wrapping precedent, cross-checked against
IntelliJ if one is available. Statement-position switches (`stmt` →
`switch_stmt`, L1396) are untouched — the option governs switch expressions
used as values only — and the `flat` arm (L2864) keeps its one-line-or-verbatim
echo (flat contexts cannot contain newlines), so `switch_one_line` is unchanged.

**Tests.** Per the AGENTS.md hard rules: four new option files under
`crates/core/tests/options/` (`indent_case_from_switch.rs`,
`case_statement_on_new_line.rs`, `indent_break_from_case.rs`,
`switch_expressions_wrap.rs`), wired into `tests/options.rs` via
`#[path = "options/<name>.rs"] mod <name>;`, fixtures under
`tests/java/<option>/` referenced by relative `include_str!`. Every test is a
golden pair asserted with `format_with` (or `format` for the default-style
check); each file covers its interesting values plus an absent-option default
check that asserts the shipped layout; each new golden is asserted idempotent
by reformatting the golden and comparing it to itself (the
`assert_idempotent` helper no longer exists — AGENTS removed it).

**Docs.** On delivery: flip the four rows in `docs/settings/common.md` (three
in "Braces & indentation", `SWITCH_EXPRESSIONS_WRAP` in "Wrapping & braces →
Expressions and statements") from ❌ to ✅; add the four options to the README
honoured-options table and update the switch bullet in _Formatting behaviour
notes_; add a requirement row (R16) and a milestones note to
`docs/requirements.md`; append a `docs/dev/changelog.md` entry (newest first).

## Steps

- [ ] crates/core/src/config.rs: add the four `JavaStyle` fields + `Default`
      values (bools `true`, wrap `WrapStyle::WrapIfLong`) and the four `OPTIONS`
      entries (`Section::CodeStyleJava`; three `OptionValue::Bool`, one
      `OptionValue::Wrap`); the registry-driven parse/serialize and GUI pick
      them up with no further code. Verify the crate builds and the existing
      suite stays green (defaults match the shipped layout) (AC: config
      mapping, defaults).
- [ ] crates/core/src/formatter.rs: compute `label_level` / `statement_level`
      in `switch_stmt` from `indent_case_from_switch` and thread them through
      `switch_group`, `switch_rule` and the comment fallback, so labels outdent
      to the switch indent when the option is false (AC2: `INDENT_CASE_FROM_SWITCH`).
- [ ] crates/core/src/formatter.rs: `switch_group` — put the group's first
      statement on the label line when `case_statement_on_new_line` is false,
      and indent `break` / `continue` / `return` children at `label_level` when
      `indent_break_from_case` is false (AC2: `CASE_STATEMENT_ON_NEW_LINE`,
      `INDENT_BREAK_FROM_CASE`).
- [ ] crates/core/src/formatter.rs: `switch_expr` — decide one-line vs
      multi-line per `switch_expressions_wrap` (0 always one-line when a
      one-line form exists, 1 fits-based default unchanged, 2 always
      multi-line, 5 wrap-if-long plus chop-down of overflowing nested
      constructs, pinned by golden and cross-checked with IntelliJ if
      available); statement-position switches and the `flat` arm unchanged
      (AC3).
- [ ] Tests: create `crates/core/tests/options/indent_case_from_switch.rs`,
      `case_statement_on_new_line.rs`, `indent_break_from_case.rs` and
      `switch_expressions_wrap.rs` with golden pairs under
      `tests/java/<option>/` (per-value goldens — the three case options at
      `false` + a `format()` absent-option default check each; the wrap option
      at codes 0/1/2/5 plus a short-expression wrap-always case and a default
      check), wire them into `tests/options.rs`, and assert each new golden is
      idempotent by reformatting it (AC1, AC2, AC3, AC4).
- [ ] Run `cargo test`: all existing goldens stay green (defaults
      byte-compatible) and the four new option files pass (AC4).
- [ ] Docs: flip the four `docs/settings/common.md` rows to ✅; add the four
      options to the README honoured-options table and update the switch
      _Formatting behaviour notes_ bullet; add requirement row R16 to
      `docs/requirements.md` and touch the milestones paragraph; append a
      `docs/dev/changelog.md` entry; run `cargo test` once more to confirm the
      suite is green (AC5).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
