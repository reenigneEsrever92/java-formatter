---
type: ChangeRequest
kind: refactor
title: Restructure the test suite so each option gets a dedicated test
description: Reorganize crates/core/tests so every registered formatting option has its own test file with a fixture + golden pair, making input→output correlations easy to see.
state: done
verified: { by: maintainer, at: 2026-09-03T13:14:56Z }
priority: medium
tags: [dev, tests]
owner: maintainer
---

# Problem

The integration suite in `crates/core/tests/` is grouped by _topic_
(`assignment.rs`, `binary.rs`, `control_flow.rs`, `methods.rs`, `records.rs`,
`imports.rs`, …), not by option. The link between a `JavaStyle` option and its
tests is indirect: `control_flow.rs` covers `KEEP_SIMPLE_BLOCKS_IN_ONE_LINE`
plus switch layout and condition handling, `methods.rs` covers
`KEEP_SIMPLE_METHODS_IN_ONE_LINE` plus throws preservation and header
behaviour, and `records.rs` mixes three record options. Against the 25-option
`OPTIONS` registry in `config.rs`, several options have **no dedicated
formatting-behaviour test** at all (`METHOD_CALL_CHAIN_WRAP`,
`KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`, `ANNOTATION_PARAMETER_WRAP`,
`CALL_PARAMETERS_LPAREN/RPAREN_ON_NEXT_LINE`,
`METHOD_PARAMETERS_LPAREN/RPAREN_ON_NEXT_LINE`, `INDENT_SIZE`,
`CONTINUATION_INDENT_SIZE`, `SOFT_MARGINS` — the last three only appear as
auxiliary settings in other tests). Many tests also assert with scattered
`assert_contains` checks instead of a full input→output comparison, so a
reader cannot easily see what input produces what output for a given option.

# Proposal

Restructure `crates/core/tests/` so **each registered option gets a dedicated
test file** under `tests/options/<option>.rs` (one file per `OptionDef` in the
`OPTIONS` registry, named after the XML option), each with an input fixture
and — where the output differs — a `*.out.java` golden, so the input→output
transformation for that option is visible at a glance. Existing tests are
moved, renamed and split into these per-option files; fixture directories move
with them (e.g. `tests/java/assignment/` → `tests/java/assignment_wrap/`).
Coverage gaps for the untested options above are filled with new fixtures.
Non-option behaviour — throws preservation, method/constructor headers,
generic spacing, extends/implements clauses, switch layout, parse-error
diagnostics, idempotency/kitchen-sink goldens, and the codestyle XML
parse/serialize suite — stays in its own topic files, since it does not map to
a single option.

This is a pure test restructure: no formatting, config, or library behaviour
changes.

Docs touched: `README.md` (Testing section wording),
`docs/requirements.md` (R9 maintainability wording), `docs/dev/changelog.md`
on completion.

# Decisions

- **One file per option, under `tests/options/`.** Decided with the user on
  2026-09-03: a dedicated test file per option (option A), so option→test
  correlation is direct.
- **Keep fixture + golden pairs.** Decided with the user on 2026-09-03: the
  existing pattern of real `.java` fixtures with `*.out.java` goldens
  (option A) is kept, so input→output is visible without opening the test
  body; assertions in the test files are full `assert_eq!(format(input),
golden)` comparisons rather than scattered `assert_contains` checks where a
  golden exists.
- **Scope: all formatting-behaviour tests, gaps filled.** Recommended by the
  agent, accepted by the user ("whatever you think is best"): every option in
  the `OPTIONS` registry gets a test file, including the currently untested
  ones; non-option behaviour stays in its own topic files.
- **Existing tests are moved/renamed/split, not kept alongside.** Decided with
  the user on 2026-09-03: old topic files are dissolved into the per-option
  files so there is exactly one obvious place per option.
- **No behavioural change.** Formatting output and library behaviour are
  byte-identical; the restructured suite must pass with the same results.

# Acceptance criteria

- For every option in the `OPTIONS` registry (25 entries), there is a
  `tests/options/<xml_name>.rs` test file whose tests exercise that option's
  formatting behaviour with at least one fixture (+ golden where the output
  differs), including the previously untested
  `METHOD_CALL_CHAIN_WRAP`, `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`,
  `ANNOTATION_PARAMETER_WRAP`, the four lparen/rparen-on-next-line options,
  `INDENT_SIZE`, `CONTINUATION_INDENT_SIZE` and `SOFT_MARGINS`.
- Non-option behaviour suites (throws/headers, generic spacing,
  extends/implements, switch, parse errors, idempotency, config XML) still
  exist in their own files with the same coverage as before.
- `cargo test` passes with the same results as before the restructure (no
  assertion weakened or dropped).
- The README Testing section describes the per-option layout; a changelog
  entry is appended when the change ships (fawi-implement).

# Implementation plan

## Approach

Create `crates/core/tests/options/` with one test file per `OptionDef` in the
`OPTIONS` registry (25 files, named `<xml_name>.rs`). Each file uses the
existing `common` helpers (`format`, `format_with`, `style`, `assert_contains`,
`assert_idempotent`) and asserts full `assert_eq!(format_with(input, style),
golden)` where a golden exists. Fixture directories move beside their option
(`tests/java/assignment/` → `tests/java/assignment_wrap/`, etc.) so the
option↔fixture correlation is direct; `include_str!` paths in the moved tests
are adjusted accordingly.

Existing topic files are dissolved:

- `assignment.rs` → `options/assignment_wrap.rs` (fixtures move).
- `binary.rs` → `options/binary_operation_wrap.rs` (fixtures move).
- `control_flow.rs` → `options/keep_simple_blocks_in_one_line.rs` (if/else/for/
  while/do/try/sync one-liner tests + their fixtures move); the switch and
  condition-handling tests stay in a topic file `switch.rs`.
- `imports.rs` → `options/class_count_to_use_import_on_demand.rs` (fixtures
  move).
- `indent.rs` → `options/use_tab_character.rs` + `options/tab_size.rs` (split
  the tab fixture tests; keep the logical-column equivalence and idempotency
  checks with the `use_tab_character` file).
- `methods.rs` → `options/keep_simple_methods_in_one_line.rs`; the throws /
  header / varargs / annotation tests stay in a topic file `methods.rs`.
- `records.rs` → `options/record_components_wrap.rs`,
  `options/align_multiline_records.rs`, `options/new_line_after_lparen_in_record_header.rs`
  (component-wrap tests + fixtures split across the three); the structural
  record tests stay in a topic file `records.rs`.

New per-option files fill the coverage gaps, each with a minimal fixture (+ a
`*.out.java` golden where output differs) generated from the formatter itself
and sanity-checked against the option's semantics:

- `indent_size.rs`, `continuation_indent_size.rs`, `right_margin.rs`
  (`SOFT_MARGINS`), `class_brace_style.rs`, `method_brace_style.rs`,
  `brace_style.rs` (`BRACE_STYLE` → `other_brace_style`),
  `call_parameters_wrap.rs`, `call_parameters_lparen_on_next_line.rs`,
  `call_parameters_rparen_on_next_line.rs`, `method_parameters_wrap.rs`,
  `method_parameters_lparen_on_next_line.rs`,
  `method_parameters_rparen_on_next_line.rs`, `method_call_chain_wrap.rs`,
  `keep_simple_lambdas_in_one_line.rs`, `annotation_parameter_wrap.rs`.

Existing per-option coverage already present elsewhere is _moved_, not
rewritten: `records.rs`'s `record_class_brace_style_is_honoured` →
`options/class_brace_style.rs`; `methods.rs`'s and `control_flow.rs`'s
next-line-brace interplay tests → the respective brace files;
`methods.rs`'s `throws_survives_wrapped_parameter_lists` →
`options/method_parameters_*.rs` (split by the option it exercises).

Topic files that stay: `config.rs` (XML parse/serialize), `generics.rs`
(spacing normalisation), `types.rs` (extends/implements), `parse_errors.rs`
(diagnostics), `idempotency.rs` (kitchen-sink goldens + idempotency),
`methods.rs` (throws/headers), `records.rs` (structural), `switch.rs`
(switch + condition handling, split out of `control_flow.rs`).

Docs touched: `README.md` Testing section, `docs/requirements.md` R9 wording,
`docs/dev/backlog/index.md` (row state), `docs/dev/changelog.md` on completion.

## Steps

- [ ] Create `crates/core/tests/options/`; move `assignment.rs` to
      `options/assignment_wrap.rs`, `tests/java/assignment/` to
      `tests/java/assignment_wrap/`, fix `include_str!` paths (AC: per-option
      file + fixtures).
- [ ] Move `binary.rs` to `options/binary_operation_wrap.rs`, `tests/java/binary/`
      to `tests/java/binary_operation_wrap/`, fix paths (AC: per-option file).
- [ ] Split `control_flow.rs`: one-liner tests + `tests/java/control/`
      one-liner fixtures → `options/keep_simple_blocks_in_one_line.rs` and
      `tests/java/keep_simple_blocks_in_one_line/`; switch/condition tests →
      `switch.rs` with `tests/java/switch/` fixtures (AC: non-option coverage
      preserved).
- [ ] Move `imports.rs` to `options/class_count_to_use_import_on_demand.rs`,
      `tests/java/imports/` to `tests/java/class_count_to_use_import_on_demand/`
      (AC: per-option file).
- [ ] Split `indent.rs`: tab-output tests → `options/use_tab_character.rs`;
      add `options/tab_size.rs` covering tab-width arithmetic; move
      `tests/java/indent/` fixtures (AC: both options covered).
- [ ] Split `methods.rs`: keep-simple-methods tests →
      `options/keep_simple_methods_in_one_line.rs`; throws/header tests stay in
      `methods.rs`; move the wrapped-params fixture and split
      `throws_survives_wrapped_parameter_lists` across
      `options/method_parameters_wrap.rs` / `_lparen_on_next_line.rs` /
      `_rparen_on_next_line.rs` (AC: coverage preserved, options covered).
- [ ] Split `records.rs`: component-wrap tests →
      `options/record_components_wrap.rs`, `options/align_multiline_records.rs`,
      `options/new_line_after_lparen_in_record_header.rs` with the
      `component_wrap*` fixtures; structural tests stay in `records.rs`; move
      `record_class_brace_style_is_honoured` → `options/class_brace_style.rs`
      (AC: per-option files, structural coverage preserved).
- [ ] Move next-line-brace interplay tests from `methods.rs` /
      `control_flow.rs` into `options/method_brace_style.rs` and
      `options/brace_style.rs` respectively (AC: brace options covered).
- [ ] Add `options/indent_size.rs`, `options/continuation_indent_size.rs`,
      `options/right_margin.rs` with minimal fixtures (goldens where output
      differs) (AC: previously untested options covered).
- [ ] Add `options/call_parameters_wrap.rs`,
      `options/call_parameters_lparen_on_next_line.rs`,
      `options/call_parameters_rparen_on_next_line.rs` with wrapped-call
      fixtures (AC: call-parameter options covered).
- [ ] Add `options/method_call_chain_wrap.rs` with a chain fixture (AC:
      previously untested option covered).
- [ ] Add `options/keep_simple_lambdas_in_one_line.rs` and
      `options/annotation_parameter_wrap.rs` with minimal fixtures (AC:
      previously untested options covered).
- [ ] Run `cargo test`; fix any moved-path or assertion mistakes; confirm the
      suite passes with the same results (no assertion dropped) (AC: cargo
      test green, coverage preserved).
- [ ] Update `README.md` Testing section and `docs/requirements.md` R9 wording;
      set the backlog index row to planned→done and append a changelog entry
      when shipping (AC: docs updated, changelog entry).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
