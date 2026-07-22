---
type: ChangeRequest
kind: feature
title: Force braces on statement bodies per the *_BRACE_FORCE options
description: Implement IF_BRACE_FORCE, FOR_BRACE_FORCE, WHILE_BRACE_FORCE and DOWHILE_BRACE_FORCE.
state: proposed
priority: high
tags: [dev, formatter]
owner: maintainer
---

# Problem

The `*_BRACE_FORCE` rows of docs/settings/common.md "Force braces" are all ❌ — `IF_BRACE_FORCE`, `FOR_BRACE_FORCE`, `WHILE_BRACE_FORCE` and `DOWHILE_BRACE_FORCE` — valid IntelliJ options whose force codes (`0` do not force, `1` force when the body is multiline, `3` always force; docs/settings/index.md "Force-brace codes") java-formatter neither parses nor applies, so schemes setting them are only partially honoured and the options are ignored (R7). Today the formatter preserves whatever braces the input has and renders brace-less single-statement bodies as-is, so a scheme with "always force braces" is not honoured and output diverges from IntelliJ.

# Proposal

Parse `IF_BRACE_FORCE`, `FOR_BRACE_FORCE`, `WHILE_BRACE_FORCE` and `DOWHILE_BRACE_FORCE` into `JavaStyle` via the OPTIONS registry in crates/core/src/config.rs — `OptionDef` entries in the JAVA `codeStyleSettings` block with IntelliJ's built-in default `0` (do not force; absent → default), decoded through a dedicated force-code mapping alongside the existing `WrapStyle`/`BraceStyle` mappings. Apply them in crates/core/src/formatter.rs at the statement bodies they govern: when forcing, wrap the single-statement body of the `if`, `for`, `while` or `do … while` in `{ … }` with the statement indented one level; force-if-multiline (code `1`) adds braces only when the body already spans multiple lines.

Docs touched: `docs/settings/common.md` (❌ → ✅ for these rows), `README.md` (honoured-options table rows and formatting-behaviour notes), `docs/requirements.md` (a new requirement row) and `docs/dev/changelog.md` on completion.

# Decisions

1. **One family, one request.** Only the four `*_BRACE_FORCE` options are added; other brace/force options still ❌ stay unimplemented and are ignored safely (R7).
2. **Defaults.** IntelliJ's built-in default is `0` (do not force), matching today's preserve-as-is behaviour; default/absent schemes keep current output byte-identical and existing goldens stay green.
3. **Semantics — behavioural risk.** Forcing braces inserts `{ }` tokens that were not in the source — explicitly still a whitespace/brace-layout change under R5 (braces around a single statement do not change semantics) and called out in the changelog; reformatting the tool's own output stays a no-op (R6), since braces are only added when absent, never stripped.
4. **Encodings.** Force codes `0`/`1`/`3` as documented in docs/settings/index.md; values outside that set fall back to `0` (do not force).

# Acceptance criteria

- Golden fixture + test file per option under crates/core/tests/options/ at force codes `0`, `1` and `3`, plus an absent-option default case.
- Force-always (`3`) adds `{ … }` with the body indented one level around brace-less `if`/`for`/`while`/`do … while` bodies; force-if-multiline (`1`) braces only multiline bodies; do-not-force (`0`) preserves the input.
- Default-scheme output unchanged; whole suite green (`cargo test`).
- `docs/settings/common.md` marks flipped (❌ → ✅); README honoured-options table / behaviour notes and `docs/requirements.md` updated.
- New goldens idempotent: formatting the braced output again is a no-op.
