---
type: ChangeRequest
kind: feature
title: Force braces on statement bodies per the *_BRACE_FORCE options
description: Implement IF_BRACE_FORCE, FOR_BRACE_FORCE, WHILE_BRACE_FORCE and DOWHILE_BRACE_FORCE.
state: done
verified: { by: maintainer, at: 2026-09-03T21:09:21Z }
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

# Implementation plan

## Approach

Two sides: configuration and rendering.

**Configuration (crates/core/src/config.rs).** Add a fourth code-mapping enum
beside `WrapStyle`/`BraceStyle` — `ForceStyle { DoNotForce,
ForceIfMultiline, ForceAlways }` — with `from_int` (`1` → `ForceIfMultiline`,
`3` → `ForceAlways`, everything else — including `0` and out-of-set values —
→ `DoNotForce`, per decision 4) and `to_int` (`0`/`1`/`3`), documented against
the force-brace codes in docs/settings/index.md. Extend `OptionValue` with a
`Force(ForceStyle)` variant, add `OptionMap::get_force` beside `get_wrap` /
`get_brace`, and add the matching decode arm in `parse_codestyle` and value arm
in `serialize_codestyle` (both already key off `def.default` / the value's
variant, mirroring the Wrap/Brace arms). `JavaStyle` gains four fields
(`if_brace_force`, `for_brace_force`, `while_brace_force`,
`dowhile_brace_force`) defaulting to `DoNotForce`; the struct is built only via
`Default`, so no other construction sites change and default/absent schemes
keep byte-identical output and serialization (decision 2). Register four
`OptionDef`s in the JAVA `codeStyleSettings` block (`Section::CodeStyleJava`,
default `OptionValue::Force(ForceStyle::DoNotForce)`, group `"Braces"`, placed
right after the three brace-style defs so the GUI lists them in the same
Braces group); the registry drives parsing, serialization and the GUI, so no
per-option hook is needed elsewhere. Because `crates/gui/src/main.rs` matches
`OptionValue` exhaustively in `option_row`, adding the variant requires a new
`Force` arm there (a labelled combo over the three variants via a `force_label`
helper like `brace_label`) or the workspace stops compiling.

**Rendering (crates/core/src/formatter.rs).** Brace-less single-statement
bodies are laid out today in `stmt_as_block_or_inline` (`if` consequence and
non-`if_statement` `else` alternative, `for` / `enhanced_for` / `while`
bodies) and in `do_while`'s inline single-statement branch; the
`one_line_body` / `if_one_line` collapse paths only fire for existing `block`
nodes, so a brace-less body never collapses and the force logic belongs in the
multi-line body path. Extend `stmt_as_block_or_inline` to take the governing
`ForceStyle`, each caller passing its own option field: `if_stmt` consequence
and `else` alternative → `if_brace_force`, `for_stmt` and `enhanced_for`
bodies → `for_brace_force`, `while_stmt` → `while_brace_force`, and
`do_while`'s single-statement branch → `dowhile_brace_force`. A brace-less
`else if` chain needs no special handling: each nested `if_statement`
alternative recurses through `if_stmt`, so every consequence is covered.
For a non-block body: `DoNotForce` keeps today's output byte-for-byte;
`ForceIfMultiline` renders the body first and adds braces only when that
rendered statement text contains a newline (the body already spans multiple
lines); `ForceAlways` always adds them. The braced text is emitted with
exactly the bytes `block()` produces for a single-statement block — an
opening `{` and newline, the statement at `indent + 1`, a newline, and the
closing `}` at `indent` — so a forced
body and a source that was already braced converge on identical canonical
output. Braces are only ever added, never stripped (decision 3), so
reformatting the braced output is a no-op (R6) and the insertion is
whitespace-only, keeping semantic equivalence (R5). Interplay with
`KEEP_SIMPLE_BLOCKS_IN_ONE_LINE` (which collapses pre-existing simple blocks)
is out of scope: forced bodies render in the multi-line block form. If an
IntelliJ installation is available to the implementer, cross-check the
goldens — especially a brace-less `else` body and an enhanced-`for` body — and
adjust if they differ (this plan treats both the classic and the enhanced
`for` as `for` bodies governed by `FOR_BRACE_FORCE`, since tree-sitter splits
them into `for_statement` / `enhanced_for_statement`); otherwise record the
convention in the changelog.

**Tests.** The .agents/AGENTS.md hard conventions apply: golden pairs only
(`assert_eq!(format_with(INPUT, &style), GOLDEN)`), one test file per option
under `crates/core/tests/options/<option>.rs` wired via `#[path]` in
tests/options.rs, fixtures under `tests/java/<option>/` referenced with
relative `include_str!`, no new helpers in tests/common/mod.rs, no
`parse_codestyle` tests. Each of the four files styles only its own field via
`style(|s| s.<field> = ForceStyle::…)` and covers codes `0`, `1`, `3`, the
absent-option default (`format(INPUT)` with the default style), and an
already-braced idempotency pair.

## Steps

- [x] src/config.rs: add `ForceStyle` (with doc table and `from_int`/
      `to_int`), `OptionValue::Force`, `OptionMap::get_force`; add the four
      `JavaStyle` fields (`if_brace_force`, `for_brace_force`,
      `while_brace_force`, `dowhile_brace_force`) and their `Default` values;
      register the four `OptionDef`s (`IF_BRACE_FORCE`, `FOR_BRACE_FORCE`,
      `WHILE_BRACE_FORCE`, `DOWHILE_BRACE_FORCE`) in the Braces group; add the
      `parse_codestyle` and `serialize_codestyle` arms. Verify the workspace
      compiles and default-style serialization is unchanged (decisions 2 and 4).
- [x] crates/gui/src/main.rs: import `ForceStyle`, add the `OptionValue::Force`
      combo arm with a `force_label` helper (like `brace_label`), and update
      the module doc's “wrap/brace → labeled combo” phrase; run `cargo check`
      so the new variant compiles across the workspace.
- [x] src/formatter.rs: extend `stmt_as_block_or_inline` with the governing
      `ForceStyle` and update its call sites (`if_stmt` consequence and
      non-`if_statement` alternative → `if_brace_force`; `for_stmt` and
      `enhanced_for` bodies → `for_brace_force`; `while_stmt` →
      `while_brace_force`); mirror the behaviour in `do_while`'s inline
      single-statement branch (`dowhile_brace_force`). `DoNotForce` output
      stays byte-identical; forced bodies match `block()`'s bytes. Run
      `cargo test` and confirm no existing golden changed (AC3, decision 2).
- [x] Fixtures + test files per option at codes 0/1/3 and absent default: for
      each of `if_brace_force`, `for_brace_force` (covering both classic and
      enhanced `for` bodies), `while_brace_force` and `dowhile_brace_force`,
      add fixtures under `tests/java/<option>/` — force-always (`{ … }` with
      the statement indented one level around brace-less bodies),
      force-if-multiline (a single-line body stays unbraced, a multiline body
      is braced), do-not-force and absent-option default (input preserved
      byte-for-byte) — each with a `*.out.java` golden, plus one
      `tests/options/<option>.rs` file (doc header + `use super::common::*;`)
      wired alphabetically into `tests/options.rs` (AC1, AC2).
- [x] Idempotency goldens: per option add a golden pair whose input is the
      already-braced force-always output; assert formatting it again is
      byte-identical, proving reformatting the braced output is a no-op (AC5,
      decision 3). Run the full `cargo test` and confirm the whole suite is
      green (AC3).
- [x] If IntelliJ is available, format the same snippets there and align the
      goldens if they differ (brace-less `else` bodies, enhanced-`for`
      bodies); record the outcome in the changelog entry. (No IntelliJ
      installation was available; the conventions — `else` governed by
      `IF_BRACE_FORCE`, classic and enhanced `for` by `FOR_BRACE_FORCE` — were
      recorded in the changelog entry instead.)
- [x] Docs + final validation: flip the four `docs/settings/common.md`
      “Force braces” rows ❌ → ✅; extend the `docs/settings/index.md` Caveats
      sentence so it also covers force-code decoding (out-of-set values fall
      back to do-not-force); add the four rows to the README honoured-options
      table plus a force-code note and a formatting-behaviour bullet, and
      update the README GUI sentence and `crates/gui` doc phrase (“wrap/brace/
      force → labeled combo”); add requirement R17 to `docs/requirements.md`
      (R16 was taken by the blank-line-policy request, so the next free number
      is used) and extend its Milestones paragraph; append the entry to
      `docs/dev/changelog.md` on completion; run `cargo test` once more and
      confirm everything is green.

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
