---
type: ChangeRequest
kind: feature
title: Honour the within-parentheses/brackets/braces spacing options
description: Apply the SPACE_WITHIN_* options so padding inside parens, brackets, braces and array initialisers follows the scheme.
state: proposed
priority: high
tags: [dev, formatter]
owner: maintainer
---

# Problem

Every row of the `docs/settings/common.md` "Spaces / Within parentheses, brackets, braces" table is marked ❌: java-formatter parses none of them and emits no inner padding — conditions are rendered with exactly their own parentheses (README formatting-behaviour notes) and calls, declarations, casts, brackets, braces and array initialisers come out tight, a fixed canonical style that cannot be adjusted. A scheme that sets any of these options is only partially honoured; unimplemented options are safely ignored (R7 in `docs/requirements.md`).

# Proposal

Parse the following options into `config::JavaStyle` via the `OPTIONS` registry in `crates/core/src/config.rs`: `SPACE_WITHIN_PARENTHESES`, `SPACE_WITHIN_METHOD_CALL_PARENTHESES`, `SPACE_WITHIN_EMPTY_METHOD_CALL_PARENTHESES`, `SPACE_WITHIN_METHOD_PARENTHESES`, `SPACE_WITHIN_EMPTY_METHOD_PARENTHESES`, `SPACE_WITHIN_IF_PARENTHESES`, `SPACE_WITHIN_WHILE_PARENTHESES`, `SPACE_WITHIN_FOR_PARENTHESES`, `SPACE_WITHIN_TRY_PARENTHESES`, `SPACE_WITHIN_CATCH_PARENTHESES`, `SPACE_WITHIN_SWITCH_PARENTHESES`, `SPACE_WITHIN_SYNCHRONIZED_PARENTHESES`, `SPACE_WITHIN_CAST_PARENTHESES`, `SPACE_WITHIN_BRACKETS`, `SPACE_WITHIN_BRACES`, `SPACE_WITHIN_ARRAY_INITIALIZER_BRACES`, `SPACE_WITHIN_EMPTY_ARRAY_INITIALIZER_BRACES`, `SPACE_WITHIN_ANNOTATION_PARENTHESES` — each an `OptionDef` with the IntelliJ built-in default from the table (absent → default), mapped to the existing `OptionValue::Bool`. In `crates/core/src/formatter.rs`, each toggle inserts padding just inside the paren / bracket / brace kind it names, with the empty-vs-nonempty variants distinct where IntelliJ splits them.

Docs touched: on delivery the implementation flips the ❌ marks to ✅ for these rows in `docs/settings/common.md`, updates the README honoured-options table and formatting-behaviour notes, adds a requirement row to `docs/requirements.md`, and appends `docs/dev/changelog.md`.

# Decisions

1. **One family, one request.** Only the listed options are added; unlisted spacing options stay unimplemented and safely ignored (R7).
2. **Defaults.** All default `false`, which equals today's no-padding output, so absent or default schemes keep byte-identical goldens.
3. **Semantics.** Whitespace-only change (R5); unmodelled shapes echoed verbatim (R4); padding is idempotent (R6).
4. **Per-construct granularity.** Each paren / bracket / brace kind is its own toggle, so e.g. `SPACE_WITHIN_IF_PARENTHESES` affects only `if (...)`, and the empty variants (`SPACE_WITHIN_EMPTY_METHOD_CALL_PARENTHESES`, `SPACE_WITHIN_EMPTY_METHOD_PARENTHESES`, `SPACE_WITHIN_EMPTY_ARRAY_INITIALIZER_BRACES`) are independent of their non-empty counterparts.

# Acceptance criteria

- Per listed option: a golden fixture and per-option test file under `crates/core/tests/options/` testing the option's interesting values (on / off) plus the absent-option default.
- Toggle on → the governed paren / bracket / brace is padded in the `*.out.java` golden (`if( x )`, `f( args )`, `f( )`, `( Type ) expr`, `{ 1, 3, 5 }`, `a[ 0 ]`); off (and by default) → output stays tight.
- Default-scheme output is unchanged and the whole suite is green (`cargo test`); the new goldens are idempotent.
- `docs/settings/common.md` marks are flipped to ✅ and the README / `docs/requirements.md` / `docs/dev/changelog.md` are updated.
