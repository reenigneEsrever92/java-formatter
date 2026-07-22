---
type: ChangeRequest
kind: feature
title: Honour clause-keyword and brace-less control-statement layout options
description: Implement else/while/catch/finally on-new-line, special else-if, lambda brace style and brace-less one-lining.
state: proposed
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
