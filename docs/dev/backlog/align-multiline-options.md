---
type: ChangeRequest
kind: feature
title: Honour the align-when-multiline options
description: Implement the common ALIGN_MULTILINE_* / ALIGN_* options so wrapped constructs align instead of using the plain continuation indent.
state: done
priority: medium
tags: [dev, formatter]
owner: maintainer
verified:
  by: Zed coding agent
  at: 2026-09-05T14:45:00Z
---

# Problem

Every row of the `docs/settings/common.md` "Alignment" table is marked ❌: java-formatter parses none of the common align options, so a scheme that sets them is only partially honoured and wrapped constructs keep using the plain continuation indent. Only the record-header alignment (`ALIGN_MULTILINE_RECORDS`) is implemented today and is the model this family extends; the rest are safely ignored (R7 in `docs/requirements.md`).

# Proposal

Parse the following options into `config::JavaStyle` via the `OPTIONS` registry in `crates/core/src/config.rs`: `ALIGN_MULTILINE_PARAMETERS`, `ALIGN_MULTILINE_PARAMETERS_IN_CALLS`, `ALIGN_MULTILINE_RESOURCES`, `ALIGN_MULTILINE_FOR`, `ALIGN_MULTILINE_BINARY_OPERATION`, `ALIGN_MULTILINE_ASSIGNMENT`, `ALIGN_MULTILINE_TERNARY_OPERATION`, `ALIGN_MULTILINE_THROWS_LIST`, `ALIGN_THROWS_KEYWORD`, `ALIGN_MULTILINE_EXTENDS_LIST`, `ALIGN_MULTILINE_METHOD_BRACKETS`, `ALIGN_MULTILINE_PARENTHESIZED_EXPRESSION`, `ALIGN_MULTILINE_ARRAY_INITIALIZER_EXPRESSION`, `ALIGN_MULTILINE_CHAINED_METHODS`, `ALIGN_GROUP_FIELD_DECLARATIONS`, `ALIGN_CONSECUTIVE_VARIABLE_DECLARATIONS`, `ALIGN_CONSECUTIVE_ASSIGNMENTS`, `ALIGN_SUBSEQUENT_SIMPLE_METHODS` — each an `OptionDef` with the IntelliJ built-in default from the table (absent → default), mapped to the existing `OptionValue::Bool`. In `crates/core/src/formatter.rs`, when a wrapped construct's toggle is on, its continuation lines align under the first element instead of the continuation indent, extending the shipped record-component alignment approach.

Docs touched: on delivery the implementation flips the ❌ marks to ✅ for these rows in `docs/settings/common.md`, updates the README honoured-options table and formatting-behaviour notes, adds a requirement row to `docs/requirements.md`, and appends `docs/dev/changelog.md`.

# Decisions

1. **One family, one request.** Only the listed align options are added; unlisted `ALIGN_*` options stay unimplemented and safely ignored (R7).
2. **Defaults.** IntelliJ built-in defaults from the table: most are `false`, but `ALIGN_MULTILINE_PARAMETERS`, `ALIGN_MULTILINE_RESOURCES` and `ALIGN_MULTILINE_FOR` default `true`.
3. **Semantics.** Whitespace-only layout change (R5); unmodelled shapes echoed verbatim (R4); aligned output is stable, so idempotency (R6) holds.
4. **Per-construct granularity.** Each wrapped construct is its own toggle; absent or default schemes keep byte-identical output, so existing goldens stay green.

# Acceptance criteria

- Per listed option: a golden fixture and per-option test file under `crates/core/tests/options/` testing the option's interesting values (on / off) plus the absent-option default.
- Toggle on → the wrapped construct's continuation lines align under its first element in the `*.out.java` golden; off (and by default) → the plain continuation indent; the three default-`true` options align when absent.
- Default-scheme output is unchanged and the whole suite is green (`cargo test`); the new goldens are idempotent.
- `docs/settings/common.md` marks are flipped to ✅ and the README / `docs/requirements.md` / `docs/dev/changelog.md` are updated.

# Implementation plan

## Approach

**Configuration first.** Add eighteen `bool` fields to `JavaStyle` (crates/core/src/config.rs L105-150) with `Default` values per the docs/settings/common.md Alignment table — only `ALIGN_MULTILINE_PARAMETERS`, `ALIGN_MULTILINE_RESOURCES` and `ALIGN_MULTILINE_FOR` default `true` — and eighteen `OptionDef` entries in the `OPTIONS` registry (L232-567): all `Section::CodeStyleJava` (the Alignment table describes the `<codeStyleSettings language="JAVA">` block; codestyle.xml confirms these option names live there, unlike the record options under `<JavaCodeStyleSettings>`), `OptionValue::Bool`, GUI group `"Alignment"`, and `get`/`set` closures over the new fields, placed in the registry between the wrapping entries (after `BINARY_OPERATION_WRAP`, L459) and the one-liner entries so the GUI shows an "Alignment" group between "Wrapping" and "One-liners". Because `parse_codestyle`, `serialize_codestyle` and the GUI iterate `OPTIONS`, parsing, minimal-scheme serialization (only non-defaults written) and the egui editor pick the options up with no further wiring, and `JavaStyle` is constructed only via `Default`, so no literal sites change. Per `.agents/AGENTS.md` there are no `parse_codestyle` tests; the config mapping is exercised through the per-option files — fields driven via `style(...)`, and an absent-option case formatted with `default_style()`.

**Alignment mechanics.** The shipped `ALIGN_MULTILINE_RECORDS` implementation is the model: `record_components` (formatter.rs L642-691) computes the column of the first element (`open_col`, tab-aware via `col_after`) and, when aligning, prefixes each continuation line with spaces to that column (`" ".repeat(open_col + 1)`, L666-670) instead of `self.cont(indent)`. "Align under the first element" is exactly this replacement of the fixed continuation prefix with spaces to the column where the construct's first element sits, so the plan reuses that pattern via a small shared helper (e.g. `fn align_prefix(&self, first_col: usize) -> String`). Alignment stays space-based like the record model (README tab behaviour note); the toggle-off / absent path keeps today's `cont`/`ind` prefix byte-for-byte, so the default-`false` options cannot disturb any existing golden.

**Already-wrappable constructs (delivered here, one toggle each).** Each option below governs a site whose wrapped layout exists today; the map of option → site → current continuation emission:

- `ALIGN_MULTILINE_PARAMETERS` (default `true`) → declaration parameter lists: `formal_params` called from `method_decl` / `constructor_decl` (L793-796, L836-839, L1143-1193);
- `ALIGN_MULTILINE_PARAMETERS_IN_CALLS` → `args_wrapped` (L2132-2179), shared by method calls and `new` expressions (`inv_wrapped`, `new_expr`);
- `ALIGN_MULTILINE_BINARY_OPERATION` → `binary` continuation lines (L2409-2417, `cont`);
- `ALIGN_MULTILINE_ASSIGNMENT` → `assign_expr` wrapped-RHS line (L2371-2374, `cont`);
- `ALIGN_MULTILINE_TERNARY_OPERATION` → `ternary` `?`/`:` lines (L2469-2483, `cont`);
- `ALIGN_MULTILINE_ARRAY_INITIALIZER_EXPRESSION` → `array_init` elements (L2564-2576, `ind(inner)`);
- `ALIGN_MULTILINE_CHAINED_METHODS` → `fmt_chain` link lines (L2241-2248, `cont`);
- `ALIGN_MULTILINE_METHOD_BRACKETS` → `(`/`)` placement of a wrapped declaration parameter list (the `formal_params` paren arms, L1187-1192);
- `ALIGN_MULTILINE_PARENTHESIZED_EXPRESSION` → a `parenthesized_expression` whose content wraps (expr arm L2049-2055; the inner continuation then aligns to the column after `(`).

For the paren-based lists the canonical aligned shape keeps the first element on the header line after `(` when the lparen-on-next-line toggle is off and aligns the remaining elements under it — the same two layouts `record_components` distinguishes (L672-690) — while with lparen/rparen on their own lines the elements already begin their own lines and the "first element column" is that element's own start column; under the default-`true` `ALIGN_MULTILINE_PARAMETERS` this keeps the existing method-parameter goldens (`method_parameters_wrap`, `method_parameters_rparen_on_next_line`, …) byte-identical. Where two toggles could govern one wrapped output (e.g. a wrapped binary inside a parenthesized expression) the innermost wrapped list's own toggle governs its lines; any shape left ambiguous by the documentation is pinned by the goldens and, when an IntelliJ installation is available to the implementer, verified against real IntelliJ output before the golden is committed — the same mitigation the binary-expression and switch requests used.

**Consecutive (columnar) alignment.** `ALIGN_CONSECUTIVE_VARIABLE_DECLARATIONS` and `ALIGN_CONSECUTIVE_ASSIGNMENTS` are not continuation alignment but column alignment of runs of consecutive statements in `Fmt::block` (L1249-1280). A run is a maximal sequence of the governed statement kind (`local_variable_declaration`, or `expression_statement` whose expression is an assignment) with no source blank line and no comment between members — blank detection via `has_blank_line_between` (L1284), comment/extras break runs. Run members are padded with spaces so the aligned element (name / `=` / value per the option) shares one column; padding is space-based like the other aligns.

**Options gated on sibling requests.** The remaining options govern constructs whose *wrapped rendering does not exist yet*, so no alignment is observable until the construct can wrap; this request delivers their parse/serialize plumbing in step 1, their per-option files pin the R7-safe no-op (nothing wraps → nothing aligns → on/off/absent byte-identical), and their alignment branches and aligned goldens attach to the layout the sibling requests introduce:

- `ALIGN_MULTILINE_RESOURCES`, `ALIGN_MULTILINE_THROWS_LIST`, `ALIGN_THROWS_KEYWORD` and `ALIGN_MULTILINE_EXTENDS_LIST` align the resource / `throws` / `extends`-`implements` list layouts that the "wrapping-declaration-clauses" request delivers (`try_stmt` L1696-1753, `method_decl` throws L798-807, `class_decl` L461-496 / `iface_decl` L498-529 / `enum_decl` / `record_decl` headers);
- `ALIGN_MULTILINE_FOR` aligns the for-header parts that "wrapping-expressions-and-statements" will wrap (`for_stmt` L1528-1555);
- `ALIGN_GROUP_FIELD_DECLARATIONS` and `ALIGN_SUBSEQUENT_SIMPLE_METHODS` need consecutive output-adjacent members, but `class_body` (L705-732) currently separates every member with a blank line, so runs are singletons until the "blank-line-policy" request's `BLANK_LINES_AROUND_FIELD` / `BLANK_LINES_AROUND_METHOD` (default `0`) makes fields / one-line methods adjacent.

Recommended execution order for this request is therefore after (or in lockstep with) wrapping-declaration-clauses, wrapping-expressions-and-statements and blank-line-policy; the steps below mark exactly which parts stay open until then.

**Tests and docs.** Tests follow the `.agents/AGENTS.md` hard rules: one golden-pair module per option in `crates/core/tests/options/<xml_option>.rs`, wired via `#[path]` in `crates/core/tests/options.rs`, starting `use super::common::*;`, doc header `//! <XML_OPTION> — …` plus `//! Fixtures live under tests/java/<option>/.`, fixtures under `crates/core/tests/java/<option>/` referenced by `include_str!`, shared input/golden stems, no inline Java, no new test helpers. Each file tests the interesting values — align on (and, for the default-`true` options, the absent case) → aligned `*.out.java` golden; off → continuation-indent golden; absent → `default_style()` golden — and each new golden is checked idempotent by formatting it a second time during development (no `assert_idempotent` helper exists or is added). Default/absent schemes must keep the whole existing suite green; any existing golden that shifts because a default-`true` align now engages is regenerated only after verifying the new shape against IntelliJ. Docs on delivery: flip the eighteen ❌ Alignment rows to ✅ in docs/settings/common.md (L106-125), add the eighteen rows to the README honoured-options table plus a formatting-behaviour note for the alignment family (extending the record-alignment wording), add a requirement row to docs/requirements.md, and append docs/dev/changelog.md.

## Steps

- [x] config.rs: add the eighteen `bool` fields + `Default` values to `JavaStyle` (three `true` per the table) and the eighteen `OptionDef` entries (group `"Alignment"`, `Section::CodeStyleJava`, `OptionValue::Bool`, `get`/`set` closures) after the wrapping entries; `cargo build` and `cargo test` stay green — no behaviour change yet (AC: option mapping / defaults / absent → default).
- [x] formatter.rs: add the shared space-based alignment-prefix helper mirroring `record_components` L666-670; leave every toggle-off path untouched (AC2/AC3 mechanics).
- [x] `ALIGN_MULTILINE_PARAMETERS`: teach the `formal_params` declaration path both aligned layouts (first element stays on the header line when the lparen stays, else elements align to the first element's own column) and keep off = today's form; add `tests/options/align_multiline_parameters.rs` + fixtures under `tests/java/align_multiline_parameters/` (on, off, and absent-default aligned) and wire the module in `tests/options.rs` (AC1, AC2, AC3).
- [x] `ALIGN_MULTILINE_PARAMETERS_IN_CALLS`: same treatment for `args_wrapped` (calls and `new` expressions); add `tests/options/align_multiline_parameters_in_calls.rs` + fixtures (on, off, absent-default = today's form) (AC1, AC2, AC3).
- [x] `ALIGN_MULTILINE_BINARY_OPERATION`: in `binary` emit the first-operand column prefix when on (off/absent = `cont`, existing `long_sum` goldens unchanged); add `tests/options/align_multiline_binary_operation.rs` + fixtures (AC1, AC2, AC3).
- [x] `ALIGN_MULTILINE_ASSIGNMENT`: in `assign_expr` align the wrapped RHS when on (off/absent unchanged); add `tests/options/align_multiline_assignment.rs` + fixtures covering the statement and field/local-initialiser sites (AC1, AC2, AC3).
- [x] `ALIGN_MULTILINE_TERNARY_OPERATION`: in `ternary` align the `?`/`:` lines when on; add `tests/options/align_multiline_ternary_operation.rs` + fixtures; verify the aligned shape against IntelliJ when available (AC1, AC2, AC3).
- [x] `ALIGN_MULTILINE_ARRAY_INITIALIZER_EXPRESSION` and `ALIGN_MULTILINE_CHAINED_METHODS`: align `array_init` elements / `fmt_chain` dots when on (off/absent unchanged); add `tests/options/align_multiline_array_initializer_expression.rs` and `align_multiline_chained_methods.rs` + fixtures (AC1, AC2, AC3).
- [x] `ALIGN_MULTILINE_METHOD_BRACKETS` and `ALIGN_MULTILINE_PARENTHESIZED_EXPRESSION`: align the wrapped-declaration parens and the wrapped paren-content continuation when on; add `tests/options/align_multiline_method_brackets.rs` and `align_multiline_parenthesized_expression.rs` + fixtures; check the ambiguous shapes against IntelliJ (AC1, AC2, AC3).
- [x] `ALIGN_CONSECUTIVE_VARIABLE_DECLARATIONS` and `ALIGN_CONSECUTIVE_ASSIGNMENTS`: run detection + column padding in `Fmt::block`; add `tests/options/align_consecutive_variable_declarations.rs` and `align_consecutive_assignments.rs` + fixtures (consecutive statements; runs broken by a blank line or comment; off; absent) (AC1, AC2, AC3).
- [x] `ALIGN_GROUP_FIELD_DECLARATIONS` and `ALIGN_SUBSEQUENT_SIMPLE_METHODS`: add the `class_body` run machinery over output-adjacent members plus their per-option files; while `class_body` inserts a blank line between every member the fixtures pin the no-op, and the aligned goldens are completed when the blank-line-policy request makes members adjacent (AC1; AC2/AC3 gated on that request).
- [x] Wrap-gated options `ALIGN_MULTILINE_RESOURCES`, `ALIGN_MULTILINE_FOR`, `ALIGN_MULTILINE_THROWS_LIST`, `ALIGN_THROWS_KEYWORD`, `ALIGN_MULTILINE_EXTENDS_LIST`: option plumbing is live from step 1; add their per-option files pinning the no-op, and together with the wrapping-declaration-clauses / wrapping-expressions-and-statements layouts add the alignment branch + aligned goldens at the `try_stmt` / `for_stmt` / `method_decl` / class-header sites (AC1; AC2/AC3 gated on those requests).
- [x] Full-suite gate: run `cargo test`; existing goldens stay byte-identical under default/absent schemes; any golden that shifts because a default-`true` align now engages is regenerated only after IntelliJ verification; each new golden formats to itself on a second pass (AC3).
- [x] Docs: flip the eighteen ❌ → ✅ Alignment rows in docs/settings/common.md; add the eighteen rows to the README honoured-options table and an alignment formatting-behaviour note; add a requirement row to docs/requirements.md; append an entry to docs/dev/changelog.md; run `cargo test` once more to confirm the shipped state is green (AC4).

Commit: not committed (worktree changes only — the repository is left for the owner to commit).
