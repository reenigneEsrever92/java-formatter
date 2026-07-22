---
type: ChangeRequest
kind: feature
title: Honour the spacing-around-separators options
description: Apply comma/semicolon/colon/question-mark spacing options, including the Java for-each colon option.
state: proposed
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
