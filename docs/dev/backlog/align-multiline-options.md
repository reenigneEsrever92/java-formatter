---
type: ChangeRequest
kind: feature
title: Honour the align-when-multiline options
description: Implement the common ALIGN_MULTILINE_* / ALIGN_* options so wrapped constructs align instead of using the plain continuation indent.
state: proposed
priority: medium
tags: [dev, formatter]
owner: maintainer
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
