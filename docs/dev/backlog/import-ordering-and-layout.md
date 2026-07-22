---
type: ChangeRequest
kind: feature
title: Order and group imports per the import layout options
description: Implement IMPORT_LAYOUT_TABLE and the import-section ordering/grouping options.
state: proposed
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
