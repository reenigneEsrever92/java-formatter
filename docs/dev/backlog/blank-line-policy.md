---
type: ChangeRequest
kind: feature
title: Honour the blank-line policy options (KEEP_BLANK_LINES_*, BLANK_LINES_*)
description: Parse and apply the keep/minimum blank-line options so vertical spacing matches the scheme.
state: proposed
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
