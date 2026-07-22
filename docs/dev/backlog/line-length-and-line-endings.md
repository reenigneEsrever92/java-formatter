---
type: ChangeRequest
kind: feature
title: Honour the right margin, line separator and hard line-wrapping options
description: Implement RIGHT_MARGIN, LINE_SEPARATOR, WRAP_LONG_LINES and KEEP_LINE_BREAKS so line-length limits and separators follow the scheme.
state: proposed
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
