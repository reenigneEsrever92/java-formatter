---
type: ChangeRequest
kind: feature
title: Honour the comment layout options
description: Apply first-column, add-space and comment-wrapping options so comments follow the scheme.
state: proposed
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

The comment layout options — `LINE_COMMENT_AT_FIRST_COLUMN`,
`BLOCK_COMMENT_AT_FIRST_COLUMN`, `LINE_COMMENT_ADD_SPACE_ON_REFORMAT`,
`LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION`, `KEEP_FIRST_COLUMN_COMMENT` and
`WRAP_COMMENTS` — are valid IntelliJ options marked ❌ in docs/settings/common.md
("General & comments") and safely ignored per R7, so a scheme that sets them is
only partially honoured and comment output diverges from IntelliJ. Today
comments are echoed verbatim from the source: header comments (`Fmt::program`),
class members (`Fmt::class_member`) and in-block extras (`Fmt::block`) are all
emitted via `self.txt`, so first-column placement, the space after `//`, and
comment line length follow the input text rather than the scheme.

# Proposal

Parse each listed option into a `JavaStyle` field via the `OPTIONS` registry in
crates/core/src/config.rs (`Section::CodeStyleJava`, IntelliJ built-in defaults
from the table: `true` / `true` / `false` / `false` / `true` / `false`);
absent-from-scheme options keep the default. Apply them in
crates/core/src/formatter.rs at the comment-emitting paths: the two
`*_AT_FIRST_COLUMN` options control whether comments are pinned to column 1,
`KEEP_FIRST_COLUMN_COMMENT` keeps source first-column comments there,
`LINE_COMMENT_ADD_SPACE_ON_REFORMAT` inserts a space after `//` on reformat,
`LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION` does the same inside `// noinspection`
comments, and `WRAP_COMMENTS` wraps long comments to the right margin. The base
`LINE_COMMENT_ADD_SPACE` / `BLOCK_COMMENT_ADD_SPACE` rows are **not** part of
this request: they are comment/uncomment IDE actions marked n/a in the docs,
not reformatting options.

Docs touched: on delivery the implementation updates the docs/settings support
marks (❌ → ✅ for these rows), the README honoured-options table /
formatting-behaviour notes, docs/requirements.md (a new requirement row), and
docs/dev/changelog.md.

# Decisions

- **One family, one request.** Only the six listed options are added here; the
  n/a IDE-action rows (`LINE_COMMENT_ADD_SPACE`, `BLOCK_COMMENT_ADD_SPACE`,
  `DOCUMENTATION_LINE_COMMENT_PREFERRED`) stay out and unimplemented rows
  remain safely ignored (R7).
- **Defaults.** The registry records the IntelliJ built-ins from the table;
  default and absent-from-scheme styles keep current byte-identical output so
  existing goldens stay green.
- **Semantics.** Comment text is preserved; only indentation, the optional
  space after `//`, and (under `WRAP_COMMENTS`) line breaks inside the comment
  change — whitespace/layout only (R5); unmodelled constructs are echoed
  verbatim (R4); formatting formatted output is a no-op (R6).
- **Encodings.** All six options are plain bools; no new registry value types.

# Acceptance criteria

- A dedicated golden fixture + test file per option following the pattern in
  crates/core/tests/options/, each tested at both values plus an absent-option
  default check.
- `LINE_COMMENT_AT_FIRST_COLUMN` / `BLOCK_COMMENT_AT_FIRST_COLUMN` move comments
  to/from column 1; `KEEP_FIRST_COLUMN_COMMENT` pins source first-column
  comments.
- `LINE_COMMENT_ADD_SPACE_ON_REFORMAT` / `LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION`
  insert the space only in their documented situations; `WRAP_COMMENTS` wraps a
  long comment at the right margin.
- Default scheme output is unchanged and the suite stays green (`cargo test`);
  new goldens are idempotent.
- docs/settings marks flipped to ✅; README and docs/requirements.md updated.
