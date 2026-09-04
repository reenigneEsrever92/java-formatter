---
type: ChangeRequest
kind: feature
title: Honour the blank-line policy options (KEEP_BLANK_LINES_*, BLANK_LINES_*)
description: Parse and apply the keep/minimum blank-line options so vertical spacing matches the scheme.
state: done
verified: { by: maintainer, at: 2026-09-03T20:55:27Z }
priority: high
tags: [dev, formatter]
owner: maintainer
---

# Problem

The blank-line policy options — `KEEP_BLANK_LINES_*` (caps on preserved blank
lines) and `BLANK_LINES_*` (minimums inserted around constructs) — are valid
IntelliJ options that java-formatter does not yet parse or apply: they are
marked ❌ in docs/settings/common.md ("Blank lines" table) and
docs/settings/java.md ("Miscellaneous spacing & blank lines") and safely
ignored per R7, so a team scheme that sets them is only partially honoured and
output diverges from IntelliJ for the affected constructs. Today vertical
spacing is hard-coded: `Fmt::imports` inserts a blank line only between the
third-party group and the `java.*` / `javax.*` import group (README formatting
note), `Fmt::class_body` emits exactly one blank line between every member, and
package/import surroundings, method/field spacing and `}`-adjacent spacing
follow no option at all.

# Proposal

Parse each listed option into a `JavaStyle` field via the `OPTIONS` registry in
crates/core/src/config.rs: add `OptionDef` entries (section `CodeStyleJava` for
the common.md rows; `JavaCodeStyle` for `BLANK_LINES_AROUND_INITIALIZER` and
`BLANK_LINES_AROUND_FIELD_WITH_ANNOTATIONS`, which live in the
`<JavaCodeStyleSettings>` block) with the IntelliJ built-in default recorded in
the docs/settings tables; absent-from-scheme options keep the default. The
options are, from the common block, `KEEP_BLANK_LINES_IN_CODE`,
`KEEP_BLANK_LINES_IN_DECLARATIONS`,
`KEEP_BLANK_LINES_BETWEEN_PACKAGE_DECLARATION_AND_HEADER`,
`KEEP_BLANK_LINES_BEFORE_RBRACE`, `BLANK_LINES_BEFORE_PACKAGE`,
`BLANK_LINES_AFTER_PACKAGE`, `BLANK_LINES_BEFORE_IMPORTS`,
`BLANK_LINES_AFTER_IMPORTS`, `BLANK_LINES_AROUND_CLASS`,
`BLANK_LINES_AROUND_FIELD`, `BLANK_LINES_AROUND_METHOD`,
`BLANK_LINES_BEFORE_METHOD_BODY`, `BLANK_LINES_AROUND_FIELD_IN_INTERFACE`,
`BLANK_LINES_AROUND_METHOD_IN_INTERFACE`, `BLANK_LINES_AFTER_CLASS_HEADER`,
`BLANK_LINES_AFTER_ANONYMOUS_CLASS_HEADER` and `BLANK_LINES_BEFORE_CLASS_END`,
plus, from the Java-specific block, `BLANK_LINES_AROUND_INITIALIZER` and
`BLANK_LINES_AROUND_FIELD_WITH_ANNOTATIONS`. Apply
them in crates/core/src/formatter.rs at the constructs they govern: the
`KEEP_BLANK_LINES_*` options cap how many existing blank lines between the same
two constructs are preserved, and the `BLANK_LINES_*` options insert the
configured minimum around package, imports, class header/end, fields, methods,
initializer blocks and interfaces.

Docs touched: on delivery the implementation updates the docs/settings support
marks (❌ → ✅ for these rows), the README honoured-options table /
formatting-behaviour notes (the hard-coded "blank line before the `java.*` /
`javax.*` import group" note is reworded), docs/requirements.md (a new
requirement row), and docs/dev/changelog.md.

# Decisions

- **One family, one request.** Only the listed blank-line options are added
  here; `BLANK_LINES_BETWEEN_RECORD_COMPONENTS` belongs to the record-header
  layout request, and unimplemented rows otherwise remain safely ignored (R7).
- **Defaults.** The registry records the IntelliJ built-ins from the
  docs/settings tables (the `KEEP_*` caps default to `2`, the `BLANK_LINES_*`
  minimums as tabled). Default and absent-from-scheme styles keep current
  byte-identical output so existing goldens stay green; any fixture encoding a
  divergence from an IntelliJ default (e.g. the one blank line between adjacent
  fields vs `BLANK_LINES_AROUND_FIELD` `0`) is updated deliberately with this
  change.
- **Semantics.** Whitespace/layout only (R5); unmodelled constructs are echoed
  verbatim (R4); formatting formatted output is a no-op (R6). The two families
  keep their distinct meanings: `KEEP_*` caps, `BLANK_LINES_*` minimums.
- **Encodings.** Every listed option is a plain `u32` — the existing
  `OptionValue::UInt` covers the whole family; no new registry value types.

# Acceptance criteria

- A dedicated golden fixture + test file per option following the pattern in
  crates/core/tests/options/ (module named after the XML option, fixtures under
  tests/java/<option>/), each option tested at its interesting values plus an
  absent-option default check.
- Package/import boundary: `BLANK_LINES_BEFORE_PACKAGE` / `AFTER_PACKAGE` /
  `BEFORE_IMPORTS` / `AFTER_IMPORTS` produce the configured minimums, and the
  `KEEP_BLANK_LINES_*` caps truncate pre-existing runs of blank lines.
- `KEEP_BLANK_LINES_BEFORE_RBRACE`, `BLANK_LINES_AROUND_CLASS` / `AROUND_FIELD`
  / `AROUND_METHOD`, `BEFORE_METHOD_BODY`, `AFTER_CLASS_HEADER`,
  `AFTER_ANONYMOUS_CLASS_HEADER`, `BEFORE_CLASS_END`, the interface variants,
  `AROUND_INITIALIZER` and `AROUND_FIELD_WITH_ANNOTATIONS` behave per the
  tables.
- Default/absent schemes are unchanged: `cargo test` stays green and the new
  goldens are idempotent (R6).
- docs/settings marks flipped to ✅; README and docs/requirements.md updated.

# Implementation plan

## Approach

Two sides, as with the other option-family requests: configuration in
crates/core/src/config.rs, rendering in crates/core/src/formatter.rs.

**Configuration.** `JavaStyle` is only ever built through `Default` (no other
struct literals exist — verified by grep), so add 19 new `u32` fields with a
`Default` value for each, grouped under two comment banners: the four
`KEEP_BLANK_LINES_*` caps plus the thirteen `BLANK_LINES_*` minimums of the
common block, and the two Java-specific rows
(`BLANK_LINES_AROUND_INITIALIZER`, `BLANK_LINES_AROUND_FIELD_WITH_ANNOTATIONS`).
Defaults are the IntelliJ built-ins recorded in the docs/settings tables
(`KEEP_*` → 2, the minimums as tabled, notably `AROUND_FIELD` 0, `AROUND_METHOD` 1,
`AFTER_CLASS_HEADER` 0, `BEFORE_CLASS_END` 0, `AROUND_INITIALIZER` 1,
`AROUND_FIELD_WITH_ANNOTATIONS` 0). Then add one `OptionDef` per field to the
`OPTIONS` registry under a new GUI display group `"Blank lines"`: `Section::CodeStyleJava`
for the common-block rows (they serialize under `<codeStyleSettings language="JAVA">`)
and `Section::JavaCodeStyle` for the two Java-specific rows (they serialize under
`<JavaCodeStyleSettings>`), all `OptionValue::UInt` — no new registry value
types. Because `parse_codestyle` / `serialize_codestyle` iterate `OPTIONS`, both
directions come for free; absent-from-scheme options keep the field default.

**Rendering model.** Today every vertical gap is hard-coded to one `'\n'`
pushed unconditionally: `Fmt::program` between header comments / package /
imports / top-level types (L246-298), `Fmt::imports` at the `java.*`/`javax.*`
group boundary (L317-339, import-layout convention, kept as-is), `Fmt::class_body`
between every member (L705-732), `Fmt::enum_body` before each member
(L558-598), and `Fmt::block` collapses any source blank run to a single blank
between statements (L1249-1280). Replace that with one shared spacing function
used at every site:

    emitted = max(min(existing, keep_cap), required_min)

where `existing` is the number of blank lines actually present in the source run
between the two constructs and `keep_cap` is the governing `KEEP_*` option. The
formula is idempotent (the output's own count already satisfies `min ≤ emitted ≤
max(keep_cap, required_min)`, so a second pass reproduces it), which keeps R6
without a dedicated idempotency helper. `required_min` is the applicable
`BLANK_LINES_*` minimum: for a gap between two members it is the max of the two
members' "around" values (each member kind maps to its option — field unless
annotated, method/constructor, nested type, initializer — with the
`*_IN_INTERFACE` variants inside interfaces); for the first member of a body the
header minimum (`AFTER_CLASS_HEADER`, or `AFTER_ANONYMOUS_CLASS_HEADER` for
anonymous bodies) applies, and for the closing-gap the end minimum
(`BEFORE_CLASS_END`) applies, with the close-brace preserved run capped by
`KEEP_BLANK_LINES_BEFORE_RBRACE`. `existing` is counted from the source bytes
(a generalisation of `has_blank_line_between`, L1284-1292, from bool to count),
treating comment nodes as content so a comment line is not counted as blank.

The engine changes are: (a) `Fmt::block` gains an optional leading-blank count
(for `BLANK_LINES_BEFORE_METHOD_BODY`, applied only in the multi-line body path,
never in the collapsed one-line form) and caps inter-statement and pre-`}`
runs by `KEEP_BLANK_LINES_IN_CODE` / `KEEP_BLANK_LINES_BEFORE_RBRACE`;
(b) `Fmt::program` computes the package/import/type boundary counts from
`BLANK_LINES_BEFORE_PACKAGE` / `AFTER_PACKAGE` / `BEFORE_IMPORTS` /
`AFTER_IMPORTS` minimums and the `KEEP_BLANK_LINES_BETWEEN_PACKAGE_DECLARATION_AND_HEADER`
cap, and `KEEP_BLANK_LINES_IN_DECLARATIONS` caps preserved runs between
declarations and top-level types; (c) `Fmt::class_body` (shared by class,
interface, record and — via `Fmt::new_expr` — anonymous bodies) takes a small
body-kind argument so the right header minimum and per-member options are
selected, computes each inter-member and closing gap through the shared
function, and `Fmt::enum_body` routes its member declarations through the same
member-spacing path. Where two minimums compete for one gap (e.g. after a field
`0` vs around the next method `1`) the max rule reproduces IntelliJ; ambiguous
interplay (notably before-package vs header-comment gaps) is verified against a
real IntelliJ install when one is available to the implementer and otherwise
pinned by the goldens. Default/absent styles (all fields at the IntelliJ
built-ins) keep byte-identical output wherever today's canonical output already
matches IntelliJ — methods/classes keep one blank, and runs of ≤1 blank stay as
they are — so only goldens encoding the old engine's non-IntelliJ spacing (the
inserted blank between consecutive fields, or collapsed runs of 2+ blanks) change
and are regenerated deliberately with this request.

**Tests.** Follow the hard testing conventions of `.agents/AGENTS.md`: one
golden-pair test module per option in `crates/core/tests/options/` named after
the XML option, wired in `crates/core/tests/options.rs` via `#[path]`, starting
with `use super::common::*;`, doc header `//! <OPTION> — …` plus `//! Fixtures
live under tests/java/<option>/.`, fixtures under `crates/core/tests/java/<option>/`
referenced by `include_str!` with a shared input/golden stem, no inline Java
strings and no `parse_codestyle` tests. Each option is exercised at interesting
values (e.g. `0`, `1`, and one value past the default like `3`) plus an
absent-option check formatted with `default_style()` (i.e. `format(fixture) ==
fixture_default.out.java`), which pins the IntelliJ default. New goldens are
checked idempotent by a second manual format pass during development (no
`assert_idempotent` helper exists or is added).

## Steps

- [x] Add the 19 `u32` fields to `JavaStyle` in crates/core/src/config.rs (four
      `keep_blank_lines_*`, thirteen common `blank_lines_*`, two Java-specific
      `blank_lines_around_initializer` / `blank_lines_around_field_with_annotations`)
      with their `Default` values from the docs/settings tables; add the 19
      `OptionDef` entries (`UInt`, group `"Blank lines"`, `CodeStyleJava` for the
      common rows, `JavaCodeStyle` for the two Java rows) so parse/serialize
      cover them. `cargo build` + suite stay green (AC: config mapping / AC4).
- [x] Add the shared spacing machinery in crates/core/src/formatter.rs: a source
      blank-run counter generalising `has_blank_line_between`, the
      `max(min(existing, keep), min)` spacing function, and a leading/closing
      blank emitter; thread a body-kind parameter through `class_body`/`block`
      (named class / interface / anonymous / record; statement block)
      (AC2-AC4 mechanics).
- [x] Statement level: wire `KEEP_BLANK_LINES_IN_CODE` (inter-statement gaps in
      `block`) and `KEEP_BLANK_LINES_BEFORE_RBRACE` (pre-`}` gaps of statement
      blocks), and insert `BLANK_LINES_BEFORE_METHOD_BODY` leading blanks in the
      multi-line `method_body` path only (AC3).
- [x] File boundary: rework `Fmt::program` so the header-comment/package/imports/
      types gaps honour `BLANK_LINES_BEFORE_PACKAGE` / `AFTER_PACKAGE` /
      `BEFORE_IMPORTS` / `AFTER_IMPORTS`, the
      `KEEP_BLANK_LINES_BETWEEN_PACKAGE_DECLARATION_AND_HEADER` cap and
      `KEEP_BLANK_LINES_IN_DECLARATIONS` preserved-run caps; keep the
      `java.*`/`javax.*` group separator (import-layout convention) unchanged
      (AC2).
- [x] Member level: rework `class_body` / `enum_body` so inter-member and
      closing gaps use the shared function with per-kind around minimums —
      `AROUND_FIELD` (or `AROUND_FIELD_WITH_ANNOTATIONS` when the member carries
      an annotation), `AROUND_METHOD`, `AROUND_CLASS`, `AROUND_INITIALIZER`,
      the `*_IN_INTERFACE` variants — plus `AFTER_CLASS_HEADER`,
      `AFTER_ANONYMOUS_CLASS_HEADER` (via `new_expr`) and `BEFORE_CLASS_END`,
      capped by `KEEP_BLANK_LINES_IN_DECLARATIONS` (AC3).
- [x] Add fixtures + golden-pair tests for the four KEEP options
      (`tests/options/keep_blank_lines_in_code.rs`,
      `keep_blank_lines_in_declarations.rs`,
      `keep_blank_lines_between_package_declaration_and_header.rs`,
      `keep_blank_lines_before_rbrace.rs`): inputs with runs of 1 and 3 blank
      lines at the governed constructs, styles at caps `0` / `1` / `3`, plus an
      absent default golden; wire the modules in `tests/options.rs` (AC1, AC2).
- [x] Add fixtures + golden-pair tests for the four package/import boundary
      options (`blank_lines_before_package.rs`, `blank_lines_after_package.rs`,
      `blank_lines_before_imports.rs`, `blank_lines_after_imports.rs`): a
      package + imports + type file with pre-existing blank runs, minimums set
      to `0` and `2` plus default goldens asserting truncation of over-cap runs
      (AC1, AC2).
- [x] Add fixtures + golden-pair tests for the member options (`blank_lines_around_class.rs`,
      `blank_lines_around_field.rs`, `blank_lines_around_method.rs`,
      `blank_lines_before_method_body.rs`, `blank_lines_around_field_in_interface.rs`,
      `blank_lines_around_method_in_interface.rs`, `blank_lines_after_class_header.rs`,
      `blank_lines_after_anonymous_class_header.rs`, `blank_lines_before_class_end.rs`,
      `blank_lines_around_initializer.rs`, `blank_lines_around_field_with_annotations.rs`),
      each at `0` / a non-default value like `3` plus an absent default golden
      (AC1, AC3).
- [x] Run `cargo test`; regenerate only the existing goldens that encode the old
      engine's non-IntelliJ spacing (e.g. an inserted blank between consecutive
      fields, collapsed runs of 2+ blanks) — every other golden stays
      byte-identical under the default style, and every new golden formats to
      itself on a second pass (AC4).
- [x] Update docs: flip ❌ → ✅ for the nineteen rows in the "Blank lines" table
      of docs/settings/common.md and the "Miscellaneous spacing & blank lines"
      rows in docs/settings/java.md; add the options to the README honoured-options
      table and reword the `java.*`/`javax.*` blank-line formatting note (now an
      import-layout convention, not a blank-line-policy effect); add a new
      blank-line-policy requirement row to docs/requirements.md; append a
      changelog entry to docs/dev/changelog.md; run `cargo test` once more to
      confirm the shipped state is green (AC5).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
