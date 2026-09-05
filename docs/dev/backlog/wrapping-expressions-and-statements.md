---
type: ChangeRequest
kind: feature
title: Wrap the remaining expressions, statements and initialisers per their *_WRAP options
description: Implement ternary/assert/for-header/array-initialiser wrapping and the sign-placement sub-options not yet shipped.
state: done
priority: medium
tags: [dev, formatter]
owner: maintainer
verified: { by: java-formatter-agent, at: 2026-09-05T12:00:00Z }
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

# Implementation plan

## Approach

Two sides, as in the binary-expression-wrapping CR: configuration (the `OPTIONS`
registry in src/config.rs) and rendering (src/formatter.rs per construct), then
goldens and docs.

**Configuration (crates/core/src/config.rs).** `JavaStyle` gains 17 fields —
four `WrapStyle` (`ternary_operation_wrap`, `assert_statement_wrap`,
`for_statement_wrap`, `array_initializer_wrap`) and thirteen `bool`
(`wrap_first_method_in_call_chain`, `parentheses_expression_lparen_wrap`,
`parentheses_expression_rparen_wrap`, `binary_operation_sign_on_next_line`,
`ternary_operation_signs_on_next_line`, `place_assignment_sign_on_next_line`,
`assert_statement_colon_on_next_line`, `for_statement_lparen_on_next_line`,
`for_statement_rparen_on_next_line`, `array_initializer_lbrace_on_next_line`,
`array_initializer_rbrace_on_next_line`, `modifier_list_wrap`,
`wrap_semicolon_after_call_chain`) — all defaulting to `DoNotWrap` / `false`
(the IntelliJ built-in defaults per the tables; absent → default). `JavaStyle`
is constructed only via `Default`, so no literal-site changes are needed.
Sixteen `OptionDef` entries are added to the `OPTIONS` registry in
`Section::CodeStyleJava` (the `<codeStyleSettings language="JAVA">` block,
where common.md documents them) and `WRAP_SEMICOLON_AFTER_CALL_CHAIN` in
`Section::JavaCodeStyle` (the `<JavaCodeStyleSettings>` block, where java.md
lists it). The existing `OptionMap::get_wrap` / `get_bool` helpers and the
registry-driven `parse_codestyle` / `serialize_codestyle` pick the entries up
automatically, and the GUI renders the new options from the same registry, so
no GUI change is needed. Per the AGENTS.md hard rules there are no
`parse_codestyle` tests; each option's default/absent state is pinned by the
absent-option golden in its test file.

**Rendering (crates/core/src/formatter.rs).** Each family follows the shipped
flatten-then-wrap pattern (`binary` for `BINARY_OPERATION_WRAP`): render the
flat form and return it when the wrap style is `DoNotWrap` or (for
`WrapIfLong`) the line fits; otherwise break the construct at its natural token
boundaries at the continuation indent — inserting only whitespace, never
reordering tokens, so R5 holds by construction and unmodelled shapes stay
verbatim (R4). The sign/paren/colon/brace booleans steer placement only when
the construct actually wraps (Decision 4). The default/absent state keeps every
existing layout byte-identical **except** the wrapped `binary_operation_wrap`
goldens, which are updated because `BINARY_OPERATION_SIGN_ON_NEXT_LINE` is
implemented with the polarity common.md documents: `false` (the default) puts
the operator at the end of the preceding line, `true` at the start of the
continuation line (the layout the binary CR shipped unverified). All other
families have no fixture coverage today — no fixture under `tests/java/`
contains a ternary, an assert, a long array initialiser or a long for header —
so their default-state behaviour may change without breaking existing goldens.
The `flat` inline arms (`binary`, `ternary`, `array_initializer`, …) stay
unchanged: flat contexts cannot contain newlines, and the placement booleans
affect only wrapped output.

Per construct:

- `binary` (L2377): the spine walk gains the sign-placement bool; each link is
  emitted either operator-first (`true`) or operator-end (`false`).
- `ternary` (L2453): becomes option-driven — `DoNotWrap` keeps the flat form
  even when long; `WrapIfLong` wraps only when it does not fit; `WrapAlways`
  always wraps; `ChopDownIfLong` additionally recurses into a consequence/
  alternative that is itself a ternary when its line overflows (mirroring
  `binary_operand`). `TERNARY_OPERATION_SIGNS_ON_NEXT_LINE=false` (default)
  keeps `?` / `:` at the end of the preceding line, `true` starts the
  continuation lines (today's always-on layout).
- `assert_stmt` (L1793): wrap at the expression and after the `:` per
  `ASSERT_STATEMENT_WRAP`; `ASSERT_STATEMENT_COLON_ON_NEXT_LINE` moves the `:`
  to the start of the next line when wrapped.
- `for_stmt` (L1528) / `enhanced_for` (L1557): when `FOR_STATEMENT_WRAP` is
  active, the classic header is re-rendered from its `init` / `condition` /
  `update` fields and broken at the semicolons, and the enhanced header breaks
  at the `:`; the verbatim raw-header path (normalise_ws of the source slice)
  stays for `DoNotWrap`, keeping that output byte-identical.
  `FOR_STATEMENT_LPAREN/RPAREN_ON_NEXT_LINE` place the parens on their own
  lines when the header wraps.
- `array_init` (L2559): option-driven — `DoNotWrap` keeps the flat form even
  when long; the wrapped layout honours
  `ARRAY_INITIALIZER_LBRACE/RBRACE_ON_NEXT_LINE` for the braces.
- Modifier list: when `MODIFIER_LIST_WRAP` is set, the declaration renderers
  that call `modifiers()` (class/interface/enum/record, method, constructor,
  compact constructor, field) break after the modifier/annotation list instead
  of keeping it on the header line.
- Chain extensions: `fmt_chain` (L2223) puts the first link on a continuation
  line too when `WRAP_FIRST_METHOD_IN_CALL_CHAIN` is set; `stmt`'s
  `expression_statement` arm appends `\n<indent>;` when the statement's chain
  wrapped and `WRAP_SEMICOLON_AFTER_CALL_CHAIN` is set.
- `assign_expr` (L2350): `PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE` moves the
  operator to the start of the continuation line; the current shipped layout
  (operator on the header line) is the faithful `false` state, so no existing
  assignment golden changes.
- `expr`'s `parenthesized_expression` arm (L2049): when the inner expression
  wraps and `PARENTHESES_EXPRESSION_LPAREN/RPAREN_WRAP` are set, the parens go
  on their own lines.

**Tests.** Follow the AGENTS.md hard rules: one test file per option at
`crates/core/tests/options/<XML_OPTION>.rs` (doc header
`//! <XML_OPTION> — …` plus `//! Fixtures live under tests/java/<option>/.`,
starting `use super::common::*;`), golden pairs only via `format_with` /
`style`, fixtures under `tests/java/<option>/` referenced by
`include_str!("../java/<option>/<scenario>.java")`, wired alphabetically in
`tests/options.rs`. Each file covers the interesting values (wrap codes
`0`/`1`/`2`/`5`; both bool states) plus an absent-option default case (a
fixture asserted unchanged under a tight-margin style that sets only the other
options), and idempotency (AC5) is asserted by re-formatting the golden output
and comparing it to the golden. The existing `binary_operation_wrap` goldens
are updated to the false-state operator-end layout in the sign-placement step.

**Docs.** On completion: flip the sixteen ❌ rows in `docs/settings/common.md`
"Expressions and statements" and the `WRAP_SEMICOLON_AFTER_CALL_CHAIN` row in
`docs/settings/java.md` to ✅; add the seventeen rows to the README
honoured-options table and extend the formatting-behaviour notes; add a new
requirement row (R16) to `docs/requirements.md`; append an entry to
`docs/dev/changelog.md`. `docs/settings/index.md` needs no change (the wrap
codes are already documented there).

## Steps

- [x] config.rs: add the 17 `JavaStyle` fields (4 `WrapStyle` + 13 `bool`) with
      `DoNotWrap` / `false` defaults, and the 17 `OPTIONS` entries (16 in
      `Section::CodeStyleJava`, `WRAP_SEMICOLON_AFTER_CALL_CHAIN` in
      `Section::JavaCodeStyle`) with the table defaults (AC: config mapping).
- [x] `binary`: implement `BINARY_OPERATION_SIGN_ON_NEXT_LINE` (false = operator
      ends the line, true = operator starts the continuation line); update the
      existing wrapped `binary_operation_wrap` goldens (`long_sum`,
      `always_wrap`, `chop_down` `.out.java`) to the false-state layout; run
      `cargo test` and confirm only those goldens changed (AC3).
- [x] `ternary`: option-driven wrap per `TERNARY_OPERATION_WRAP` (0 flat,
      1/2/5 as documented, chop-down recursing into nested ternary operands)
      plus `TERNARY_OPERATION_SIGNS_ON_NEXT_LINE` placement (AC2).
- [x] `assert_stmt`: wrap per `ASSERT_STATEMENT_WRAP` (at the expression and
      after the `:`) plus `ASSERT_STATEMENT_COLON_ON_NEXT_LINE` placement (AC2).
- [x] `for_stmt` / `enhanced_for`: header wrap per `FOR_STATEMENT_WRAP` (classic
      header re-rendered from init/condition/update and broken at semicolons;
      enhanced header broken at `:`) plus
      `FOR_STATEMENT_LPAREN/RPAREN_ON_NEXT_LINE` placement; keep the verbatim
      header path for `DoNotWrap` (AC2).
- [x] `array_init`: option-driven wrap per `ARRAY_INITIALIZER_WRAP` plus
      `ARRAY_INITIALIZER_LBRACE/RBRACE_ON_NEXT_LINE` placement (AC2).
- [x] Apply the remaining placement options: `MODIFIER_LIST_WRAP` at the
      `modifiers()` call sites, `WRAP_FIRST_METHOD_IN_CALL_CHAIN` in `fmt_chain`,
      `WRAP_SEMICOLON_AFTER_CALL_CHAIN` in the `expression_statement` arm,
      `PARENTHESES_EXPRESSION_LPAREN/RPAREN_WRAP` in the parenthesized-expression
      arm, `PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE` in `assign_expr` (AC2).
- [x] Add the 17 option test files `crates/core/tests/options/<XML_OPTION>.rs`
      with fixtures under `tests/java/<option>/` (wrap codes 0/1/2/5 and both
      bool states, plus an absent-option default case per file), wired
      alphabetically in `tests/options.rs` (AC1, AC2).
- [x] Assert idempotency of each new wrapped golden: formatting the `*.out.java`
      again with the same style is a no-op (AC5).
- [x] If an IntelliJ installation is available, format representative
      ternary/assert/for/array/binary snippets there and align the placement
      goldens; record the outcome in the changelog. (No IntelliJ installation
      available; the goldens follow the request's decisions and the existing
      call-parameter / binary wrap conventions — recorded in the changelog.)
- [x] Docs + full suite: flip the ❌ rows to ✅ in `docs/settings/common.md` and
      `docs/settings/java.md`; add the 17 rows to the README honoured-options
      table and extend the formatting-behaviour notes; add the R27 row to
      `docs/requirements.md`; append the changelog entry; run `cargo test` and
      confirm the whole suite is green with default-scheme output unchanged
      (AC3, AC4).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
