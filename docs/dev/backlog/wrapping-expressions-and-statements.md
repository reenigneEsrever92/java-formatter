---
type: ChangeRequest
kind: feature
title: Wrap the remaining expressions, statements and initialisers per their *_WRAP options
description: Implement ternary/assert/for-header/array-initialiser wrapping and the sign-placement sub-options not yet shipped.
state: proposed
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

The expression/statement wrapping rows of docs/settings/common.md "Wrapping & braces / Expressions and statements" are all ❌ — `TERNARY_OPERATION_WRAP`, `ASSERT_STATEMENT_WRAP`, `FOR_STATEMENT_WRAP`, `ARRAY_INITIALIZER_WRAP` and `MODIFIER_LIST_WRAP` with their sign/keyword/paren placement bools, plus `WRAP_SEMICOLON_AFTER_CALL_CHAIN` in docs/settings/java.md "Miscellaneous spacing & blank lines" — valid IntelliJ options java-formatter neither parses nor applies, so schemes setting them are only partially honoured and the options are ignored (R7). Today ternary expressions, assert statements, for headers and array initialisers are laid out on one line only, while method-call chains, assignments, binary expressions and switch layout already wrap per their options (`METHOD_CALL_CHAIN_WRAP`, `ASSIGNMENT_WRAP`, `BINARY_OPERATION_WRAP`).

# Proposal

Parse `WRAP_FIRST_METHOD_IN_CALL_CHAIN`, `PARENTHESES_EXPRESSION_LPAREN_WRAP`, `PARENTHESES_EXPRESSION_RPAREN_WRAP`, `BINARY_OPERATION_SIGN_ON_NEXT_LINE`, `TERNARY_OPERATION_WRAP`, `TERNARY_OPERATION_SIGNS_ON_NEXT_LINE`, `PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE`, `ASSERT_STATEMENT_WRAP`, `ASSERT_STATEMENT_COLON_ON_NEXT_LINE`, `FOR_STATEMENT_WRAP`, `FOR_STATEMENT_LPAREN_ON_NEXT_LINE`, `FOR_STATEMENT_RPAREN_ON_NEXT_LINE`, `ARRAY_INITIALIZER_WRAP`, `ARRAY_INITIALIZER_LBRACE_ON_NEXT_LINE`, `ARRAY_INITIALIZER_RBRACE_ON_NEXT_LINE`, `MODIFIER_LIST_WRAP` and `WRAP_SEMICOLON_AFTER_CALL_CHAIN` into `JavaStyle` via the OPTIONS registry in crates/core/src/config.rs — `OptionDef` entries with the IntelliJ built-in defaults from the tables (all `0`/`false`; absent → default), `*_WRAP` entries reusing the existing `WrapStyle` mapping, bools as `OptionValue::Bool`. Apply them in crates/core/src/formatter.rs at the constructs they govern: wrap a ternary at `?`/`:`, an assert at its expression and `:`, a for header at its semicolons and an array initialiser at its elements; the sign/paren/colon bools (`BINARY_OPERATION_SIGN_ON_NEXT_LINE` complementing the shipped `BINARY_OPERATION_WRAP`, `PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE` complementing `ASSIGNMENT_WRAP`, `WRAP_FIRST_METHOD_IN_CALL_CHAIN` extending `METHOD_CALL_CHAIN_WRAP`, the `PARENTHESES_EXPRESSION_*_WRAP` and the `FOR_STATEMENT`/`ARRAY_INITIALIZER` paren/brace bools) steer placement only when the construct wraps; `MODIFIER_LIST_WRAP` and `WRAP_SEMICOLON_AFTER_CALL_CHAIN` add the declaration-prefix and chain-`;` breaks.

Docs touched: `docs/settings/common.md` and `docs/settings/java.md` (❌ → ✅ for these rows), `README.md` (honoured-options table rows and formatting-behaviour notes), `docs/requirements.md` (a new requirement row) and `docs/dev/changelog.md` on completion.

# Decisions

1. **One family, one request.** Only the listed options are added; `SWITCH_EXPRESSIONS_WRAP` belongs to the switch/case layout request, and other wrapping options still ❌ stay unimplemented and are ignored safely (R7).
2. **Defaults.** IntelliJ built-in defaults (`0`/`false`) per the tables; absent → default, so default/absent schemes keep today's single-line layout byte-identical and existing goldens stay green.
3. **Semantics.** R5: wrapping inserts whitespace at existing token boundaries and the sign-placement bools relocate an operator to the start of a continuation line, never reordering operands; unmodelled shapes stay verbatim (R4); new goldens pin R6 idempotency.
4. **Encodings.** The `*_WRAP` options share the wrap codes `0`/`1`/`2`/`5`; the LPAREN/RPAREN/sign/colon-on-next-line bools affect only constructs that actually wrap, and the sign-placement bools are sub-options of already-shipped wraps rather than new wrap toggles.

# Acceptance criteria

- Golden fixture + test file per option under crates/core/tests/options/ at the interesting values (wrap codes `0`/`1`/`2`/`5`, both bool states) plus an absent-option default case.
- Ternary, assert, for-header and array-initialiser fixtures wrap within the margin under wrap-if-long and always under wrap-always, with sign/paren/colon placement honoured on wrapped output.
- Default-scheme output unchanged; whole suite green (`cargo test`).
- `docs/settings` marks flipped (❌ → ✅); README honoured-options table / behaviour notes and `docs/requirements.md` updated.
- New goldens idempotent: formatting the wrapped output again is a no-op.
