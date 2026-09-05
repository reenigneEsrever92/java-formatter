---
type: ChangeRequest
kind: feature
title: Extend import-on-demand merging per the on-demand import options
description: Implement NAMES_COUNT_TO_USE_IMPORT_ON_DEMAND, PACKAGES_TO_USE_IMPORT_ON_DEMAND and USE_SINGLE_CLASS_IMPORTS.
state: done
verified: { by: maintainer, at: 2026-09-05 }
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

The "Imports" table in `docs/settings/java.md` marks `NAMES_COUNT_TO_USE_IMPORT_ON_DEMAND` (int, default `3`), `PACKAGES_TO_USE_IMPORT_ON_DEMAND` (table, default `java.awt`, `javax.swing`) and `USE_SINGLE_CLASS_IMPORTS` (bool, default `true`) ❌; only `CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND` (default `5`) is parsed and applied, by `merge_on_demand_imports` in `crates/core/src/formatter.rs`. The README notes describe that merge as conservative — skipped when a wildcard import is present, a name would become ambiguous, or a same-name top-level type is declared — and state that static imports are never merged, so schemes setting these three options are only partially honoured (safely ignored, R7).

# Proposal

Parse the three options into `config::JavaStyle` via the `OPTIONS` registry in `crates/core/src/config.rs` (IntelliJ built-in defaults from the table; absent → default) and extend `merge_on_demand_imports` so the static-member count, the always-merge package list and the single-class toggle join the shipped class-count merge: member imports of one owner collapse into `import static pkg.Owner.*;` at the NAMES count, single-type imports of a listed package collapse into `pkg.*` regardless of count, and `USE_SINGLE_CLASS_IMPORTS` flips the single-type/on-demand preference. The conservative guards (wildcard present, ambiguity, local same-name type) carry over. `PACKAGES_TO_USE_IMPORT_ON_DEMAND` serializes in the nested import-table XML of `docs/settings/java.md`, so the registry needs an `OptionValue` variant able to hold the package list — it currently supports only `Bool` / `UInt` / `Wrap` / `Brace`.

Docs touched: on delivery the implementation flips the three rows in `docs/settings/java.md` (❌ → ✅), updates the README honoured-options table and the conservative-merging note (the "never merges static imports" sentence changes), adds a new requirement row to `docs/requirements.md`, and appends `docs/dev/changelog.md`.

# Decisions

- **One family, one request.** Only these three options are added; the import-ordering rows (`IMPORT_LAYOUT_TABLE`, `LAYOUT_STATIC_IMPORTS_SEPARATELY`, module-import and import blank-line options) belong to the import-ordering-and-layout request and the remaining ❌ rows are ignored safely (R7).
- **Defaults.** `OptionDef` / `JavaStyle::default` carry IntelliJ's built-in defaults (`3`; `java.awt`, `javax.swing`; `true`); absent schemes behave as IntelliJ defaults, and default-scheme output stays byte-identical except where a fixture trips a new default threshold — verified during implementation (`cargo test` green).
- **Semantics.** Merging is the layout-level import grouping `CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND` already ships (R3); the ambiguity/wildcard/local-name guards are inherited, so no referenced name changes binding (R5) and the never-corrupt contract holds.
- **Registry.** The always-merge package list is a nested-`<value>` table, so the registry grows a list-holding `OptionValue` variant with XML serialization rather than flattening the entries into one string option.

# Acceptance criteria

- Golden fixture + test file per option under `crates/core/tests/options/` (e.g. `names_count_to_use_import_on_demand.rs`, `packages_to_use_import_on_demand.rs`, `use_single_class_imports.rs`): member imports of one owner collapse into `import static pkg.Owner.*;` at the NAMES count; listed-package single-type imports collapse into `pkg.*` below the class count; `USE_SINGLE_CLASS_IMPORTS` = `false` prefers on-demand imports wherever the guards allow, keeping the first merged import's position.
- The conservative guards still hold: no merge with a wildcard import present, on ambiguity, or against same-name top-level types.
- Absent-option schemes keep today's output and the whole suite stays green (`cargo test`); the new goldens are idempotent.
- The three `docs/settings/java.md` rows flip to ✅ and the README conservative-merging note is updated.

# Implementation plan

## Approach

Three layers: configuration/registry, the merge engine, then tests + docs.

**Dependency / sequencing.** This CR shares machinery with the sibling
`import-ordering-and-layout.md`: both need a non-scalar `OptionValue` variant
and the nested-`<value>` XML parse/serialize path in `crates/core/src/config.rs`,
both touch the import region of `crates/core/src/formatter.rs` (`imports()` /
`merge_on_demand_imports`), and both flip rows in the same
`docs/settings/java.md` "Imports" table. Recommend landing the ordering CR
first: its layout pass consumes exactly the merged import list this CR
extends, it defines the registry/`<value>` machinery (its acceptance criteria
pin the parse/serialize round trip), and it owns the "Import-table format"
prose. This CR then adds its list variant and the three merge rules on top.
The halves compose either way — layout only moves whole import lines, merge
only replaces a group of single imports with one wildcard line at the first
import's position — and default/absent schemes stay byte-identical under both,
so this is a sequencing preference, not a hard block. If this CR lands first,
it introduces the nested-`<value>` machinery itself, shaped so the ordering
CR's `<package>`/`<emptyLine>` table can reuse the same `XmlOption` extension
and per-line-indenting serializer.

**Configuration (`crates/core/src/config.rs`).** `JavaStyle` gains three fields
under the imports group (L148-150) with `Default` values (L179) from the
java.md table: `names_count_to_use_import_on_demand: u32` = `3`,
`packages_to_use_import_on_demand: Vec<String>` = `["java.awt",
"javax.swing"]` (package prefixes, no `.*`), and `use_single_class_imports:
bool` = `true`. Three `OptionDef` entries follow the
`CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND` entry (L553-566): `Section::JavaCodeStyle`
(the `<JavaCodeStyleSettings>` block, matching java.md), group "Imports", with
the same defaults and get/set closures over the new fields.

`OptionValue` (L202-208) grows a list-holding variant, e.g.
`OptionValue::Packages(Vec<String>)` storing package prefixes; the `.*` suffix
is an XML-boundary concern. A `Vec` is not `Copy`, so drop `Copy` from the
derive (keep `Debug, Clone, PartialEq, Eq`); `WrapStyle`/`BraceStyle` stay
`Copy`. Contained fallout:

- `parse_codestyle` (L696-719) matches on `def.default` — moving out of a
  `&OptionDef` works today only because of `Copy`, so switch to `match
  &def.default` (deref the scalar defaults); the new arm reads the nested list.
- `serialize_codestyle` (L740-761) compares `value == def.default` — becomes
  `&value == &def.default`; the new variant emits a multi-line nested fragment.
- `crates/gui/src/main.rs` `option_row` (L133-180) matches exhaustively on
  `&mut OptionValue` and re-sets the value; add a `Packages` arm (a multi-line
  text box, one package per line) so the crate compiles. The registry
  iteration renders the new option automatically, so no other GUI change is
  needed.

**Nested-`<value>` XML.** The mirror struct `XmlOption` (L573-580) requires the
`value` attribute, so a scheme carrying the real IntelliJ shape for this
option — `<option name="PACKAGES_TO_USE_IMPORT_ON_DEMAND"><value><list><option
value="java.awt.*"/>…</list></value></option>` — currently fails the whole
parse with `missing field @value` (verified empirically by feeding such a
scheme to the CLI). Extend the mirror: make `@name`/`@value` optional
(`#[serde(default)]`) and add an optional nested capture (a small `XmlValue` /
`XmlList` pair holding the `<option value="pkg.*"/>` entries). Scalar options
keep reading the attribute as today; `Packages` options read the child tree;
unimplemented nested options stay safely ignored (R7). This also fixes the R7
robustness gap for any real-world scheme that contains a nested-valued option.
`OptionMap` (L629-664) gains `get_packages(name, default) -> Vec<String>`
(parse the entries, strip a trailing `.*`). Serialization writes the nested
form only when the value differs from the default, emitting entries with the
`.*` suffix; the four section writers must indent *every* line of a fragment
by the section prefix (today only the first line is prefixed) so nested output
nests correctly under `<JavaCodeStyleSettings>`.

The exact IntelliJ export shape must be pinned during implementation against a
real export when one is available (the java.md "Import-table format" section
currently shows the layout-table `<package>`/`<emptyLine>` shape and claims
both options share it); if the captured PACKAGES shape differs, align the doc
prose — coordinating with the ordering CR, which owns the layout-table half of
that section.

**Merge engine (`crates/core/src/formatter.rs` `merge_on_demand_imports`,
L351-447).** Today `Entry` splits `import [static] <path>;` into `pkg` /
`simple` / `is_static` / `is_wildcard`, groups only non-static entries by
package, and never merges statics. Rework into one rule set that keeps the
conservative structure:

1. Wildcard guard unchanged: if any entry (static or not) is a wildcard,
   return the verbatim lines.
2. Non-static single-type imports, grouped by package: a package collapses to
   `import pkg.*;` (emitted at its first import's position) when the guards
   allow and any of: the package is in `packages_to_use_import_on_demand`
   (any count, incl. 1, i.e. below the class count); the group size exceeds
   `class_count_to_use_import_on_demand` (today's rule, "more than"); or
   `use_single_class_imports` is `false` (on-demand preferred, any non-empty
   group collapses).
3. Static member imports: new groups keyed by owner (`e.pkg` is
   `pkg.Owner` for `import static pkg.Owner.m;`). A group collapses to
   `import static pkg.Owner.*;` at its first member's position when its size
   exceeds `names_count_to_use_import_on_demand` and the guards allow.
4. Guards inherited per the decisions: collapse only when every simple name
   in the group maps to exactly one package/owner (dropping a single import
   could otherwise hand name precedence to a remaining same-name single
   import from another package), no name collides with a `local_types`
   top-level name, and no wildcard is present. The change is whitespace-only
   (R5), unmodelled shapes stay verbatim (R4), and re-formatting is stable
   (R6).

The `merge_on_demand_imports` doc comment (L341-350, "Static imports are never
merged") and the `imports()` call site (L317-339) are updated; the
`java.*`/`javax.*` blank-line split in `imports()` is untouched (this CR does
not cover ordering — see the sibling request).

Default/absent behaviour: no existing fixture imports from `java.awt` /
`javax.swing`, and the only same-owner static group in the suite
(`class_count_to_use_import_on_demand/static_never_merged`, 3 members) sits at
— not above — the default names count, so no existing golden changes.
`cargo test` verifies. The class-count file's `static_imports_are_never_merged`
case is now governed by `NAMES_COUNT_TO_USE_IMPORT_ON_DEMAND` rather than
"never": fold it into the new names-count test file (its fixture is the
below-threshold case) so each option file tests only its own option (R9).

**Tests.** Per the AGENTS.md hard rules: one file per option at
`crates/core/tests/options/<XML_OPTION>.rs` (`names_count_to_use_import_on_demand.rs`,
`packages_to_use_import_on_demand.rs`, `use_single_class_imports.rs`) with doc
header `//! <XML_OPTION> — …` plus `//! Fixtures live under
tests/java/<option>/.`, opening `use super::common::*;`, wired alphabetically
into `tests/options.rs`; fixtures under `tests/java/<option>/` as golden pairs
via `include_str!` and `assert_eq!(format_with(INPUT, &style), GOLDEN)` (or
`format(INPUT)` for default-style cases). No `parse_codestyle` tests and no new
common helpers; idempotency is asserted by re-formatting each new `*.out.java`
with the same style and comparing it to the golden.

## Steps

- [x] config.rs: add the three `JavaStyle` fields and `Default` values, plus the
      three `OPTIONS` entries (`Section::JavaCodeStyle`, group "Imports",
      defaults `3` / `["java.awt", "javax.swing"]` / `true`); `cargo check`
      (AC: absent option → default, config mapping).
- [x] config.rs: add `OptionValue::Packages(Vec<String>)`, drop `Copy` from the
      derive, and fix the mechanical fallout — `parse_codestyle` matches on
      `&def.default`, `serialize_codestyle` compares `&value == &def.default`;
      adapt the new entries' get/set closures.
- [x] config.rs: extend the XML mirrors so `XmlOption` tolerates a missing
      `value` attribute and captures the nested `<value><list><option
      value="pkg.*"/></list></value>` child; add `OptionMap::get_packages` and
      the parse arm; extend the serializer to emit the nested fragment (entries
      with `.*`) only when the value differs from the default, indenting every
      line of a fragment per section. Verify by hand that a scheme with a
      nested-valued option now parses (previously `missing field @value`) and
      that `parse_codestyle(serialize_codestyle(style)) == style` for a
      non-default package list (per AGENTS no committed `parse_codestyle`
      test).
- [x] crates/gui: add the `OptionValue::Packages` arm to `option_row` (multi-line
      editor, one package per line) so the exhaustive match compiles; `cargo
      test` builds the GUI crate.
- [x] formatter.rs: extend `merge_on_demand_imports` — static member groups by
      owner collapsing to `import static pkg.Owner.*;` above the NAMES count,
      listed-package single-type imports collapsing to `pkg.*` at any count,
      and `use_single_class_imports == false` collapsing any non-empty
      non-static group; keep the wildcard / ambiguity / local-name guards for
      both kinds; update the function and `imports()` doc comments; run
      `cargo test` and confirm no existing golden changed (AC: guards hold,
      absent-option output unchanged).
- [x] Reconcile `class_count_to_use_import_on_demand.rs`: fold the now-stale
      `static_imports_are_never_merged` case (and its fixture) into the new
      names-count test file as the below-threshold case, so the class-count
      file no longer claims statics are never merged (R9).
- [x] Add `tests/options/names_count_to_use_import_on_demand.rs` + fixtures
      under `tests/java/names_count_to_use_import_on_demand/`: collapse above
      the count (`import static a.one.Methods.*;` at the first member's
      position), threshold respected, wildcard-present guard, same-member-name
      ambiguity guard, local-name guard, and a default-style below-threshold
      case (AC: member imports of one owner collapse at the NAMES count).
- [x] Add `tests/options/packages_to_use_import_on_demand.rs` + fixtures under
      `tests/java/packages_to_use_import_on_demand/`: listed package collapses
      below the class count (incl. a single import), unlisted packages stay
      single-class, the default list collapses a lone `java.awt` import under
      the default style (`format`), and the wildcard/ambiguity/local guards
      hold (AC: listed-package collapse; guards).
- [x] Add `tests/options/use_single_class_imports.rs` + fixtures under
      `tests/java/use_single_class_imports/`: `false` prefers on-demand for
      ordinary packages below the class count (incl. count 1) keeping each
      merged import's first position; `true` (default) keeps single imports
      below the count; guards hold when `false`; an absent-option default case
      pins unchanged output (AC: on-demand preference + position; guards;
      absent-option output).
- [x] Wire the three modules alphabetically into `tests/options.rs` (names
      before `new_line_after_lparen_in_record_header`, packages after it,
      `use_single_class_imports` between `tab_size` and `use_tab_character`);
      assert idempotency of each new golden by re-formatting the `*.out.java`
      with the same style (AC: goldens idempotent, suite green).
- [x] If an IntelliJ installation is available, reformat representative
      static/package/single-class snippets there to cross-check the collapse
      thresholds and capture the real `PACKAGES_TO_USE_IMPORT_ON_DEMAND` XML
      shape; align goldens and the java.md format prose accordingly and record
      the outcome in the changelog.
- [x] Docs + full suite: flip `USE_SINGLE_CLASS_IMPORTS`, `NAMES_COUNT_…` and
      `PACKAGES_TO_USE_IMPORT_ON_DEMAND` to ✅ in `docs/settings/java.md` and
      refresh the "Imports" intro sentence; add the three rows to the README
      honoured-options table and rewrite the conservative-merging note (the
      "never merges static imports" claim), updating the GUI controls sentence
      for the new list control; add a requirement row to `docs/requirements.md`
      (next free number — R16 unless `wrapping-expressions-and-statements` has
      taken it, then R17) and extend the milestone paragraph; append the
      `docs/dev/changelog.md` entry; run `cargo test` and confirm the whole
      workspace suite is green with default-scheme output unchanged (AC:
      docs marks, README note, suite green).

## Verification

- `cargo build --workspace` succeeds (the GUI `Packages` arm is required for
  compilation).
- `cargo test --workspace`: 696 passed, 0 failed (was 675 before this change).
  New per-option files: `names_count_to_use_import_on_demand.rs` (7 tests),
  `packages_to_use_import_on_demand.rs` (8 tests) and
  `use_single_class_imports.rs` (7 tests), each golden asserted idempotent by
  re-formatting. Pre-existing golden changes are limited to the class-count
  re-fold (the stale `static_imports_are_never_merged` case moved to the
  names-count file as its below-threshold case) plus the four
  `import_layout_table` goldens whose lone `javax.swing.JButton` inputs now
  merge to `javax.swing.*` under the default always-merge package list — the
  fixture trip the decisions anticipated (default-scheme output stays
  byte-identical "except where a fixture trips a new default threshold"), and
  the layout ordering semantics those tests pin are unchanged.
- parse(serialize(style)) == style for a non-default package list verified by
  hand (the nested `<list>` with `.*` entries survives the round trip exactly;
  an explicitly empty list round-trips; a scheme mixing the package list with
  the layout table's `<value>` `<package>`/`<emptyLine>` children parses). No
  IntelliJ installation was available to cross-check the collapse thresholds or
  the captured XML shape; the pinned semantics follow the docs/settings table.

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
