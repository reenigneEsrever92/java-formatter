---
type: ChangeRequest
kind: feature
title: Honour clause-keyword and brace-less control-statement layout options
description: Implement else/while/catch/finally on-new-line, special else-if, lambda brace style and brace-less one-lining.
state: done
verified: { by: maintainer, at: 2026-09-04T21:21:22Z }
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

`ELSE_ON_NEW_LINE`, `WHILE_ON_NEW_LINE`, `CATCH_ON_NEW_LINE`,
`FINALLY_ON_NEW_LINE`, `SPECIAL_ELSE_IF_TREATMENT`, `LAMBDA_BRACE_STYLE` and
`KEEP_CONTROL_STATEMENT_IN_ONE_LINE` are valid IntelliJ options marked ❌ in
docs/settings/common.md ("Braces & indentation" and "General & comments") and
safely ignored per R7, so a scheme that sets them is only partially honoured
and output diverges from IntelliJ. Brace styles (`CLASS_BRACE_STYLE` /
`METHOD_BRACE_STYLE` / `BRACE_STYLE`) and the keep-simple-* one-liners already
ship, but these clause-keyword and placement refinements do not: `Fmt::if_stmt`
always renders the alternative as ` else …` on the same line as the closing
brace, `try_stmt` / `while_stmt` / `do_while` keep their clause keywords
inline, lambda bodies go through the generic block path, and brace-less bodies
are always moved to their own line by `stmt_as_block_or_inline`.

# Proposal

Parse each listed option into a `JavaStyle` field via the `OPTIONS` registry in
crates/core/src/config.rs (`Section::CodeStyleJava`, IntelliJ built-in defaults
from the tables: the four `*_ON_NEW_LINE` flags `false`,
`SPECIAL_ELSE_IF_TREATMENT` `true`, `KEEP_CONTROL_STATEMENT_IN_ONE_LINE`
`true`, `LAMBDA_BRACE_STYLE` `1` / end of line); absent-from-scheme options
keep the default. Apply them in crates/core/src/formatter.rs at the constructs
they govern: `if_stmt` (else / else-if placement, `SPECIAL_ELSE_IF_TREATMENT`,
brace-less one-lining), `while_stmt` / `do_while` and `try_stmt` (`while` /
`catch` / `finally` on a new line), and `lambda` (`LAMBDA_BRACE_STYLE`, reusing
the existing `BraceStyle` mapping and the brace codes documented in
docs/settings/index.md).

Docs touched: on delivery the implementation updates the docs/settings support
marks (❌ → ✅ for these rows), the README honoured-options table /
formatting-behaviour notes, docs/requirements.md (a new requirement row), and
docs/dev/changelog.md.

# Decisions

- **One family, one request.** Only the seven listed options are added here;
  `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS` belongs to the indentation request and
  the force-braces rows to their own request; the other
  unimplemented rows stay out and are safely ignored (R7).
- **Defaults.** The registry records the IntelliJ built-ins from the tables;
  today's output already matches them for every option here except
  `KEEP_CONTROL_STATEMENT_IN_ONE_LINE` (brace-less bodies are currently forced
  onto their own line), so default and absent-from-scheme styles keep current
  byte-identical output and existing goldens stay green; any fixture encoding
  that divergence is updated deliberately with this change.
- **Semantics.** Whitespace/layout only (R5); unmodelled constructs are echoed
  verbatim (R4); formatting formatted output is a no-op (R6).
- **Encodings.** The six flags are plain bools; `LAMBDA_BRACE_STYLE` reuses the
  existing `OptionValue::Brace` mapping (brace codes) — no new registry types.

# Acceptance criteria

- A dedicated golden fixture + test file per option following the pattern in
  crates/core/tests/options/, each option tested at its interesting values plus
  an absent-option default check.
- `ELSE_ON_NEW_LINE` / `WHILE_ON_NEW_LINE` / `CATCH_ON_NEW_LINE` /
  `FINALLY_ON_NEW_LINE` move the keyword to its own line;
  `SPECIAL_ELSE_IF_TREATMENT` = `false` nests `else { if … }` instead of the
  fused `else if`.
- `KEEP_CONTROL_STATEMENT_IN_ONE_LINE` keeps a brace-less `if (…) …;` on one
  line (and breaks it when disabled); `LAMBDA_BRACE_STYLE` places lambda braces
  per its brace code, including a scheme that sets it differently from
  `BRACE_STYLE`.
- Default/absent schemes behave as today; `cargo test` stays green and the new
  goldens are idempotent (R6).
- docs/settings marks flipped to ✅; README and docs/requirements.md updated.

# Implementation plan

## Approach

Two sides: configuration and rendering.

**Configuration** (crates/core/src/config.rs). Add seven fields to
`JavaStyle` (L105-150) — `else_on_new_line`, `while_on_new_line`,
`catch_on_new_line`, `finally_on_new_line`, `special_else_if_treatment`,
`keep_control_statement_in_one_line: bool` and `lambda_brace_style:
BraceStyle` — with matching entries in `impl Default` (L152-182) set to the
IntelliJ built-ins from the docs/settings/common.md tables: the four
`*_ON_NEW_LINE` flags `false`, `SPECIAL_ELSE_IF_TREATMENT` `true`,
`KEEP_CONTROL_STATEMENT_IN_ONE_LINE` `true`, `LAMBDA_BRACE_STYLE`
`BraceStyle::EndOfLine` (brace code 1). Register one `OptionDef` per option
in the `OPTIONS` registry (L232-567), all `Section::CodeStyleJava` (the JAVA
`codeStyleSettings` block): the six flags as `OptionValue::Bool` grouped
under `"Braces"` next to the existing brace rows (L300-339), and
`KEEP_CONTROL_STATEMENT_IN_ONE_LINE` under `"One-liners"` next to the
`KEEP_SIMPLE_*` rows (L460-499). No new registry types, no GUI changes: the
gui, `parse_codestyle` and `serialize_codestyle` are registry-driven, so the
new rows appear everywhere automatically and serialize only when they differ
from default (`KEEP_CONTROL_STATEMENT_IN_ONE_LINE`, default `true`, is
omitted at default — matching IntelliJ's export convention). `JavaStyle` is
constructed only via `Default`, so new fields need no literal-site changes.
Per AGENTS.md there are no `parse_codestyle` unit tests; option files drive
the fields through `style(...)` directly.

**Rendering** (crates/core/src/formatter.rs). Four groups of edits:

1. *`if_stmt` clause layout* (L1483-1517). The consequence is rendered by
   `stmt_as_block_or_inline` and ends with the closing `}` at `ind(indent)`
   (block bodies) or with the body statement on its own line (brace-less
   bodies). The alternative prefix ` else ` is replaced by a `\n` +
   `ind(indent)` + `else` when `else_on_new_line` is set. When the
   alternative is a fused `if_statement` and `special_else_if_treatment` is
   `false`, synthesise a brace block around the recursive `if_stmt(alt)`:
   ` else {\n` + `ind(indent+1)` + nested `if` (rendered at `indent+1`) +
   `\n` + `ind(indent)` + `}` — this is the AC2 "else { if … }" nesting; the
   braces group an already-single `if`, so semantics are unchanged (R5) and
   the braces survive re-parsing, keeping R6. Gate the existing one-line
   collapse in `if_one_line`/`if_stmt` (L1339-1358, L1485-1491): suppress it
   whenever the statement has an alternative and `else_on_new_line` is set,
   or the alternative is an `if_statement` chain and
   `special_else_if_treatment` is `false` — the collapse candidate would
   contradict the flag.
2. *Clause keywords in `try_stmt` / `do_while`*. `try_stmt` (L1696-1753)
   appends ` catch ({param}) {body}` / ` finally {body}`; when the
   corresponding flag is set, prefix each clause with `\n` + `ind(indent)`
   instead of a space (the try body's `}` already sits at `ind(indent)`).
   `do_while` (L1618-1651) appends ` while {cond};`; with `while_on_new_line`
   emit the `while` on a fresh line at `ind(indent)`. Gate `try_one_line`
   (L1656-1694) and the do-while one-line collapse on the flags (a one-line
   `do { … } while (…);` / `try { … } catch …` would contradict them). Note:
   the brace-less do-body arm (L1640-1642) already ends its text with
   `ind(indent)`; keep that tail intact and pin the exact keyword placement
   with the do-while golden.
3. *`KEEP_CONTROL_STATEMENT_IN_ONE_LINE` (brace-less bodies)*. A "keep when
   formatting" option: enabled (the default) preserves the source's
   line relationship at each brace-less control-body boundary, and disabled
   forces today's layout (body on its own line). Concretely: at each site
   that routes a non-`block` body through `stmt_as_block_or_inline`
   (if/else L1500-1511, `for_stmt` L1550-1554, `enhanced_for` L1583-1587,
   `while_stmt` L1610-1613) and at `do_while`'s own brace-less arm
   (L1634-1643), when the flag is set and the source has **no newline**
   between the construct's last header token and the body node (scan
   `self.src[header_end..body_start]`, the same technique as
   `has_blank_line_between` L1284-1292), render `header stmt` inline;
   otherwise keep/force the own-line layout. No margin (`fits`) check — the
   option preserves author line breaks rather than joining by length, which
   is exactly why default/absent schemes (flag `true`) keep today's output
   byte-identical: today's formatted output always has brace-less bodies on
   their own lines, and no existing fixture carries a same-line brace-less
   body (verified by grepping tests/java/), so no existing golden changes.
   The only behaviour change is that a hand-written same-line `if (x)
   foo();` input is no longer split under the default — the deliberate
   divergence the request's Decisions call out.
4. *`LAMBDA_BRACE_STYLE` in `lambda`* (L2486-2523). A block-bodied lambda is
   rendered `format!("{} ->", params)` + `brace_before_body(indent,
   lambda_brace_style, &block_str)` (helper at L964-971, same machinery
   `method_body` uses): `EndOfLine`/`NextLineIfWrapped` keep `-> {`, the
   `NextLine` family put the `{` on its own line at `ind(indent)`. Reuse the
   shipped mapping exactly — the `NextLine`/`NextLineShifted`/`NextLineShifted2`
   arms are already identical in `brace_before_body`/`with_brace`, and
   `NextLineIfWrapped` stays same-line, consistent with the shipped class /
   method brace handling. Gate the existing `keep_simple_lambdas_in_one_line`
   collapse (L2511-2514) on an `EndOfLine`/`NextLineIfWrapped` lambda style,
   mirroring the `method_body` gate (L885-889). `flat_lambda` / `flat_block`
   stay as-is: they only produce candidate strings for contexts that already
   fit on one line.

The default values for the six layout flags reproduce today's output, so the
whole change is whitespace/layout only (R5), unmodelled constructs stay
verbatim (R4), and the interesting-value goldens pin the composed rules above
(and should be checked against a real IntelliJ installation when one is
available, adjusting the goldens if IntelliJ disagrees — as the
binary-expression-wrapping request did for operator placement).

**Tests.** Per AGENTS.md hard rules: one test file per option
(`crates/core/tests/options/<XML_OPTION>.rs`) wired by `tests/options.rs`,
fixtures under `tests/java/<option>/`, every test a byte-exact golden pair
(`format_with(INPUT, &style)` vs `*.out.java`, or `format(INPUT)` for the
default-style absent check), doc header `//! <XML_OPTION> — …` per file, no
inline Java strings, no `parse_codestyle` tests, no extra helpers. New-golden
idempotency (R6) is verified during development by reformatting each
`*.out.java` under the same style and asserting it is unchanged — no
`assert_idempotent` test is added.

**Docs.** `docs/settings/common.md` rows: `KEEP_CONTROL_STATEMENT_IN_ONE_LINE`
("General & comments", L54), `LAMBDA_BRACE_STYLE`, `ELSE_ON_NEW_LINE`,
`WHILE_ON_NEW_LINE`, `CATCH_ON_NEW_LINE`, `FINALLY_ON_NEW_LINE` and
`SPECIAL_ELSE_IF_TREATMENT` ("Braces & indentation", L90-96) flip ❌ → ✅
(adding a short qualifier where the applied semantics are narrower, matching
how existing ✅ rows annotate). `README.md` honoured-options table gains the
seven rows plus a formatting-behaviour note. `docs/requirements.md` gains a
new requirement row (R16). `docs/dev/changelog.md` gets an entry on
completion; work is left uncommitted for the owner (AGENTS.md).

## Steps

- [x] crates/core/src/config.rs: add the seven `JavaStyle` fields + defaults
      and the seven `OptionDef` registry entries (`Section::CodeStyleJava`;
      six bools, `LAMBDA_BRACE_STYLE` as `OptionValue::Brace`), defaults as
      in the Approach; `cargo build` green (foundation; AC4 absent-option
      defaults).
- [x] formatter.rs `if_stmt`: honour `ELSE_ON_NEW_LINE` (alternative keyword
      on a fresh line at `ind(indent)`) and `SPECIAL_ELSE_IF_TREATMENT` =
      `false` (synthesise `else {\n…if…\n}` around a fused `if_statement`
      alternative); gate the `if_one_line` collapse accordingly (AC2).
- [x] formatter.rs `try_stmt` and `do_while`: honour `CATCH_ON_NEW_LINE` /
      `FINALLY_ON_NEW_LINE` (each clause on a fresh line at `ind(indent)`)
      and `WHILE_ON_NEW_LINE` (trailing `while (…);` on a fresh line); gate
      `try_one_line` and the do-while one-line collapse on the flags (AC2).
- [x] formatter.rs brace-less sites (`stmt_as_block_or_inline` callers in
      `if_stmt` / `for_stmt` / `enhanced_for` / `while_stmt` and `do_while`):
      honour `KEEP_CONTROL_STATEMENT_IN_ONE_LINE` source-driven — inline a
      brace-less body only when the flag is set **and** the source has no
      newline between the header and the body; disabled keeps today's
      own-line layout (AC3 first half, AC4).
- [x] formatter.rs `lambda`: render block bodies through `brace_before_body`
      with `lambda_brace_style` and gate the `keep_simple_lambdas_in_one_line`
      collapse on an inline-compatible lambda brace style (AC3 second half).
- [x] Fixtures + tests: for each of the seven options create
      `tests/java/<option>/` input/golden pairs and a
      `tests/options/<XML_OPTION>.rs` file covering the interesting values
      plus an absent-option default check via `format(INPUT)`: the four
      `*_ON_NEW_LINE` flags at `true` (absent/false = today, already covered
      by the default golden); `SPECIAL_ELSE_IF_TREATMENT` at `true`/`false`;
      `KEEP_CONTROL_STATEMENT_IN_ONE_LINE` with a same-line brace-less input
      (kept when on, split when off) and a multi-line input (unchanged when
      on — proves default byte-identity); `LAMBDA_BRACE_STYLE` incl. a case
      setting it to `NextLine` while `BRACE_STYLE` stays end-of-line to prove
      independence; wire each module into tests/options.rs in alphabetical
      position (AC1, AC2, AC3).
- [x] Regression + idempotency: run `cargo test`; inspect the diff of any
      changed golden (expect none — if one changed, update it only when it
      encodes the deliberate `KEEP_CONTROL_STATEMENT_IN_ONE_LINE`
      divergence); reformat each new `*.out.java` under its style and assert
      it is unchanged (R6) (AC4).
- [x] Docs + final suite: flip the seven docs/settings/common.md rows to ✅;
      add the seven README honoured-options rows and a formatting-behaviour
      note; add the R16 requirement row to docs/requirements.md (and the
      milestones paragraph when shipped); append the docs/dev/changelog.md
      entry; run `cargo test` one last time (AC4, AC5).

Commit: not committed — worktree changes only, left for the owner to commit
(AGENTS.md).
