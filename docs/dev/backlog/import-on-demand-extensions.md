---
type: ChangeRequest
kind: feature
title: Extend import-on-demand merging per the on-demand import options
description: Implement NAMES_COUNT_TO_USE_IMPORT_ON_DEMAND, PACKAGES_TO_USE_IMPORT_ON_DEMAND and USE_SINGLE_CLASS_IMPORTS.
state: proposed
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
