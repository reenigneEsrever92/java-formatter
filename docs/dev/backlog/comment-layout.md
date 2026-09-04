---
type: ChangeRequest
kind: feature
title: Honour the comment layout options
description: Apply first-column, add-space and comment-wrapping options so comments follow the scheme.
state: planned
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

# Implementation plan

## Approach

**Configuration (crates/core/src/config.rs).** Add six `bool` fields to
`JavaStyle` (L105-150) — `line_comment_at_first_column`,
`block_comment_at_first_column`, `line_comment_add_space_on_reformat`,
`line_comment_add_space_in_suppression`, `keep_first_column_comment`,
`wrap_comments` — with the IntelliJ built-in defaults (`true` / `true` /
`false` / `false` / `true` / `false`) in the `Default` impl (L152-182), and
register each as an `OptionDef` in the `OPTIONS` array (L232-567) with
`section: Section::CodeStyleJava`, `group: "Comments"` and
`OptionValue::Bool` get/set closures, following the existing bool rows (e.g.
`KEEP_SIMPLE_BLOCKS_IN_ONE_LINE`). `parse_codestyle` / `serialize_codestyle`
and the GUI are registry-driven, so the new rows are parsed, written and
rendered automatically; all six are plain bools, so no new value types. The
n/a IDE-action rows (`LINE_COMMENT_ADD_SPACE`, `BLOCK_COMMENT_ADD_SPACE`) are
not touched, per the request decisions.

**Comment rendering (crates/core/src/formatter.rs).** Today every comment
emit site echoes the node text verbatim: header comments in `Fmt::program`
(L267-269), class members in `Fmt::class_member` (L760), in-block extras in
`Fmt::block` (L1269-1270), the `line_comment` / `block_comment` arm of
`Fmt::stmt` (L1423), the stray-node fallback in `Fmt::switch_stmt`
(L1837-1841) and the `is_extra` shortcut in `Fmt::expr` (L1993-1994). Add one
shared helper — e.g. `fn comment(&self, node, indent) -> String` returning
the full rendered line — and route all six sites through it. The helper
decides, in order:

1. **Column.** A comment whose source text starts in column 1
   (`node.start_position().column == 0`) stays at column 1 when
   `keep_first_column_comment` is set; otherwise a line comment goes to
   column 1 when `line_comment_at_first_column` is set, and a block comment
   when `block_comment_at_first_column` is set; otherwise it is emitted at
   the contextual `indent`. Call sites that currently prefix `self.ind(...)`
   (class_body L722, block L1267, switch_stmt L1838) must skip that prefix
   for column-1 comments.
2. **Space after `//`.** When `line_comment_add_space_on_reformat` is set,
   insert one space after `//` if absent; `line_comment_add_space_in_suppression`
   does the same only for comments starting with `//noinspection` — the two
   flags are independent, matching the request decisions.
3. **Wrapping.** When `wrap_comments` is set and the rendered comment exceeds
   `right_margin`, break it at word boundaries; continuation lines keep the
   comment prefix (`//` for line comments, aligned text for block comments).
   The exact continuation layout is pinned by the goldens and cross-checked
   against IntelliJ when available.

Whitespace/layout only (R5): comment text is preserved, only indentation,
the optional space and (under `WRAP_COMMENTS`) line breaks change. The
comment content is never invented; unmodelled shapes keep the verbatim echo
(R4). With the defaults (all six at their built-ins, and any absent-from-
scheme style), comments are placed at column 1 — matching IntelliJ — while
no existing golden contains comments, so the suite stays green and AC4 holds.

**Tests.** One file per option under `crates/core/tests/options/` named after
the XML option, wired via `#[path]` in `tests/options.rs`, fixtures under
`tests/java/<option>/` — per the AGENTS.md golden-pair hard rules (no inline
Java strings, no `parse_codestyle` tests, only the `default_style` / `style` /
`format` / `format_with` common helpers). Each file tests both values plus an
absent-option default check; the `keep_first_column_comment` file sets the
corresponding `*_at_first_column` flag off so the keep behaviour is
observable in isolation.

**Docs.** On delivery: flip the six rows in docs/settings/common.md
("General & comments") from ❌ to ✅, add the six options to the README
honoured-options table plus a formatting-behaviour note, add a new
requirement row (R16) to docs/requirements.md, and append a changelog entry.

## Steps

- [ ] config.rs: add the six `bool` fields + `Default` values to `JavaStyle`;
      register the six `OptionDef`s in `OPTIONS` (`Section::CodeStyleJava`,
      `group: "Comments"`, `OptionValue::Bool`, IntelliJ defaults) — AC:
      option mapping and absent-option defaults.
- [ ] formatter.rs: add the shared `comment` helper implementing the column-1
      decision (`keep_first_column_comment` first, then the two
      `*_at_first_column` flags, else contextual indent), the `//`-space
      insertion (ON_REFORMAT for all line comments, IN_SUPPRESSION for
      `//noinspection` only), and `wrap_comments` margin wrapping — AC2, AC3.
- [ ] formatter.rs: route all six emit sites (program header L267-269,
      class_member L760, block extra L1269-1270, stmt L1423, switch stray
      L1837-1841, expr extra L1993-1994) through the helper, skipping the
      call-site `self.ind(...)` prefix for column-1 comments — AC2, AC3.
- [ ] tests: `line_comment_at_first_column.rs` + fixtures under
      `tests/java/line_comment_at_first_column/` (indented line comment;
      goldens for true → column 1 and false → contextual indent, plus a
      default-style check) — AC1, AC2.
- [ ] tests: `block_comment_at_first_column.rs` + fixtures (same shape for
      `/* */`) — AC1, AC2.
- [ ] tests: `keep_first_column_comment.rs` + fixtures (first-column comment
      with `line_comment_at_first_column = false`; true keeps column 1,
      false indents; default keeps column 1) — AC1, AC2.
- [ ] tests: `line_comment_add_space_on_reformat.rs` + fixtures
      (`//foo` → `// foo`; false unchanged; default false) — AC1, AC3.
- [ ] tests: `line_comment_add_space_in_suppression.rs` + fixtures
      (`//noinspection X` gains a space while ordinary `//foo` is untouched;
      false unchanged; default false) — AC1, AC3.
- [ ] tests: `wrap_comments.rs` + fixtures (long line and block comments at a
      tight `right_margin`; true wraps at the margin, false unchanged,
      default false) — AC1, AC3.
- [ ] Register all six files in `tests/options.rs` via
      `#[path = "options/<name>.rs"] mod <name>;` and run `cargo test`: the
      full suite stays green, no existing golden changes (none contain
      comments), and each new golden is idempotent (formatting it again is a
      no-op) — AC1, AC4.
- [ ] Docs: flip the six "General & comments" rows in docs/settings/common.md
      to ✅; add the six options to the README honoured-options table and a
      formatting-behaviour note; add the R16 requirement row to
      docs/requirements.md; append the entry to docs/dev/changelog.md — AC5.
- [ ] If an IntelliJ installation is available, format a representative
      fixture there and align the comment goldens (column placement, `//`
      space, wrap continuation); record the outcome in the changelog.
