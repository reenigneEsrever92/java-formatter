---
type: ChangeRequest
kind: feature
title: Honour the before-parentheses/braces/keywords spacing options
description: Apply the SPACE_BEFORE_* options that control the gap before parens, braces and clause keywords.
state: proposed
priority: high
tags: [dev, formatter]
owner: maintainer
---

# Problem

Every row of the `docs/settings/common.md` "Spaces / Before parentheses, braces, keywords" table is marked ❌: java-formatter parses none of them and emits a fixed canonical gap — clause keywords joined to their parens and braces, method and call parens tight to the name, braces tight after `)`, and `else` / `while` / `catch` / `finally` tight after `}` — that cannot be adjusted. A scheme that sets any of these options is only partially honoured; unimplemented options are safely ignored (R7 in `docs/requirements.md`).

# Proposal

Parse the following options into `config::JavaStyle` via the `OPTIONS` registry in `crates/core/src/config.rs`: `SPACE_BEFORE_METHOD_CALL_PARENTHESES`, `SPACE_BEFORE_METHOD_PARENTHESES`, `SPACE_BEFORE_IF_PARENTHESES`, `SPACE_BEFORE_WHILE_PARENTHESES`, `SPACE_BEFORE_FOR_PARENTHESES`, `SPACE_BEFORE_TRY_PARENTHESES`, `SPACE_BEFORE_CATCH_PARENTHESES`, `SPACE_BEFORE_SWITCH_PARENTHESES`, `SPACE_BEFORE_SYNCHRONIZED_PARENTHESES`, `SPACE_BEFORE_ANOTATION_PARAMETER_LIST` — spelled exactly as in IntelliJ sources, typo included — plus the brace options `SPACE_BEFORE_CLASS_LBRACE`, `SPACE_BEFORE_METHOD_LBRACE`, `SPACE_BEFORE_IF_LBRACE`, `SPACE_BEFORE_ELSE_LBRACE`, `SPACE_BEFORE_WHILE_LBRACE`, `SPACE_BEFORE_FOR_LBRACE`, `SPACE_BEFORE_DO_LBRACE`, `SPACE_BEFORE_SWITCH_LBRACE`, `SPACE_BEFORE_TRY_LBRACE`, `SPACE_BEFORE_CATCH_LBRACE`, `SPACE_BEFORE_FINALLY_LBRACE`, `SPACE_BEFORE_SYNCHRONIZED_LBRACE`, `SPACE_BEFORE_ARRAY_INITIALIZER_LBRACE`, `SPACE_BEFORE_ANNOTATION_ARRAY_INITIALIZER_LBRACE` and the keyword options `SPACE_BEFORE_ELSE_KEYWORD`, `SPACE_BEFORE_WHILE_KEYWORD`, `SPACE_BEFORE_CATCH_KEYWORD`, `SPACE_BEFORE_FINALLY_KEYWORD` — each an `OptionDef` with the IntelliJ built-in default from the table (absent → default), mapped to the existing `OptionValue::Bool`. In `crates/core/src/formatter.rs`, each toggle controls the gap before the paren / brace / keyword it names.

Docs touched: on delivery the implementation flips the ❌ marks to ✅ for these rows in `docs/settings/common.md`, updates the README honoured-options table and formatting-behaviour notes, adds a requirement row to `docs/requirements.md`, and appends `docs/dev/changelog.md`.

# Decisions

1. **One family, one request.** Only the listed options are added; unlisted spacing options stay unimplemented and safely ignored (R7).
2. **Defaults.** Split by group: the keyword, paren and brace options default `true` — equal to today's canonical gap — while `SPACE_BEFORE_METHOD_CALL_PARENTHESES`, `SPACE_BEFORE_METHOD_PARENTHESES`, `SPACE_BEFORE_ANOTATION_PARAMETER_LIST`, `SPACE_BEFORE_ARRAY_INITIALIZER_LBRACE` and `SPACE_BEFORE_ANNOTATION_ARRAY_INITIALIZER_LBRACE` default `false`, so absent/default schemes keep byte-identical goldens.
3. **Semantics.** Whitespace-only change (R5); unmodelled shapes echoed verbatim (R4); inserting/removing one space is idempotent (R6).
4. **Per-construct granularity.** Each keyword, paren and brace is its own toggle; the keyword-gap options (`SPACE_BEFORE_ELSE_KEYWORD`, `SPACE_BEFORE_WHILE_KEYWORD`, `SPACE_BEFORE_CATCH_KEYWORD`, `SPACE_BEFORE_FINALLY_KEYWORD`) control the `}` → keyword gap independently of the brace options.

# Acceptance criteria

- Per listed option: a golden fixture and per-option test file under `crates/core/tests/options/` testing the option's interesting values (on / off) plus the absent-option default.
- Toggle away from its default renders the gap accordingly in the `*.out.java` golden (e.g. `f (x)` with call-paren on, `if(x)` with if-paren off, `@Anno (…)`, `new int[] {`, `} else {` variants).
- Default-scheme output is unchanged and the whole suite is green (`cargo test`); the new goldens are idempotent.
- `docs/settings/common.md` marks are flipped to ✅ and the README / `docs/requirements.md` / `docs/dev/changelog.md` are updated.
