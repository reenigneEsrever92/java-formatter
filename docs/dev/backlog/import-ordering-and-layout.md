---
type: ChangeRequest
kind: feature
title: Order and group imports per the import layout options
description: Implement IMPORT_LAYOUT_TABLE and the import-section ordering/grouping options.
state: planned
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

`CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND` merging is the only import option shipped today (✅ in docs/settings/java.md "Imports"), and the formatter hard-codes a blank line before the `java.*`/`javax.*` group (README formatting notes) — every ordering/grouping row is ❌: `IMPORT_LAYOUT_TABLE` (table; the default layout is documented in java.md), `LAYOUT_STATIC_IMPORTS_SEPARATELY`, `LAYOUT_ON_DEMAND_IMPORT_FROM_SAME_PACKAGE_FIRST`, `KEEP_BLANK_LINES_BETWEEN_IMPORTS`, `PRESERVE_MODULE_IMPORTS` and `DELETE_UNUSED_MODULE_IMPORTS`.
`IMPORT_LAYOUT_TABLE` serialises as nested `<package>`/`<emptyLine>` entries (format documented in java.md "Import-table format"), which the registry's `OptionValue` cannot represent — it has no table/String variant yet.
Schemes that set the family are only partially honoured (safely ignored today, R7).

# Proposal

Add an import-layout value to `JavaStyle` via an `OptionDef` entry in the `OPTIONS` registry in crates/core/src/config.rs: this family introduces an `OptionValue` table/String variant holding the ordered entries (module slot, `<package>` and `<emptyLine>`), defaulting to the built-in layout in java.md "Default layout", and the parse/serialize path must round-trip the nested `<value>` XML; the five booleans are ordinary entries in the same Java-specific section as `CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND` (defaults `true`/`true`/`false`/`true`/`false` from the table). Apply them in crates/core/src/formatter.rs on the merged import list: group and order imports per the table (static vs non-static separation, `java.*`/`javax.*` placement, blank lines from `<emptyLine>` entries), sort same-package on-demand imports first, preserve blank lines inside sections, and handle `import module` lines per the module options.

Docs touched: `docs/settings/java.md` "Imports" marks flipped ❌→✅ (keeping the "Import-table format" prose in sync), the README honoured-options table and the blank-line formatting note, `docs/requirements.md` (a new requirement row), and `docs/dev/changelog.md` on delivery.

# Decisions

1. **One family, one request.** Only the six listed options ship; `PACKAGES_TO_USE_IMPORT_ON_DEMAND` (table-typed) belongs to the import-on-demand-extensions request, and the remaining ❌ rows stay unimplemented and are ignored safely (R7).
2. **Defaults.** The table defaults to IntelliJ's built-in layout (java.md "Default layout") and the booleans to the documented defaults; the layout pass must reproduce today's single-blank-line-before-`java.*`/`javax.*` output for import lists that already follow the default ordering (the current import fixtures), keeping existing goldens green.
3. **Semantics.** Moving import lines is semantic-preserving (import order is not significant), so R5 holds; R4 is never violated because nothing is invented or dropped — the only sanctioned removals are `import module` lines, and they stay conservative: `PRESERVE_MODULE_IMPORTS` defaults to `true`, and with `DELETE_UNUSED_MODULE_IMPORTS` `true` only clearly unused module imports are removed (any doubtful case keeps the line). R6 holds because the canonical group order makes re-formatting stable.
4. **Registry.** The new table/String variant must round-trip parse(serialize(style)) == style for a non-default layout — the parse path is extended from quick-xml scalar `<option value="…">` shapes to read the `<value>` child tree for this option, and serialize writes the table only when it differs from the built-in default.

# Acceptance criteria

- `tests/options/import_layout_table.rs` (fixtures under `tests/java/import_layout_table/`) asserts a scheme whose table orders `java.*` first and adds/removes `<emptyLine>` entries reorders imports and blank lines accordingly; an absent table reproduces today's `java.*`/`javax.*` blank-line output on the existing import fixtures.
- parse(serialize(style)) == style for a non-default layout — the nested `<package/>`/`<emptyLine/>` entries survive the round trip.
- The boolean rows are asserted both ways: static imports in their own section vs inline (`LAYOUT_STATIC_IMPORTS_SEPARATELY`), same-package on-demand imports first (`LAYOUT_ON_DEMAND_IMPORT_FROM_SAME_PACKAGE_FIRST`), user blank lines preserved (`KEEP_BLANK_LINES_BETWEEN_IMPORTS`), and module imports kept by default / dropped only when clearly unused with `DELETE_UNUSED_MODULE_IMPORTS` (conservative, R4/R5).
- Default-scheme import output is unchanged and the whole suite stays green (`cargo test`); the new goldens are idempotent (R6); docs/settings marks are flipped, the README blank-line note is updated, and `docs/requirements.md` / `docs/dev/changelog.md` are updated.

# Implementation plan

## Approach

Three layers, as with the sibling option-family requests: configuration and
XML in `crates/core/src/config.rs`, rendering in `crates/core/src/formatter.rs`,
then tests and docs. Scope boundary: `PACKAGES_TO_USE_IMPORT_ON_DEMAND`,
`NAMES_COUNT_TO_USE_IMPORT_ON_DEMAND` and `USE_SINGLE_CLASS_IMPORTS` belong to
`import-on-demand-extensions.md` (which builds on this request and, per its
plan, recommends this one lands first), import-on-demand merging itself is not
touched, and the blank-line-policy family
(`KEEP_BLANK_LINES_IN_*` / `BLANK_LINES_*`, which explicitly leaves the
import-group separator to this request) stays out.

**Configuration — `crates/core/src/config.rs`.** `JavaStyle` (L105-150) is
constructed only via `Default`, so add six fields under the `// --- imports ---`
banner (L148) with defaults from the java.md table: `import_layout:
Vec<ImportLayoutEntry>` = the built-in layout of java.md "Default layout", plus
five bools — `layout_static_imports_separately` = `true`,
`layout_on_demand_import_from_same_package_first` = `true`,
`keep_blank_lines_between_imports` = `false`, `preserve_module_imports` =
`true`, `delete_unused_module_imports` = `false`. `ImportLayoutEntry` is a new
public enum (name / `with_subpackages` / `is_static` / `module` vs `EmptyLine`)
exported beside `JavaStyle`; the built-in default list holds the reserved
module slot first, then the empty-name non-static catch-all, `<emptyLine>`,
`javax` (withSubpackages), `java` (withSubpackages), `<emptyLine>`, and the
empty-name static catch-all — exactly java.md "Default layout". Because the
sibling request extends the same registry, the built-in list is shared as one
construction site so `JavaStyle::default` and the option's default never
diverge.

Six `OptionDef` entries follow `CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND`
(L553-566): `Section::JavaCodeStyle` (all six serialize under
`<JavaCodeStyleSettings>`, matching where `CLASS_COUNT_…` lives in
codestyle.xml and java.md), GUI group "Imports", get/set closures over the new
fields. `OptionValue` (L202-208) grows `OptionValue::ImportLayout(
Vec<ImportLayoutEntry>)`; a `Vec` is not `Copy`, so drop `Copy` from the derive
(keep `Debug, Clone, PartialEq, Eq`). Contained fallout, as analysed in the
sibling request: `parse_codestyle` (L696-719) matches on `def.default` — moving
out of a `&OptionDef` works today only because of `Copy`, so switch to `match
&def.default` (deref the scalar defaults) with a new arm for the layout; and
`serialize_codestyle` (L740-761) compares each option's value to its default.
For list-typed options the built-in default cannot be a `static` literal, so the
serializer compares `value` against `(def.get)(&JavaStyle::default())`
computed once per call instead of against `def.default` (identical for all
existing scalar options, so minimal-scheme output is unchanged), and
`OptionDef::default` for the layout entry is a `Vec::new()` type tag whose real
value lives in `JavaStyle::default` — documented on `OptionDef` (L210-229).
Absent from a scheme → field keeps the built-in default (R7).

**Nested-`<value>` XML — `crates/core/src/config.rs`.** The serde mirror
`XmlOption` (L573-580) requires the `value` attribute, so any real IntelliJ
scheme carrying the import table currently fails the whole parse with `missing
field @value` (same robustness gap the sibling request notes). Extend the
mirror: make `@name` / `@value` optional (`#[serde(default)]`), so scalar
options parse as today, nested-valued options no longer abort the parse, and
unimplemented nested options stay safely ignored (R7). The layout's
`<value>` child interleaves `<package …/>` and `<emptyLine/>` in document
order, which the serde mirror cannot preserve across two tag-typed `Vec`s, so
read that one option's `<value>` subtree with quick-xml's event API
(order-preserving scan inside the `<JavaCodeStyleSettings>` block) and decode
`name` / `withSubpackages` / `static` / `module` attributes per entry; absent
option → no change (field keeps the built-in default). Serialization emits the
nested form

```xml
<option name="IMPORT_LAYOUT_TABLE">
  <value>
    <package name="java" withSubpackages="true" static="false" />
    <emptyLine />
  </value>
</option>
```

only when the layout differs from the built-in default, and the four section
writers indent *every* line of a multi-line fragment by the section prefix
(today only the first line is prefixed), so nested output nests correctly
under `<JavaCodeStyleSettings>`. The parse and serialize sides share the one
entry model, which is what makes `parse(serialize(style)) == style` exact
(AC2); per the AGENTS.md hard rules there is no committed `parse_codestyle`
test — the round trip is verified by hand during implementation and recorded
in the changelog (see step 2).

**GUI — `crates/gui/src/main.rs`.** `option_row` (L133-180) matches
exhaustively on `&mut OptionValue` and re-sets the value, so the new variant
needs an arm for the crate to compile: render a read-only summary (option
count + "import layout"), leaving the value untouched so `set` writes it back
unchanged; a full table editor is out of scope. `cargo test` builds the GUI
crate, so the arm is required.

**Rendering — `crates/core/src/formatter.rs`.** Today `imports()` (L317-339)
merges via `merge_on_demand_imports` (L351-447) and then hard-codes one split:
non-`java`/`javax` lines, a blank line, then the `java`/`javax` lines. Replace
that tail with a table-driven layout pass over the *merged* list (the sibling
request's merge extensions compose above it unchanged):

- Recover, per merged line, the classification the layout needs — module /
  static / non-static, package, on-demand (`.*`) — from the line text, plus
the blank-line count between it and the preceding import in the source. Blank
counts come from the byte gap between consecutive `import_declaration` nodes
(node `end_byte()`/`start_byte()` into `self.src`); `merge_on_demand_imports`
is extended (private helper) to annotate each emitted line with the index of
the import node it came from (a collapsed wildcard keeps its first import's
index) so the layout pass can attach source gaps. Default
`KEEP_BLANK_LINES_BETWEEN_IMPORTS` is `false`, so gaps are dropped — today's
output — and only the option turns them on (AC3).
- Matching a line to a table entry: a non-empty-name entry matches when its
package equals the import's package, or is a prefix of it when
`withSubpackages`; among matching entries the longest name wins; the
empty-name entry is the catch-all fallback for lines no named entry matches.
When `LAYOUT_STATIC_IMPORTS_SEPARATELY` is `true`, static lines match only
entries with `static="true"` and non-static lines only `static="false"`;
when `false`, the `static` attribute is ignored so static imports join the
ordinary package sections (this is what "inline" means in AC3). Module lines
match the reserved module entry. This longest-prefix model is what lets the
default table (catch-all listed before `javax`/`java`) still send
`java.*`/`javax.*` to their named entries, so the default layout reproduces
today's blank-line-before-`java`/`javax` output on the existing import
fixtures (AC1/AC4) — every fixture that imports today sits under
`tests/java/class_count_to_use_import_on_demand/` and none mixes static with
non-static or `java` with `javax`, so none changes.
- Output order is the table's entry order; an empty group is skipped. Blank
lines between two emitted groups equal the number of `<emptyLine/>` entries
strictly between their table positions (so a custom table that adds/removes
`<emptyLine/>`s changes the gap, AC1); the default table's single separators
reproduce the current goldens, and there is no trailing blank after the last
group (the `program()` boundary supplies the one before the first type).
Group-internal order is preserved (imports are not alphabetically sorted),
so the canonical output is stable and re-formatting is a no-op (R6).
- `LAYOUT_ON_DEMAND_IMPORT_FROM_SAME_PACKAGE_FIRST` (`true` by default):
within a group, on-demand imports whose package equals the file's own package
(threaded from `program()`'s `package_declaration`) move before the group's
other lines, preserving relative order; no package declaration → no reorder.
None of today's fixtures has an own package or mixes wildcards, so default
output is unchanged.
- `import module …;` is not a production of tree-sitter-java 0.23.5 (verified:
it parses as an `ERROR` node and today is echoed verbatim as top-level
garbage). Handle module imports at the source level instead: scan the file's
import region (leading blank/comment/package/import lines before the first
type) for lines matching an `import module <name>;` pattern, collect their
full line text, and blank them out in place with equal-length spaces *before*
parsing so the section parses cleanly (no parse-error goldens, positions and
diagnostics unaffected). Thread the collected lines into `imports()` as the
module slot group. Semantics per the request decisions:
  - `PRESERVE_MODULE_IMPORTS = true` (default): module lines are kept, placed
    at the module slot position (reserved module entry; when a custom table
    omits it, at the head of the import section).
  - `PRESERVE_MODULE_IMPORTS = false`: module lines are not preserved and are
    removed — the one removal the request sanctions.
  - `DELETE_UNUSED_MODULE_IMPORTS = true` (with preserve on): remove only
    module imports that are *clearly unused*. Without symbol resolution the
    only provable case is a repeated identical `import module` line (a
    duplicate adds nothing), so duplicates beyond the first are dropped and
    every other case is doubtful and kept (conservative, R4/R5). The exact
    predicate is cross-checked against IntelliJ when one is available and
    recorded in the changelog.

If an IntelliJ installation is available to the implementer, format
representative import sections there to cross-check the group order, the
`<emptyLine/>` blank rule, the `java`-vs-`javax` split and the module-option
semantics, and align the goldens; otherwise record that no reference was
available in the changelog, as the sibling requests did.

**Tests.** Follow the AGENTS.md hard rules: one file per option under
`crates/core/tests/options/<XML_OPTION>.rs` — `import_layout_table.rs`,
`layout_static_imports_separately.rs`,
`layout_on_demand_import_from_same_package_first.rs`,
`keep_blank_lines_between_imports.rs`, `preserve_module_imports.rs`,
`delete_unused_module_imports.rs` — each opening `use super::common::*;`, doc
header `//! <XML_OPTION> — …` plus `//! Fixtures live under
tests/java/<option>/.`, fixtures as golden pairs via `include_str!`, wired
alphabetically into `tests/options.rs` (`delete_unused_module_imports` and
`import_layout_table` between `continuation_indent_size` and `indent_size`,
`keep_blank_lines_between_imports` between `indent_size` and
`keep_simple_blocks_in_one_line`, the two `layout_*` between
`keep_simple_methods_in_one_line` and `method_brace_style`,
`preserve_module_imports` between `new_line_after_lparen_in_record_header`
and `record_components_wrap`). No `parse_codestyle` tests, no new common
helpers; a test file may define a tiny local helper (e.g. to build a custom
`import_layout` `Vec`, as `binary_operation_wrap.rs` does with `narrow`).
Each file asserts `format_with(INPUT, &style)` against a golden (or
`format(INPUT)` for the absent-option default case) and idempotency is
checked by re-formatting each new `*.out.java` under the same style during
development.

**Docs.** Flipped on delivery: the six ❌ rows of the "Imports" table in
`docs/settings/java.md` (keeping the "Import-table format" prose in sync),
README honoured-options table + formatting-behaviour note (the
"blank line before `java.*`/`javax.*`" bullet becomes the import-layout
description), `docs/requirements.md` (new row + milestone paragraph),
`docs/dev/changelog.md` entry.

## Steps

- [ ] `crates/core/src/config.rs`: add the six `JavaStyle` fields with their
      `Default` values (built-in layout list from java.md "Default layout" +
      the five bools), the `ImportLayoutEntry` type, and the six `OptionDef`s
      (`Section::JavaCodeStyle`, group "Imports", after
      `CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND`); `cargo build` and the suite stay
      green (AC4 absent → default; config mapping).
- [ ] `crates/core/src/config.rs`: add `OptionValue::ImportLayout(
      Vec<ImportLayoutEntry>)`, drop `Copy` from the derive, and fix the
      mechanical fallout — `parse_codestyle` matches on `&def.default`;
      `serialize_codestyle` compares each value against `(def.get)(&
      JavaStyle::default())` so list-typed options still serialize only when
      non-default; update the `OptionDef`/`OptionValue` doc comments (AC2
      structural round-trip guarantee, AC4 minimal scheme).
- [ ] `crates/core/src/config.rs`: extend the XML mirrors so `XmlOption`
      tolerates a missing `value` attribute (nested-valued options no longer
      abort the whole parse, R7); add the order-preserving event-API reader
      for the layout option's `<value>` `<package>`/`<emptyLine>` children and
      the parse arm; extend the serializer to write the nested fragment only
      when the layout differs from the built-in default, indenting every line
      of a fragment per section. Verify by hand that a scheme carrying the
      java.md import-table XML now parses and that
      `parse(serialize(style)) == style` for a non-default layout — nested
      entries survive — recording the check in the changelog (AC2; per AGENTS
      no committed `parse_codestyle` test).
- [ ] `crates/gui/src/main.rs`: add the `OptionValue::ImportLayout` arm to
      `option_row` (read-only summary; value passed back unchanged) so the
      exhaustive match compiles; `cargo test` builds the GUI crate (AC4).
- [ ] `crates/core/src/formatter.rs`: extend `merge_on_demand_imports` to
      annotate each emitted line with its source import-node index, then
      replace the hard-coded third-party/`java` split in `imports()` with the
      table-driven layout pass (longest-prefix matching, group order,
      `<emptyLine/>` blank rule, no trailing blank); wire the built-in default
      layout and confirm on `cargo test` that every existing import golden
      under `class_count_to_use_import_on_demand/` is byte-identical (AC1
      absent-table default, AC4).
- [ ] `crates/core/src/formatter.rs`: honour `LAYOUT_STATIC_IMPORTS_SEPARATELY`
      (static section vs inline per the entry `static` attribute) and
      `LAYOUT_ON_DEMAND_IMPORT_FROM_SAME_PACKAGE_FIRST` (same-package
      on-demand lines to the front of their group, using the file's package
      threaded from `program()`); defaults keep today's output (AC3 for these
      two rows).
- [ ] `crates/core/src/formatter.rs`: honour `KEEP_BLANK_LINES_BETWEEN_IMPORTS`
      using the per-line source-gap blank counts — false (default) drops them
      as today, true preserves them within each resulting group (AC3).
- [ ] `crates/core/src/formatter.rs`: implement module-import handling —
      line-pattern recognition in the import region, equal-length blanking
      before parse so no ERROR node is produced, module-slot placement when
      preserved, removal when `PRESERVE_MODULE_IMPORTS` is `false`, and
      clearly-unused-only removal (duplicates beyond the first) when
      `DELETE_UNUSED_MODULE_IMPORTS` is `true`; default keeps the lines (AC3,
      R4/R5).
- [ ] Add `tests/options/import_layout_table.rs` + fixtures under
      `tests/java/import_layout_table/`: a custom table ordering `java.*`
      first (with a changed `<emptyLine/>` set) reorders imports and blanks
      accordingly; a table with an extra/removed `<emptyLine/>` shifts the
      group gap; an absent-table default check (`format`) reproduces today's
      output on an import list shaped like the existing fixtures (AC1).
- [ ] Add `tests/options/layout_static_imports_separately.rs` + fixtures under
      `tests/java/layout_static_imports_separately/`: a mixed static/
      non-static import list with the static lines in their own final section
      (`true`, the default, via an absent-option golden) vs inline with the
      ordinary sections (`false`) (AC3).
- [ ] Add `tests/options/layout_on_demand_import_from_same_package_first.rs`
      + fixtures under `tests/java/layout_on_demand_import_from_same_package_first/`:
      a packaged file with an own-package on-demand import among other
      wildcards, asserted at `true` (default golden) and `false`, plus an
      absent-option default check (AC3).
- [ ] Add `tests/options/keep_blank_lines_between_imports.rs` + fixtures under
      `tests/java/keep_blank_lines_between_imports/`: an import list with
      user blank lines inside one group (kept below the merge threshold so the
      option is exercised in isolation), asserted at `true` (preserved) and
      `false`/absent (dropped, today's layout) (AC3).
- [ ] Add `tests/options/preserve_module_imports.rs` and
      `tests/options/delete_unused_module_imports.rs` + fixtures under
      `tests/java/preserve_module_imports/` and
      `tests/java/delete_unused_module_imports/`: module lines kept by default
      and placed in the module slot; dropped when preserve is `false`; a
      duplicated module line dropped (only the first kept) when delete-unused
      is `true` while a single doubtful module line is kept (AC3; R4/R5).
- [ ] Wire the six modules alphabetically into `tests/options.rs` (per the
      positions in the approach); re-format each new `*.out.java` under its
      own style to confirm the goldens are idempotent (R6); `cargo test`
      green with no existing golden changed (AC4).
- [ ] If an IntelliJ installation is available, format representative import
      sections there to cross-check the group order, blank-line and
      module-option semantics and the export shape of the table; align the
      goldens and the java.md prose accordingly and record the outcome in the
      changelog.
- [ ] Docs + final suite: flip the six `docs/settings/java.md` "Imports" rows
      ❌ → ✅ and refresh the "Imports" intro sentence and "Import-table
      format" prose; add the six options to the README honoured-options table
      and rewrite the blank-line formatting-behaviour note (plus the GUI
      sentence if the read-only layout summary warrants it); add a new
      requirement row to `docs/requirements.md` (next free number — R16
      unless an earlier-landing request has taken it, then R17) and extend the
      milestone paragraph; append the `docs/dev/changelog.md` entry; run
      `cargo test` once more and confirm the whole workspace suite is green
      (AC4).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
