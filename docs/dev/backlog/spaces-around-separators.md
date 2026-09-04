---
type: ChangeRequest
kind: feature
title: Honour the spacing-around-separators options
description: Apply comma/semicolon/colon/question-mark spacing options, including the Java for-each colon option.
state: done
verified: { by: maintainer, at: 2026-09-03T22:03:18Z }
priority: high
tags: [dev, formatter]
owner: maintainer
---

# Problem

Every row of the `docs/settings/common.md` "Spaces / After / before separators" table — plus the `SPACE_BEFORE_COLON_IN_FOREACH` row in `docs/settings/java.md` "Miscellaneous spacing & blank lines" — is marked ❌: java-formatter parses none of them and emits a fixed canonical spacing (one space after commas, semicolons and colons, none before; `?` and `:` spaced both sides in ternaries; the for-each colon spaced). The canonical generic type-argument spacing normalised in the shipped R14 work (README formatting-behaviour notes) is likewise fixed; unimplemented options are safely ignored (R7 in `docs/requirements.md`).

# Proposal

Parse the following options into `config::JavaStyle` via the `OPTIONS` registry in `crates/core/src/config.rs`: `SPACE_AFTER_COMMA`, `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS`, `SPACE_BEFORE_COMMA`, `SPACE_AFTER_SEMICOLON`, `SPACE_BEFORE_SEMICOLON`, `SPACE_BEFORE_QUEST`, `SPACE_AFTER_QUEST`, `SPACE_BEFORE_COLON`, `SPACE_AFTER_COLON`, `SPACE_BEFORE_TYPE_PARAMETER_LIST`, `SPACE_BEFORE_COLON_IN_FOREACH` — each an `OptionDef` with the IntelliJ built-in default from the table (absent → default), mapped to the existing `OptionValue::Bool`. `SPACE_BEFORE_COLON_IN_FOREACH` is the Java-specific colon knob from the `<JavaCodeStyleSettings>` block and belongs to this family. In `crates/core/src/formatter.rs`, the space around each separator token follows its toggle; the type-argument spacing normalised in R14 honours `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` when set.

Docs touched: on delivery the implementation flips the ❌ marks to ✅ for these rows in `docs/settings/common.md` and `docs/settings/java.md`, updates the README honoured-options table and formatting-behaviour notes, adds a requirement row to `docs/requirements.md`, and appends `docs/dev/changelog.md`.

# Decisions

1. **One family, one request.** Only the listed separator options are added; unlisted spacing options stay unimplemented and safely ignored (R7).
2. **Defaults.** IntelliJ built-in defaults from the tables: `SPACE_BEFORE_COMMA`, `SPACE_BEFORE_SEMICOLON` and `SPACE_BEFORE_TYPE_PARAMETER_LIST` default `false`, the rest default `true` — equal to today's canonical output, so absent/default schemes keep byte-identical goldens.
3. **Semantics.** Whitespace-only change (R5); unmodelled shapes echoed verbatim (R4); single-space insertions / removals are idempotent (R6).
4. **Separator granularity.** Each separator context is its own toggle — commas in calls / declarations vs. commas in type arguments, for-header semicolons, ternary `?` / `:` vs. the for-each colon, and the class / method-name-to-type-parameter gap.

# Acceptance criteria

- Per listed option: a golden fixture and per-option test file under `crates/core/tests/options/` testing the option's interesting values (on / off) plus the absent-option default.
- Toggle away from its default renders the affected separator accordingly in the `*.out.java` golden (e.g. `Map<String,Integer>` with `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` off, `for (T t: xs)` with `SPACE_BEFORE_COLON_IN_FOREACH` off, `a?b:c` with `SPACE_BEFORE_QUEST` off).
- Default-scheme output is unchanged and the whole suite is green (`cargo test`); the new goldens are idempotent.
- `docs/settings/common.md` and `docs/settings/java.md` marks are flipped to ✅ and the README / `docs/requirements.md` / `docs/dev/changelog.md` are updated.

# Implementation plan

## Approach

Two sides: configuration and rendering.

**Configuration (src/config.rs).** Add eleven `bool` fields to `JavaStyle`
(`space_after_comma`, `space_after_comma_in_type_arguments`, `space_before_comma`,
`space_after_semicolon`, `space_before_semicolon`, `space_before_quest`,
`space_after_quest`, `space_before_colon`, `space_after_colon`,
`space_before_type_parameter_list`, `space_before_colon_in_foreach`) in a new
`// --- separator spacing ---` group. `JavaStyle` is constructed only via
`Default`, so no literal-site changes are needed; give each field the IntelliJ
default from the tables — `space_before_comma`, `space_before_semicolon` and
`space_before_type_parameter_list` → `false`, the rest → `true` — equal to
today's canonical output, so absent/default schemes keep goldens byte-identical.
Add one `OptionDef` per option to the `OPTIONS` registry (group `"Spaces"`,
`OptionValue::Bool`, description mirroring the settings tables): the ten
separator options live in the JAVA `codeStyleSettings` block
(`Section::CodeStyleJava`), and `SPACE_BEFORE_COLON_IN_FOREACH` — the
Java-specific colon knob, per the request — in the `<JavaCodeStyleSettings>`
block (`Section::JavaCodeStyle`). Parsing and serialization are registry-driven,
so `parse_codestyle` / `serialize_codestyle` need no changes: absent → default
falls out of the existing `OptionMap::get_bool`, and the GUI's option panel
iterates `OPTIONS`, so the new entries appear automatically as checkboxes — no
GUI source change. Per AGENTS.md there are no `parse_codestyle` tests and no
config-XML topic suite; the mapping is exercised through the per-option golden
tests (absent-option default = default-style golden).

**Rendering (src/formatter.rs).** Each separator context is its own toggle
(decision 4). Add a small comma-separator helper (e.g. `comma_sep(after: bool)`
built from `space_before_comma` and the after toggle) and route every
single-line comma join through it: `SPACE_AFTER_COMMA` governs commas in calls
(`flat_args`), declarations (`formal_params`, `flat_formal_params`, `field_decl`,
`local_var` declarator lists), annotations (`flat_ann_args`), arrays
(`flat_arr_init`), record components (`record_components`), lambda inferred
parameters, `throws` lists and `implements`/`extends` type lists
(`flat_type_list`), while `flat_type_args` — the R14-normalised site — uses
`SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` for its after space. The `",\n"` wrapped
joins (`args_wrapped`, `annotation_expanded`, `array_init`, wrapped
`formal_params` / `record_components`, `enum_body`) stay as-is: a newline
already replaces the after space. For the `for` header, `for_stmt` keeps its
raw-header echo but normalises the whitespace around each `;` to follow
`SPACE_BEFORE_SEMICOLON` / `SPACE_AFTER_SEMICOLON` (never inserting a space
before `)`), e.g. after-off → `for (int i = 0;i < n;i++)`, before-on →
`for (int i = 0 ; i < n ; i++)`; if the transform hits awkward empty-slot edges
(`for (;;)`), rebuild the header from the `for_statement` init/condition/update
children instead. Ternary rendering (`ternary` and the `flat` ternary arm)
builds the `?` and `:` separators from `SPACE_BEFORE_QUEST` / `SPACE_AFTER_QUEST`
and `SPACE_BEFORE_COLON` / `SPACE_AFTER_COLON` instead of the hard-coded
`" ? "` / `" : "`. The `enhanced_for` colon takes its before space from
`SPACE_BEFORE_COLON_IN_FOREACH` and its after space from `SPACE_AFTER_COLON`.
`SPACE_BEFORE_TYPE_PARAMETER_LIST` inserts the name→`<…>` gap in `class_decl`,
`iface_decl` and `record_decl` (default off keeps `class Foo<T>`); generic
method/constructor type-parameter lists follow the modifiers and are left alone
unless an IntelliJ check shows otherwise. Deliberately unchanged (defaults
already match today): statement-terminating `;`, switch `case`-label colons
(emit `:\n`), the assert colon, and enum-constant commas.

**Tests (hard rules from AGENTS.md).** Eleven new option files under
`crates/core/tests/options/<xml_option>.rs` (lower-snake of the XML name, e.g.
`space_after_comma_in_type_arguments.rs`), each starting
`use super::common::*;`, wired in `tests/options.rs` via `#[path = "options/<name>.rs"]
mod <name>;`, with fixtures under `tests/java/<option>/` embedded through
relative `include_str!` paths (`../java/<option>/<scenario>.java`). Each file
holds two goldens: the option toggled away from its default (e.g.
`Map<String,Integer>` with `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` off,
`for (T t: xs)` with `SPACE_BEFORE_COLON_IN_FOREACH` off, `a?b:c` with
`SPACE_BEFORE_QUEST` off, `class Foo <T>` with `SPACE_BEFORE_TYPE_PARAMETER_LIST`
on) and the absent-option default via the default style, asserting today's
canonical output is unchanged. No inline Java strings, no new helpers, no
`parse_codestyle` tests. Insertion/removal of a single space is idempotent by
construction (R6); verify by formatting each golden with its own style during
development.

**Docs.** `docs/settings/common.md` (10 rows) and `docs/settings/java.md`
(1 row) flip to ✅ (reword `SPACE_BEFORE_COLON`'s effect so the for-each colon
is no longer listed there), the README honoured-options table gains the 11
options and the type-argument spacing formatting-behaviour note is adjusted to
mention `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS`, `docs/requirements.md` gains a
new requirement row, and `docs/dev/changelog.md` is appended.

## Steps

- [x] config.rs: add the 11 bool fields to `JavaStyle` with the table defaults
      and the 11 `OptionDef` entries (group "Spaces", `OptionValue::Bool`; ten
      `Section::CodeStyleJava`, `SPACE_BEFORE_COLON_IN_FOREACH`
      `Section::JavaCodeStyle`) (AC: absent → default mapping).
- [x] formatter.rs: add the comma-separator helper and route the single-line
      comma joins through it — `SPACE_AFTER_COMMA` at the
      call/declaration/annotation/array/record/lambda/throws/implements sites
      and `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` at `flat_type_args`; leave the
      `",\n"` wrapped joins as-is (AC2 for the comma options).
- [x] formatter.rs: `for_stmt` — space around each header `;` per
      `SPACE_AFTER_SEMICOLON` / `SPACE_BEFORE_SEMICOLON` (AC2).
- [x] formatter.rs: `ternary` and the `flat` ternary arm — `?` spacing per
      `SPACE_BEFORE_QUEST` / `SPACE_AFTER_QUEST`, `:` spacing per
      `SPACE_BEFORE_COLON` / `SPACE_AFTER_COLON` (AC2).
- [x] formatter.rs: `enhanced_for` — colon before-space per
      `SPACE_BEFORE_COLON_IN_FOREACH`, after-space per `SPACE_AFTER_COLON`
      (AC2).
- [x] formatter.rs: `class_decl` / `iface_decl` / `record_decl` — optional
      name→`<…>` gap per `SPACE_BEFORE_TYPE_PARAMETER_LIST` (AC2).
- [x] Tests: create the 11 option files under `crates/core/tests/options/`,
      wire them in `tests/options.rs`, and add the `tests/java/<option>/`
      fixture+golden pairs — each file asserts the toggle-away-from-default
      rendering and the absent-option default (AC1, AC2).
- [x] Verify: `cargo test` green with no existing golden changed, each new
      golden idempotent under its own style, and `cargo build` for the whole
      workspace (the GUI compiles with the new registry entries) (AC3).
- [x] Docs: flip the marks in `docs/settings/common.md` and
      `docs/settings/java.md`, update the README honoured-options table and the
      type-argument spacing note, add the requirement row to
      `docs/requirements.md`, append `docs/dev/changelog.md`, and re-run
      `cargo test` (AC4).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
