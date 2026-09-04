---
type: ChangeRequest
kind: feature
title: Honour the before-parentheses/braces/keywords spacing options
description: Apply the SPACE_BEFORE_* options that control the gap before parens, braces and clause keywords.
state: done
verified: { by: maintainer, at: 2026-09-03T23:00:11Z }
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

# Implementation plan

## Approach

Two sides, as in the binary-expression CR: configuration via the `OPTIONS` registry, then whitespace emission in the formatter, then the per-option golden suite, then docs. All changes are whitespace-only (R5), unmodelled shapes stay echoed verbatim (R4), and the options not listed here remain unimplemented and safely ignored (R7).

**config.rs — the registry.** `JavaStyle` is only ever constructed through `Default` (no literal sites exist to update), so add one `bool` field per option plus one `OptionDef` entry (all `Section::CodeStyleJava`, `OptionValue::Bool`, group `"Spaces"`, get/set closures). Parse and serialize are registry-driven (parse_codestyle L696-719 iterates `OPTIONS`), so no other config code changes and the GUI renders the new group automatically.

Defaults follow decision 2 / the IntelliJ table exactly: `true` for the clause-keyword paren options (if/while/for/try/catch/switch/synchronized) and for all brace options except the array ones, plus the four keyword options; `false` for `SPACE_BEFORE_METHOD_CALL_PARENTHESES`, `SPACE_BEFORE_METHOD_PARENTHESES`, `SPACE_BEFORE_ANOTATION_PARAMETER_LIST` (XML name spelled as in IntelliJ, typo included), `SPACE_BEFORE_ARRAY_INITIALIZER_LBRACE` and `SPACE_BEFORE_ANNOTATION_ARRAY_INITIALIZER_LBRACE`.

Default-output audit: today's canonical emission already equals each option's default at every site except one — `array_creation` / `flat_arr_creation` always print `new int[] {…}` (a space), whereas the option defaults `false`, so default output for that construct tightens to `new int[]{…}` (the IntelliJ built-in). No existing fixture or golden exercises array creation (verified), so the existing default suite stays green; the new fixtures pin the tightened form.

**formatter.rs — conditional gaps.** Add a tiny helper (`fn sp(&self, on: bool) -> &str` returning `" "` / `""`) and route each emission's single gap through it at the site where the token is printed. Each site must be changed in every path that renders it — the multi-line emitter, the keep-simple one-line emitter, and the flat emitter used for inline contexts and margin checks — so a toggle renders consistently and stays idempotent. Sites by family (current emitters in parens):

- *Before parentheses* (keyword/name → `(`): method calls (`flat_inv`, `inv_wrapped`, `fmt_chain` name→args joins; `new_expr` constructor-call args too — cross-check with IntelliJ), method/constructor declarations (`method_decl`, `constructor_decl` name→`formal_params` join), `if`/`while`/`switch`/`synchronized` clause headers (`if_stmt`/`if_one_line`, `while_stmt` incl. the do-while tail, `switch_stmt`/`switch_one_line`, `sync_stmt`), `for` headers (`for_stmt` rebuilds its header from source bytes + `normalise_ws`, so after normalising pin the gap between the leading `for` and `(`; `enhanced_for` builds its own header), `try`/`catch` (`try_stmt`/`try_one_line` resource-specification and catch-parameter joins), and annotation parameter lists (`annotation`/`flat_annotation`/`annotation_expanded` `@Name`→`(` joins — only where the formatter re-renders annotations such as `modifiers`; raw-echoed positions are untouched per R4).
- *Before braces*: type bodies (`with_brace` EndOfLine join for class/interface/enum/record; a NextLine brace sits at line start where the toggle is moot; plus the anonymous-class body join in `new_expr`), method/constructor bodies (`brace_before_body` EndOfLine join and the keep-simple push in `method_body`), statement bodies (`stmt_as_block_or_inline` lead space and the one-line candidate formats in `if_stmt`/`if_one_line`, `while_stmt`, `for_stmt`/`enhanced_for`, `do_while`; `else` bodies use `SPACE_BEFORE_ELSE_LBRACE`), `switch`/`try`/`catch`/`finally`/`synchronized` (`switch_stmt`/`switch_one_line`, `try_stmt`/`try_one_line`, `sync_stmt`), array initializers (`array_creation`/`flat_arr_creation` value join — `new int[] {`), and annotation array initializers (single unkeyed `element_value_array_initializer` directly after the annotation `(` — the `@SuppressWarnings( {…)` shape — in `annotation`/`flat_annotation`).
- *Before keywords* (`}` → keyword): `else` (if-chain alternative prefixes in `if_stmt`/`if_one_line`), `catch`/`finally` (clause prefixes in `try_stmt`/`try_one_line`), and the do-while trailing `while` (`do_while`, both paths). These combine with the paren/brace toggles to produce the `} else {`, `}else{`, `} while(x);` variants.

**Tests (AGENTS hard rules).** One test file per option: `crates/core/tests/options/<xml-option-snake>.rs` starting with `use super::common::*;`, doc header `//! <XML_OPTION> — …` + `//! Fixtures live under tests/java/<option>/.`, wired by `tests/options.rs` via `#[path]`. Fixtures under `tests/java/<option>/`, embedded with relative `include_str!`; input and golden share a stem; byte-exact `assert_eq!(format_with(INPUT, &style), GOLDEN)` (or `format(INPUT)` for the default style). No inline Java strings, no extra helpers, no `parse_codestyle` tests (AGENTS). Per option: a fixture containing the governed construct(s) in canonical form, a default-style golden (absent → default), and a toggled golden for the non-default value — covering on/off/absent (for a default-`true` option the default golden is the on case; for default-`false` it is the off case). Where one option governs several constructs (e.g. `SPACE_BEFORE_WHILE_*` covers `while` and the do-while tail; class lbrace covers class/interface/enum/record) the fixture includes each so every emission site is pinned.

**Docs.** Flipped at delivery: the 28 ❌ rows of the "Before parentheses / braces / keywords" table in `docs/settings/common.md`, the README honoured-options table + a formatting-behaviour bullet, a new requirement row in `docs/requirements.md`, and a `docs/dev/changelog.md` entry.

## Steps

- [x] src/config.rs: add the 28 `bool` fields to `JavaStyle` with the decision-2 defaults and the matching 28 `OptionDef`s (`Section::CodeStyleJava`, group `"Spaces"`, `get`/`set` closures) to `OPTIONS`; `cargo build` stays clean — parse and serialize pick them up through the registry, so a scheme setting any of them now round-trips. (AC: config mapping / parsing)
- [x] formatter.rs: make the before-parenthesis gaps conditional at every site listed in the approach (method-call, method-declaration, if/while/for/try/catch/switch/synchronized clause headers incl. the do-while tail, annotation parameter lists), covering the multi-line, keep-simple one-line and flat emitters; `for_stmt` pins the `for`↔`(` gap after `normalise_ws`. (AC2 paren examples: `f (x)`, `if(x)`, `@Anno (…)`)
- [x] formatter.rs: make the before-brace gaps conditional at every listed site (class/method/if/else/while/for/do/switch/try/catch/finally/synchronized bodies, array and annotation-array initializers), including the keep-simple one-line paths; `array_creation`/`flat_arr_creation` drop the space under the `false` default so default output becomes `new int[]{…}`. (AC2 brace examples: `new int[] {`, `} else {` variants)
- [x] formatter.rs: make the before-keyword gaps (`else`, `catch`, `finally`, do-while `while`) conditional in both the multi-line and one-line emitters. (AC2 keyword variants)
- [x] Regression checkpoint: run `cargo test` — the suite must stay green with no default-golden diffs (the array-creation default tightening has no existing fixture). (AC3)
- [x] Tests: add the 28 per-option test files under `crates/core/tests/options/` (one per listed XML option, snake-cased) and wire each into `tests/options.rs`; create the matching `tests/java/<option>/` fixtures with a canonical input, a default `*.out.java` golden and a toggled `*.out.java` golden per option file, asserted with `format` / `format_with` only. (AC1, AC2)
- [x] Idempotency and full suite: run `cargo test`; then re-format each new golden with its own style and confirm the output is byte-identical (no committed helper — AGENTS forbids adding `assert_idempotent`; verify once manually). (AC3)
- [x] If an IntelliJ installation is available, format snippets covering the ambiguous joins (constructor-call parens `new Foo (`, anonymous-class lbrace, do-while tail, array initializer after `=`) and align the mapping/goldens; record the outcome in the changelog (as the binary and switch CRs did). (AC2 fidelity)
- [x] Docs + final gate: flip the 28 ❌ marks to ✅ in the "Before parentheses / braces / keywords" table of `docs/settings/common.md`; add the 28 rows to the README honoured-options table plus a formatting-behaviour note; add the new requirement row (R16) to `docs/requirements.md`; append the entry to `docs/dev/changelog.md`; run `cargo test` green. (AC4)

Commit: not committed (worktree changes only — the repository is left for the owner to commit).
