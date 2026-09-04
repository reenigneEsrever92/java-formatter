---
type: ChangeRequest
kind: feature
title: Honour the right margin, line separator and hard line-wrapping options
description: Implement RIGHT_MARGIN, LINE_SEPARATOR, WRAP_LONG_LINES and KEEP_LINE_BREAKS so line-length limits and separators follow the scheme.
state: done
verified: { by: maintainer, at: 2026-09-04T20:31:49Z }
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

`RIGHT_MARGIN`, `LINE_SEPARATOR`, `WRAP_LONG_LINES` and `KEEP_LINE_BREAKS` are
valid IntelliJ options that java-formatter does not yet parse or apply — marked
❌ in docs/settings/common.md ("Root-level options" and "General & comments")
and safely ignored per R7 — so a scheme that sets them is only partially
honoured and output diverges from IntelliJ. Today the tool reads only
`SOFT_MARGINS` (root-level, into `JavaStyle::right_margin` via the `OPTIONS`
registry) for its line-length limit, always emits its own hard-coded newline
regardless of the scheme's `LINE_SEPARATOR`, and never hard-wraps lines past
the margin (`WRAP_LONG_LINES`) or consults `KEEP_LINE_BREAKS` when deciding
whether existing breaks survive.

# Proposal

Parse each option into a `JavaStyle` field via the `OPTIONS` registry in
crates/core/src/config.rs: `RIGHT_MARGIN` (default `120`) and `LINE_SEPARATOR`
(default system separator) as `Section::Root` entries, `WRAP_LONG_LINES`
(default `false`) and `KEEP_LINE_BREAKS` (default `true`) as
`Section::CodeStyleJava` bools per the table; absent-from-scheme options keep
the default. Apply them in crates/core/src/formatter.rs: the right margin
drives hard wrapping, `WRAP_LONG_LINES` breaks lines that exceed it,
`KEEP_LINE_BREAKS` decides whether existing line breaks are kept or reflowed,
and the configured separator is emitted at every line end, including the final
newline normalisation in `format_java_diagnosed`.

Docs touched: on delivery the implementation updates the docs/settings support
marks (❌ → ✅ for these rows), the README honoured-options table /
formatting-behaviour notes, docs/requirements.md (a new requirement row), and
docs/dev/changelog.md.

# Decisions

- **One family, one request.** Only the four listed options are added here;
  the other root-level rows (`FORMATTER_TAGS_*`, `AUTODETECT_INDENTS`,
  `OTHER_INDENT_OPTIONS`, all n/a) and the unimplemented wrapping rows stay out
  and are safely ignored (R7).
- **Defaults.** The registry records the IntelliJ built-ins from the
  docs/settings tables; default and absent-from-scheme styles keep current
  byte-identical output (LF, current margin behaviour) so existing goldens stay
  green — the formatter already emits one statement per line and no hard wraps,
  so the shipped fixtures match the defaults except where `KEEP_LINE_BREAKS`
  would keep a joined line that today's output breaks; such goldens are updated
  deliberately with this change.
- **Semantics.** Whitespace/layout only (R5); unmodelled constructs are echoed
  verbatim (R4); formatting formatted output is a no-op (R6) — a hard-wrapped
  line must re-wrap identically.
- **Encodings.** `RIGHT_MARGIN` is a plain `u32`; `WRAP_LONG_LINES` /
  `KEEP_LINE_BREAKS` are bools; `LINE_SEPARATOR` is the one listed option that
  needs a new registry value type (a string or enum over LF / CRLF / CR, per
  the table's `&#10;` / `&#13;&#10;` / `&#13;` values).

# Acceptance criteria

- A dedicated golden fixture + test file per option in crates/core/tests/options/
  (fixtures under tests/java/<option>/), each tested at its interesting values
  plus an absent-option default check.
- A scheme with `RIGHT_MARGIN` but no `SOFT_MARGINS` drives the line-length
  decisions; `LINE_SEPARATOR = &#13;&#10;` produces CRLF output including the
  final newline.
- `WRAP_LONG_LINES` hard-wraps a line longer than the margin;
  `KEEP_LINE_BREAKS` = `true` preserves and `false` reflows existing breaks.
- Default scheme output is unchanged and the whole suite stays green
  (`cargo test`); new goldens are idempotent.
- docs/settings marks flipped to ✅; README and docs/requirements.md updated.

# Implementation plan

## Approach

Four options, three mechanisms: registry plumbing in
`crates/core/src/config.rs`, a new value type for the separator, and two new
behaviours in `crates/core/src/formatter.rs` (a final output pass for the
separator and for `WRAP_LONG_LINES`, plus source-aware layout decisions for
`KEEP_LINE_BREAKS`). The engine emits and measures LF internally (`\n` in
`indent_str` / `col_after` / `fits` / the layout strings), so defaults stay
byte-identical and column arithmetic is untouched.

**Config and registry (src/config.rs).** Add a `LineSeparator` enum
(`System`, `Lf`, `Crlf`, `Cr`; `Copy`, `Eq` — `System` resolves to the
platform separator, which is `\n` on the test hosts) and an
`OptionValue::LineSep(LineSeparator)` variant; `JavaStyle` gains
`line_separator` (default `System`), `wrap_long_lines` (default `false`) and
`keep_line_breaks` (default `true`). `JavaStyle` is only ever built via
`Default` (no literal sites), so adding fields is safe. Register four
`OptionDef`s: `RIGHT_MARGIN` and `LINE_SEPARATOR` as `Section::Root`;
`WRAP_LONG_LINES` and `KEEP_LINE_BREAKS` as `Section::CodeStyleJava` bools per
the request decisions. `RIGHT_MARGIN` maps to the existing `right_margin`
field (default 120) with the same get/set as `SOFT_MARGINS`; register it
*before* `SOFT_MARGINS` in `OPTIONS` so that — because `parse_codestyle`
applies entries in order — `SOFT_MARGINS` keeps precedence when a scheme sets
both (today's behaviour preserved) and `RIGHT_MARGIN` drives the margin only
when `SOFT_MARGINS` is absent (AC2a). Consequence to accept: when the shared
field differs from 120, `serialize_codestyle` writes both root options
(redundant but round-trip-exact, which is the registry's contract). Parsing
needs a new `OptionMap::get_line_sep` (the decoded value is the actual
character sequence `\n` / `\r\n` / `\r` — quick-xml decodes `&#10;` etc.);
`parse_codestyle`'s dispatch on `def.default` and `serialize_codestyle`'s
value match get `LineSep` arms, serialising the *escaped* XML forms
`&#10;` / `&#13;&#10;` / `&#13;` (a raw newline inside an attribute would be
normalised to a space by XML parsers). The GUI (`crates/gui/src/main.rs`)
matches `OptionValue` exhaustively in `option_row` (L137-175), so it needs a
`LineSep` combo arm plus a label helper to keep the workspace compiling.

**Line separator emission (src/formatter.rs).** Do not thread a separator
through ~50 `\n` emission sites. Instead, after `fmt.program(...)` in
`format_java_diagnosed` (L64-69), run one finalisation helper on the
LF-normal output: collapse any `\r\n` that arrived via verbatim echoes of a
CRLF source (comments), `trim_end_matches('\n')` to exactly one trailing
line end, then when the resolved separator is not `\n` substitute `\n` →
separator and append the separator once. When the separator resolves to `\n`
take the current code path unchanged, so default output (and verbatim text
from LF sources) stays byte-identical. This makes CRLF (and CR) apply at
every line end including the final newline (AC2b) and stays idempotent: the
collapse-then-substitute order prevents doubled `\r`, and re-formatting a
CRLF document yields the same separators.

**`WRAP_LONG_LINES` (src/formatter.rs).** Implement as a deterministic
post-`program` line pass (before the separator substitution, since it works
on the LF text), gated on `style.wrap_long_lines`. For each line whose
logical width (`col_after`) exceeds `right_margin`, break at the rightmost
whitespace boundary at or before the margin, continuing at the line's own
leading-indent width + `continuation_indent_size` (built through the
tab-aware `indent_str`, so `USE_TAB_CHARACTER` stays consistent); repeat on
the continuation until it fits or has no breakable boundary. R5 is the hard
constraint: never split inside a string/char literal or a comment — the pass
scans each line tracking `"…"` / `'…'` (with backslash escapes) and
`//…` regions and only offers spaces outside them, and it skips comment-only
lines (`WRAP_COMMENTS` governs those). A line with no safe boundary (a long
string literal, a single token) is left over-long, exactly as IntelliJ
leaves unbreakable content. Default `false` is a no-op (AC3a, AC4); the
break points are a pure function of the flat text, so a second pass
re-produces them and the output is idempotent even with `KEEP_LINE_BREAKS`
true (the engine does not treat a hard-wrap break — which lands at an
arbitrary space — as a preserved source break).

**`KEEP_LINE_BREAKS` (src/formatter.rs).** The engine already emits one
statement per line; the decision that needs to become source-aware is
"flat single-line form vs the construct's wrapped layout". Define: when
`keep_line_breaks` is true and a construct's *source* spans multiple lines
(`node.start_position().row != node.end_position().row`), render the
canonical wrapped layout that already exists for that construct (one
argument/parameter/operand/chain-link per line at `cont(indent)`) even when
the flat form fits; when false (or the source is joined), keep today's
flatten-if-fits behaviour — which is exactly what `false` must produce
(reflow). Sites to make source-aware are the constructs with an existing
wrapped layout: call argument lists (`method_inv` / `flat_inv` / `args_wrapped`,
formatter.rs L2065-2179), declaration parameter lists (`formal_params`,
L1143-1193), assignment / local-variable / field initialisers
(`assign_expr`, `assignment`, `local_var`, `field_decl` — break after the
operator onto the continuation line), binary and ternary spines (`binary`
L2377, `ternary` L2453), call chains (`collect_chain` / `fmt_chain`),
annotation argument lists (`annotation` L1011), `new` argument lists
(`new_expr` L2256) and array initialisers (`array_init` L2559). The opt-in
`KEEP_SIMPLE_*` one-liner collapses (`one_line_body` / `if_one_line` /
`try_one_line` / `switch_one_line`) and fixed structural layouts are left
alone — like IntelliJ, the explicit keep-simple options win over line-break
retention. Approximation to record: preserved breaks land at canonical
wrap boundaries, not at the source's exact byte positions (the engine
re-renders rather than re-indenting source lines), which is what keeps the
output deterministic and idempotent. `KEEP_LINE_BREAKS` defaults to `true`;
current input fixtures contain no statements spread across lines (verified
by scanning `tests/java/`), so the default suite is expected to stay green
— the audit step below confirms that and regenerates only goldens that
deliberately encode a preserved source break (per the request decisions).

**Tests and fixtures.** Conventions per `.agents/AGENTS.md`: golden pairs
only, one file per XML option under `crates/core/tests/options/` wired via
`tests/options.rs`, fixtures under `tests/java/<option>/`, no inline Java
strings, no `assert_idempotent`, no `parse_codestyle` tests (so the
distinct-option parse mapping is verified only by the registry's round-trip
contract, and "absent-option default" is exercised as a default-style golden
— the established pattern in the current `right_margin.rs`). Name collision
to resolve first: the existing `SOFT_MARGINS` test module is misnamed
`options/right_margin.rs` with fixtures in `tests/java/right_margin/`; rename
it to `options/soft_margins.rs` and move its fixtures to
`tests/java/soft_margins/` (contents unchanged), freeing `right_margin.rs`
and `tests/java/right_margin/` for the new `RIGHT_MARGIN` module. CRLF (and
CR) goldens are files whose bytes include the real separator (`include_str!`
preserves them; the repo has no `.gitattributes`, so no auto-normalisation —
verify on check-in). New-golden idempotency (AC4) is confirmed manually by
re-formatting each `*.out.java` with its own style during implementation.

**Docs.** `docs/settings/common.md` (flip the four rows ❌ → ✅, adjust the
root-options intro and the `RIGHT_MARGIN`/`SOFT_MARGINS` effect/support
notes), `README.md` (four rows in the honoured-options table plus
formatting-behaviour notes for the separator, hard wrapping and line-break
retention), `docs/requirements.md` (a new requirement row — the next free
number is R16 — plus a line in the delivered-milestones paragraph), and a
`docs/dev/changelog.md` entry on completion.

## Steps

- [x] src/config.rs: add the `LineSeparator` enum (`System` / `Lf` / `Crlf` /
      `Cr`, `Copy` + `Eq`) with a resolve helper, add `JavaStyle` fields
      `line_separator` (`LineSeparator::System`), `wrap_long_lines` (false),
      `keep_line_breaks` (true) with matching `Default` values, and extend
      `OptionValue` with `LineSep(LineSeparator)` (AC4 defaults).
- [x] src/config.rs: register the four `OptionDef`s — `RIGHT_MARGIN` (Root,
      `UInt(120)`, same get/set as `SOFT_MARGINS`, placed *before* it so
      `SOFT_MARGINS` keeps precedence when both are set) and `LINE_SEPARATOR`
      (Root, `LineSep(System)`) in the Margins area; `WRAP_LONG_LINES`
      (false) and `KEEP_LINE_BREAKS` (true) as `CodeStyleJava` bools — each
      with group/description and default-equal get/set closures; extend
      `OptionMap` with `get_line_sep` and add `LineSep` arms to the
      `parse_codestyle` default-dispatch and to `serialize_codestyle` (writing
      the escaped forms `&#10;` / `&#13;&#10;` / `&#13;`, skipping `System`)
      (AC2a, AC1 absent-option defaults).
- [x] crates/gui/src/main.rs: add the exhaustive-match `LineSep` arm to
      `option_row` (a combo of System/LF/CRLF/CR with a label helper) so the
      workspace still compiles; `cargo check --workspace` passes.
- [x] src/formatter.rs: replace the trailing-newline logic in
      `format_java_diagnosed` with the finalisation helper (collapse
      verbatim `\r\n` → `\n`, trim to one trailing line end, substitute the
      resolved separator when it is not `\n`); default (`System` → `\n` on
      the test hosts) output stays byte-identical (AC2b, AC4).
- [x] src/formatter.rs: add the `WRAP_LONG_LINES` post-`program` line pass
      (width via `col_after`, break at the rightmost safe space ≤ margin,
      continuation at leading indent + `continuation_indent_size` via
      `indent_str`, literal/comment-aware so tokens are never split,
      comment-only lines skipped) and call it before separator substitution;
      `false` is a no-op (AC3a, AC4).
- [x] src/formatter.rs: make the listed layout sites source-aware for
      `KEEP_LINE_BREAKS` (call args `method_inv`/`flat_inv`/`args_wrapped`,
      decl params `formal_params`, initialisers `assign_expr`/`assignment`/
      `local_var`/`field_decl`, `binary`/`ternary` spines, `fmt_chain`,
      `annotation`, `new_expr` args, `array_init`): when the option is true
      and the construct's source spans rows, render the existing wrapped
      layout even if it fits; `false` keeps the flatten-if-fits path; the
      `KEEP_SIMPLE_*` and structural layouts are untouched (AC3b, AC4).
- [x] Test infra: rename the existing `SOFT_MARGINS` module
      `tests/options/right_margin.rs` → `soft_margins.rs` and move its
      fixtures `tests/java/right_margin/` → `tests/java/soft_margins/`
      (contents unchanged), rewiring `tests/options.rs` (AC1 naming).
- [x] Add per-option golden tests + fixtures under `tests/java/<option>/` for
      each of `right_margin` (RIGHT_MARGIN margin vs default 120),
      `line_separator` (default-LF, CRLF incl. final newline, CR),
      `keep_line_breaks` (true preserves / false reflows a multi-line call or
      parameter list; default golden), and `wrap_long_lines` (true wraps an
      over-margin line at a space / false and default leave it; unbreakable
      string literal not split); wire each module into `tests/options.rs`
      (AC1-AC3).
- [x] Run `cargo test`; confirm the default-style suite is unchanged (audit
      any diff — per the request decisions, regenerate only goldens that
      deliberately encode a preserved source break) and manually re-format
      each new `*.out.java` with its own style to confirm idempotency (AC4).
- [x] Docs: flip the four rows to ✅ and adjust the notes in
      docs/settings/common.md; add the four rows + behaviour notes to the
      README honoured-options table; add requirement R22 (and its milestone
      line) to docs/requirements.md; append the entry to
      docs/dev/changelog.md. Finish with a full `cargo test` run green; leave
      the work uncommitted for the owner.

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
