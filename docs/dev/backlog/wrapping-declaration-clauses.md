---
type: ChangeRequest
kind: feature
title: Wrap resource lists, extends/implements and throws lists per their *_WRAP options
description: Implement RESOURCE_LIST_WRAP, EXTENDS_LIST_WRAP, THROWS_LIST_WRAP and related clause-layout sub-options.
state: proposed
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

The clause-layout rows of docs/settings/common.md "Wrapping & braces" are all ❌ — `RESOURCE_LIST_WRAP`, `EXTENDS_LIST_WRAP`, `THROWS_LIST_WRAP` with their keyword/paren placement bools and `PREFER_PARAMETERS_WRAP` — valid IntelliJ options java-formatter neither parses nor applies, so schemes setting them are only partially honoured and the options are ignored (R7). Today try-with-resources clauses, `extends`/`implements` lists and `throws` lists render on one line regardless of the margin — the README records they "are preserved." — while method parameters and call arguments already wrap per their options (`METHOD_PARAMETERS_WRAP`, `CALL_PARAMETERS_WRAP` with their `LPAREN`/`RPAREN`-on-next-line bools), so these clause lists are the remaining unwrapped list constructs.

# Proposal

Parse `PREFER_PARAMETERS_WRAP`, `RESOURCE_LIST_WRAP`, `RESOURCE_LIST_LPAREN_ON_NEXT_LINE`, `RESOURCE_LIST_RPAREN_ON_NEXT_LINE`, `EXTENDS_LIST_WRAP`, `EXTENDS_KEYWORD_WRAP`, `THROWS_LIST_WRAP` and `THROWS_KEYWORD_WRAP` into `JavaStyle` via the OPTIONS registry in crates/core/src/config.rs — `OptionDef` entries in the JAVA `codeStyleSettings` block with the IntelliJ built-in defaults from the tables (all `0`/`false`; absent → default), the `*_WRAP` entries reusing the existing `WrapStyle` mapping, the bools as `OptionValue::Bool`. Apply them in crates/core/src/formatter.rs at the constructs they govern: when a resource list, `extends`/`implements` list or `throws` list exceeds the margin (or per wrap-always), break it into one clause per continuation line; the `*_KEYWORD_WRAP`/`*_ON_NEXT_LINE` bools put the keyword or paren on its own line only when the list actually wraps; `PREFER_PARAMETERS_WRAP` favours the parameter list over other wrap points.

Docs touched: `docs/settings/common.md` (❌ → ✅ for these rows), `README.md` (honoured-options table rows + removal of the "…clauses are preserved" limitation), `docs/requirements.md` (a new requirement row) and `docs/dev/changelog.md` on completion.

# Decisions

1. **One family, one request.** Only the listed clause-layout options are added; other `*_WRAP` options still ❌ stay unimplemented and are ignored safely (R7).
2. **Defaults.** IntelliJ built-in defaults (`0`/`false`) per the tables; absent → default, so default/absent schemes keep today's single-line, preserved-clause output byte-identical and existing goldens stay green.
3. **Semantics.** R5: wrapping inserts only newlines and continuation indentation at clause boundaries, never reorders tokens; unmodelled clause shapes stay verbatim (R4); new goldens pin R6 idempotency.
4. **Encodings.** The `*_WRAP` options share the wrap codes `0`/`1`/`2`/`5` already mapped by `WrapStyle`; the keyword/paren-on-next-line bools affect only constructs that actually wrap.

# Acceptance criteria

- Golden fixture + test file per option under crates/core/tests/options/ at the interesting values (wrap codes `0`/`1`/`2`/`5`, both placement-bool states) plus an absent-option default case.
- Long resource / `extends`/`implements` / `throws` lists wrap within the margin under wrap-if-long and always under wrap-always, with `*_KEYWORD_WRAP`/`*_ON_NEXT_LINE` placement honoured on wrapped output.
- Default-scheme output unchanged; whole suite green (`cargo test`).
- `docs/settings/common.md` marks flipped (❌ → ✅); README honoured-options table / behaviour notes and `docs/requirements.md` updated.
- New goldens idempotent: formatting the wrapped output again is a no-op.
