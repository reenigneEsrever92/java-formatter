---
type: ChangeRequest
kind: feature
title: Keep simple classes and multi-expression statements on one line; lay out one-line block bodies
description: Implement the remaining keep-in-one-line options plus the block-body spacing/new-line options.
state: done
priority: medium
tags: [dev, formatter]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-05T20:15:00Z
---

# Problem

The remaining keep-in-one-line rows of docs/settings/common.md "Keep in one line" are ❌ — `KEEP_SIMPLE_CLASSES_IN_ONE_LINE` and `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE` — as are the two one-line-block presentation toggles of docs/settings/java.md "Miscellaneous spacing & blank lines", `SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT` and `NEW_LINE_WHEN_BODY_IS_PRESENTED`; all are valid IntelliJ options java-formatter neither parses nor applies, so schemes setting them are only partially honoured and the options are ignored (R7). Today simple class bodies always render multi-line and multi-expression statements never collapse, while `KEEP_SIMPLE_BLOCKS`/`METHODS`/`LAMBDAS_IN_ONE_LINE` already ship (R12 extended blocks to `try`/`catch`/`synchronized`), and one-line block bodies use a fixed `{ … }` layout with no padding or new-line control.

# Proposal

Parse `KEEP_SIMPLE_CLASSES_IN_ONE_LINE` and `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE` (JAVA `codeStyleSettings` block) and `SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT` and `NEW_LINE_WHEN_BODY_IS_PRESENTED` (`<JavaCodeStyleSettings>`) into `JavaStyle` via the OPTIONS registry in crates/core/src/config.rs — `OptionDef` entries with the IntelliJ built-in defaults (`false` for all four; absent → default) as `OptionValue::Bool`. Apply them in crates/core/src/formatter.rs: keep a simple class body on one line when every member is simple and the whole fits the margin (`KEEP_SIMPLE_CLASSES_IN_ONE_LINE`), keep multiple expressions of a statement on one line (`KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE`), pad a one-line non-empty block with spaces (`SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT`) and put a one-line body on a new line (`NEW_LINE_WHEN_BODY_IS_PRESENTED`), composing with the shipped keep-simple family.

Docs touched: `docs/settings/common.md` and `docs/settings/java.md` (❌ → ✅ for these rows), `README.md` (honoured-options table rows and formatting-behaviour notes), `docs/requirements.md` (a new requirement row) and `docs/dev/changelog.md` on completion.

# Decisions

1. **One family, one request.** Only the four listed options are added; other keep-in-one-line and spacing options still ❌ stay unimplemented and are ignored safely (R7).
2. **Defaults.** IntelliJ built-in defaults (`false`); absent → default, so default/absent schemes keep today's multi-line class layout and fixed one-line block layout byte-identical and existing goldens stay green.
3. **Semantics.** R5: collapsing a class body to one line or padding / re-placing a one-line block changes only whitespace, never tokens; unmodelled bodies stay verbatim (R4); new goldens pin R6 idempotency.
4. **Scope split.** The two common-block options complete the shipped keep-simple family (classes, multi-expression statements); the two Java-block options are presentation toggles for one-line block bodies (`{ … }` padding, new-line placement) that compose with it.

# Acceptance criteria

- Golden fixture + test file per option under crates/core/tests/options/ covering the option on and off, plus an absent-option default case.
- With `KEEP_SIMPLE_CLASSES_IN_ONE_LINE` a simple class body stays on one line; with `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE` multi-expression statements are not split; the block-body toggles pad and re-place one-line bodies as configured.
- Default-scheme output unchanged; whole suite green (`cargo test`).
- `docs/settings` marks flipped (❌ → ✅); README honoured-options table / behaviour notes and `docs/requirements.md` updated.
- New goldens idempotent: formatting the output again is a no-op.

# Implementation plan

## Approach

**Configuration (crates/core/src/config.rs).** `JavaStyle` (L105-150) gains four
bools, all defaulting to `false` in the `Default` impl (L152-182), so no
literal construction sites change and absent → default keeps every existing
scheme's semantics: `keep_simple_classes_in_one_line`,
`keep_multiple_expressions_in_one_line`, `spaces_inside_block_braces_when_body_is_present`
and `new_line_when_body_is_presented`. Four `OptionDef` entries are added to the
`OPTIONS` registry (drives parse, serialize and the GUI together): the two
common options in the "One-liners" group right after `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`
(L487-499) with `section: Section::CodeStyleJava` (JAVA `codeStyleSettings`
block — confirmed a `CommonCodeStyleSettings` field upstream), and the two
Java-specific presentation toggles with `section: Section::JavaCodeStyle`
(`<JavaCodeStyleSettings>` block, matching the docs/settings/java.md rows),
`group: "One-liners"`, each `default: OptionValue::Bool(false)`. All four are
read by the existing `OptionMap::get_bool` via `parse_codestyle` (L696-719) and
written by `serialize_codestyle` only when non-default.

**One-line body presentation.** Today the collapsed-body sites each compose a
fixed `{ stmt }` with one inner space: `one_line_body` (formatter.rs L1306-1335,
consumed by `if_one_line` L1339, `method_body` L884-903, `for_stmt` L1528,
`enhanced_for` L1557, `while_stmt` L1590, `do_while` L1618, `try_one_line`
L1656 and `sync_stmt` L1755) and `flat_block` (L2955-2966, used by the
keep-simple lambda collapse in `lambda` L2486-2523). Grounding in the upstream
`JavaCodeStyleSettings` javadoc: `SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT`
adds the single inner spaces of a non-empty one-line block **only when on**;
with it off (the built-in default) a one-line non-empty block is flush
(`fun() {int x = 1;}`) — so faithful semantics make absent/false **flush**
`{s}` and true padded `{ s }`. Centralise this in one helper that turns a
rendered inner statement text into the presented block (`{s}` / `{ s }`) plus,
when `NEW_LINE_WHEN_BODY_IS_PRESENTED` is set, places that one-line block on a
new line below the statement head at the head's indent (`if (c)` NL `{ s }`),
keeping the existing whole-statement `fits` gate. Apply it at the keep-simple
**decision** sites only (`one_line_body` consumers, the `lambda` collapse and
the new class collapse); leave `flat_block` and the switch one-line rendering
(L1924-1987) untouched so flat contexts (call-argument lambdas, one-line
switch values) keep their pinned `{ … }` layout and no default-scheme golden
changes (AC3). `NEW_LINE_WHEN_BODY_IS_PRESENTED`'s exact presentation is pinned
by a golden; if an IntelliJ installation is available to the implementer,
verify both toggles against it and adjust the goldens if they differ.

**Existing goldens stay green.** The padded one-line bodies currently shipped
live only in fixtures whose styles enable a keep-simple option and *not* the
new toggles: `keep_simple_blocks_in_one_line.rs` (`keep_simple()`), the try/sync
fixtures there, `brace_style.rs` (`end_of_line_brace_style_allows_one_line_simple_blocks`),
`keep_simple_methods_in_one_line.rs` (`keep_simple()`) and
`keep_simple_lambdas_in_one_line.rs` (`on_style()`). To keep those goldens
byte-identical under the faithful flush default, each such style helper also
sets `spaces_inside_block_braces_when_body_is_present = true` (they are
non-default schemes; only their *intent* — show the spaced collapsed body — is
preserved). Default-scheme goldens contain no keep-simple one-line bodies, so
nothing else moves; the root `codestyle.xml` sample (keep-simple on, toggles
absent) additionally gains `<option name="SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT"
value="true"/>` under `<JavaCodeStyleSettings>` so its demo output stays as
documented.

**KEEP_SIMPLE_CLASSES_IN_ONE_LINE (formatter.rs).** `class_decl` (L461-496),
`iface_decl` (L498-529) and `record_decl` (L600-633) build a header and attach
`class_body` (L705-732, returns `{}` when empty) via `with_brace` (L694-701).
Add a collapse ahead of the multi-line body: when the option is set and
`class_brace_style` is `EndOfLine` / `NextLineIfWrapped` (mirroring the
`braces_style_inline` idiom at L1296, per construct style), render each member
with `class_member` (L734-763) at the declaration's indent; if every member
renders without `\n` (comments/`is_extra` members reject — R4), join with a
single space and present the whole `class A { … }` when it fits the margin.
Bodies whose header already wraps, non-simple members and multi-member bodies
that do not fit fall through to today's layout. Enums are excluded (one-line
enum layout is the separate enum-layout CR's scope); anonymous-class bodies in
`new_expr` (L2256-2286) stay as today. A shared helper keeps the three
renderers in sync. Members collapse recursively (a simple method inside a
simple class uses the same one-line body presentation).

**KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE (formatter.rs).** The multi-expression
statement forms java-formatter renders are the classic `for` header's
init/update clause lists (`for_stmt` L1528-1555, which re-creates the header
verbatim via `normalise_ws`) and multi-declarator `local_var` (L1428-1481) /
`field_decl` (L905-960) declarations; none of these is ever split
clause-by-clause today (per-clause breaking only arrives with the separately
planned `FOR_STATEMENT_WRAP`). Implement the option as the guard that keeps
these lists on one line: read it at those render sites so the inline joins are
explicit, and let the keep-simple collapse of a statement whose head carries
multiple expressions compose unchanged. The per-option goldens pin the unsplit
layout with the option on, off and absent; the behaviour note and changelog
record that the option becomes load-bearing when `FOR_STATEMENT_WRAP` ships.
If an IntelliJ installation shows the option additionally relaxing which
multi-statement bodies may collapse, note the divergence in the changelog and
extend `one_line_body` accordingly in a follow-up.

## Steps

- [x] crates/core/src/config.rs: add the four bool fields to `JavaStyle` and
      their `false` defaults, then the four `OptionDef` entries (two
      `Section::CodeStyleJava` in the "One-liners" group after
      `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`; two `Section::JavaCodeStyle`); confirm
      `cargo build` and that the GUI/registry pick them up with no other change
      (AC: schemes now parse and serialize the four options).
- [x] formatter.rs: extract the one-line `{ … }` presentation into a shared
      helper honouring `spaces_inside_block_braces_when_body_is_present`
      (absent/false → flush `{s}`, true → `{ s }`) and
      `new_line_when_body_is_presented` (block placed on the next line at the
      head's indent); route `one_line_body`'s consumers, the keep-simple lambda
      collapse in `lambda` and the new class collapse through it; leave
      `flat_block` / switch one-line untouched (AC2, AC3).
- [x] formatter.rs: `KEEP_SIMPLE_CLASSES_IN_ONE_LINE` collapse in
      `class_decl` / `iface_decl` / `record_decl` via a shared helper
      (inline class brace style, every member newline-free, whole declaration
      fits the margin; enums and anonymous classes out of scope) (AC2).
- [x] formatter.rs: read `keep_multiple_expressions_in_one_line` at the
      multi-clause `for` header and multi-declarator `local_var` / `field_decl`
      render sites so their single-line joins are explicit and never split
      (AC2).
- [x] Keep the existing one-line goldens byte-identical: enable
      `spaces_inside_block_braces_when_body_is_present` in the style helpers of
      `tests/options/keep_simple_blocks_in_one_line.rs`, `brace_style.rs`
      (the end-of-line test), `keep_simple_methods_in_one_line.rs` and
      `keep_simple_lambdas_in_one_line.rs`; add the padding option to the
      `<JavaCodeStyleSettings>` block of the root `codestyle.xml` sample; run
      `cargo test` to confirm no golden changed (AC3).
- [x] New option test files `crates/core/tests/options/keep_simple_classes_in_one_line.rs`,
      `keep_multiple_expressions_in_one_line.rs`,
      `spaces_inside_block_braces_when_body_is_present.rs` and
      `new_line_when_body_is_presented.rs` (doc header + `use super::common::*;`
      per AGENTS.md), wired into `tests/options.rs`; fixtures under
      `tests/java/<option>/` as golden pairs (AC1).
- [x] Fixtures: `keep_simple_classes_in_one_line/` — simple class/interface/
      record collapse with the option on, off/absent unchanged multi-line,
      too-wide and non-simple-member bodies staying multi-line, and the
      NextLine class brace style not collapsing (AC1, AC2).
- [x] Fixtures: `keep_multiple_expressions_in_one_line/` — a classic `for`
      with multi-clause init/update and a multi-declarator field/local
      declaration stay on one line with the option on, off and absent, plus a
      keep-simple-blocks composition whose collapsed body keeps the
      multi-expression header inline (AC1, AC2).
- [x] Fixtures: `spaces_inside_block_braces_when_body_is_present/` — the same
      keep-simple input (if/else, loop, method, try/catch) formatted with the
      toggle absent (flush `{s}` golden, the new default) and on (padded
      `{ s }` golden), and
      `new_line_when_body_is_presented/` — the toggle on places the one-line
      body on its own line, absent keeps it after the head (AC1, AC2).
- [x] Verify each new `*.out.java` is idempotent (formatting the golden with
      its own style is a no-op) by running the formatter over each golden
      during development (AC5 — the suite itself stays pure golden pairs per
      AGENTS.md).
- [x] Run the whole workspace `cargo test`; confirm every existing golden —
      default-scheme and the keep-simple/brace-style files touched above —
      stays green (AC3).
- [x] Docs: flip `KEEP_SIMPLE_CLASSES_IN_ONE_LINE` and
      `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE` ❌ → ✅ in
      docs/settings/common.md "Keep in one line" and the two rows ❌ → ✅ in
      docs/settings/java.md "Miscellaneous spacing & blank lines"; add the four
      rows to the README honoured-options table plus formatting-behaviour notes
      (one-line bodies are flush by default and padded/re-placed by the two
      Java toggles; simple class bodies collapse when every member is simple;
      multi-expression statements are never split); add the requirement row
      (R16) to docs/requirements.md and extend its milestone paragraph; append
      the entry to docs/dev/changelog.md (recording whether an IntelliJ install
      was available to cross-check the goldens); re-run `cargo test` green
      (AC4, AC3).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
