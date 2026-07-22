---
type: ChangeRequest
kind: feature
title: Honour the remaining indentation options (labels, smart tabs, relative indents, per-construct indents)
description: Implement the indent options beyond INDENT_SIZE/CONTINUATION_INDENT_SIZE/TAB_SIZE/USE_TAB_CHARACTER.
state: proposed
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

`INDENT_SIZE`, `CONTINUATION_INDENT_SIZE`, `TAB_SIZE` and `USE_TAB_CHARACTER` already ship (docs/settings/common.md "Indent options"; the latter two per R13's tab-stop output model), but every other row there is ❌: `SMART_TABS`, `LABEL_INDENT_SIZE`, `LABEL_INDENT_ABSOLUTE`, `USE_RELATIVE_INDENTS`, `KEEP_INDENTS_ON_EMPTY_LINES`, and the per-construct `DECLARATION_PARAMETER_INDENT` / `GENERIC_TYPE_PARAMETER_INDENT` / `CALL_PARAMETER_INDENT` / `CHAINED_CALL_INDENT` / `ARRAY_ELEMENT_INDENT` ints (each default `-1` = use `CONTINUATION_INDENT_SIZE`); `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS` (common.md "Braces & indentation") is ❌ too.
Constructs therefore always use the base or continuation indent: a scheme's per-construct widths, `label:` indentation and tab/indent refinements are ignored (R7), so indentation fidelity is only partial.
The shipped tab-stop column arithmetic in formatter.rs (README `USE_TAB_CHARACTER` note) is where the tab-refining options must hook in.

# Proposal

Parse all eleven into `JavaStyle` via `OptionDef` entries in the `OPTIONS` registry in crates/core/src/config.rs — `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS` in the JAVA `codeStyleSettings` block; the ten `<indentOptions>` rows (bools, `LABEL_INDENT_SIZE`, and the signed `*_INDENT` widths) in the indent-options section, with the IntelliJ defaults from the docs/settings table (absent → default) — and apply them in crates/core/src/formatter.rs at the constructs they govern: top-level class-member indentation, `label:` indentation, and the per-construct continuation-indent overrides, while `SMART_TABS` / `USE_RELATIVE_INDENTS` / `KEEP_INDENTS_ON_EMPTY_LINES` refine tab and blank-line indent behaviour on top of the shipped tab-stop model.

Docs touched: `docs/settings/common.md` "Braces & indentation" and "Indent options" marks flipped ❌→✅, the README honoured-options table and formatting-behaviour notes, `docs/requirements.md` (a new requirement row), and `docs/dev/changelog.md` on delivery.

# Decisions

1. **One family, one request.** Only the rows listed above ship; the `ALIGN_MULTILINE_*` alignment family and the switch/case indentation options (`INDENT_CASE_FROM_SWITCH`, `CASE_STATEMENT_ON_NEW_LINE`, `INDENT_BREAK_FROM_CASE`) have their own requests and stay unimplemented here and safely ignored (R7).
2. **Defaults.** Fields take the IntelliJ built-ins (`false`, `LABEL_INDENT_SIZE` `0`, `*_INDENT` `-1` = inherit), so default and absent schemes keep byte-identical current output and the existing goldens stay green.
3. **Semantics.** R5 holds — these options change only leading whitespace, tabs and empty lines, never tokens; R4 echoes unmodelled shapes verbatim. `SMART_TABS` / `USE_RELATIVE_INDENTS` interplay with the shipped tab-stop column arithmetic (formatter.rs): margin and wrap decisions keep logical columns so wrapping points do not shift (R13), and R6 is pinned by re-formatting the new goldens.
4. **Registry.** The five `*_INDENT` options default to `-1`, which the registry's unsigned `OptionValue::UInt` cannot represent — this family introduces a signed `OptionValue::Int` variant (`JavaStyle` fields hold `-1` = inherit) so parse(serialize(style)) == style for both explicit widths and `-1`; the bools and `LABEL_INDENT_SIZE` fit the existing variants.

# Acceptance criteria

- One `tests/options/<option>.rs` per option (fixtures under `tests/java/<option>/`): `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS`, `SMART_TABS`, `LABEL_INDENT_SIZE`, `LABEL_INDENT_ABSOLUTE`, `USE_RELATIVE_INDENTS`, `KEEP_INDENTS_ON_EMPTY_LINES`, `DECLARATION_PARAMETER_INDENT`, `GENERIC_TYPE_PARAMETER_INDENT`, `CALL_PARAMETER_INDENT`, `CHAINED_CALL_INDENT`, `ARRAY_ELEMENT_INDENT` — bools asserted at `true`, each `*_INDENT` at an explicit width proving it overrides `CONTINUATION_INDENT_SIZE` for that construct kind only, labels under plain and absolute indents.
- Absent-option defaults (`-1` inherit, `false`, `0`) reproduce today's output byte-for-byte on the same fixtures; the whole suite stays green (`cargo test`).
- parse(serialize(style)) == style for the signed widths at explicit values and at `-1`; the tab-refinement goldens stay idempotent under the `USE_TAB_CHARACTER` tab-stop model (R13).
- docs/settings marks are flipped ❌→✅; the README, `docs/requirements.md` and `docs/dev/changelog.md` are updated with the implementation.
