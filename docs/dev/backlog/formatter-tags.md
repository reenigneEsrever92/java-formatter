---
type: ChangeRequest
kind: feature
title: Honour the formatter control tags (FORMATTER_TAGS_ENABLED, @formatter:off / @formatter:on)
description: Implement the IntelliJ formatter-tag feature — a // @formatter:off … // @formatter:on region is preserved byte-for-byte, gated by FORMATTER_TAGS_ENABLED with configurable tags and optional regexp matching.
state: done
verified: { by: maintainer, at: 2026-09-11T00:00:00Z }
priority: medium
tags: [dev, comments]
owner: maintainer
---

# Problem

The formatter re-synthesizes every construct from the CST, so there is no way
to tell it "leave this block alone". Hand-tuned or generated code — a long
table literal, ASCII art in a comment, deliberately aligned expressions — gets
reformatted like everything else, and a team following an IntelliJ scheme
cannot use the mechanism the IDE itself offers for this. IntelliJ solves it
with formatter control tags: a `// @formatter:off` comment freezes the region
until a matching `// @formatter:on`, with the region emitted untouched. The
four governing options — `FORMATTER_TAGS_ENABLED`, `FORMATTER_OFF_TAG`,
`FORMATTER_ON_TAG`, `FORMATTER_TAGS_ACCEPT_REGEXP` — are already documented in
`docs/settings/common.md` (root-level options table) but marked `n/a`:
java-formatter ignores them, so a scheme carrying them silently loses the
protection IntelliJ would apply.

# Proposal

Implement the IntelliJ formatter-tag feature in the core formatter
(`crates/core/src/formatter.rs`), gated by the four root-level options
(`crates/core/src/config.rs`, `Section::Root`, like `RIGHT_MARGIN` /
`LINE_SEPARATOR`):

- `FORMATTER_TAGS_ENABLED` (default `true`) gates the feature; a second off tag
  is ignored while off, an on tag without a preceding off is ignored, and a
  `// @formatter:off` with no matching on protects to the end of the file —
  all matching IntelliJ.
- `FORMATTER_OFF_TAG` / `FORMATTER_ON_TAG` (defaults `@formatter:off` /
  `@formatter:on`) set the marker texts, matched **comment-scoped** rather than
  IntelliJ's raw substring scan (see Decisions): a marker is recognized only in
  a `//` line comment or a `/* */` block comment whose content, trimmed of
  surrounding whitespace, equals the tag case-insensitively; an empty tag never
  matches.
- `FORMATTER_TAGS_ACCEPT_REGEXP` (default `false`) compiles the two tags as
  regexes and matches them against the comment content with `find()`; a
  malformed regex falls back to literal matching, like IntelliJ.

A protected region spans from the start of the source line containing the off
marker to the start of the source line containing the on marker, and is
emitted **byte-for-byte** from the source: no re-indentation, no wrapping, no
line joining. The marker lines are part of the region, so the off line (and
its leading indentation) is preserved; the on line is reformatted like any
other line once the region closes. Because the emitter lays out units
(top-level types, class members, statements) rather than emitting linearly,
any such unit whose source span overlaps a protected range is echoed verbatim
from the source instead of being re-synthesized — the same "preserved
verbatim" mechanism the formatter already uses for constructs it does not
model (R4). Overlapping units keep the surrounding blank-line policy at the
unit level only; the region's own interior whitespace is never touched.

The `WRAP_LONG_LINES` post-pass (which hard-wraps over-margin lines after the
tree walk) consults the same protected-range table and leaves protected lines
alone, so an over-margin line inside `off` / `on` is never wrapped. The
finalisation step that normalizes line endings still applies the configured
`LINE_SEPARATOR` inside a protected region (as it already does for preserved
module imports), so the scheme-level separator contract is unchanged; the
region's text and interior whitespace are otherwise untouched.

The options surface in the GUI automatically through the `OPTIONS` registry,
under a new "Formatter tags" sub-group of the existing `Comments` group.

# Decisions

- **Honour all four IntelliJ options — user choice.** `FORMATTER_TAGS_ENABLED`
  gates the feature, `FORMATTER_OFF_TAG` / `FORMATTER_ON_TAG` make the marker
  texts configurable, and `FORMATTER_TAGS_ACCEPT_REGEXP` supports regexp
  markers. The defaults match IntelliJ (enabled since 2023.1, tags
  `@formatter:off` / `@formatter:on`, regexp off), so the plain
  `// @formatter:off` case works with no scheme and with IntelliJ's built-in
  defaults; a scheme that disables the feature is respected. Regexp matching is
  included rather than deferred because the option is already documented and a
  scheme carrying it must keep working.
- **Comment-scoped marker recognition instead of IntelliJ's substring scan —
  user choice.** IntelliJ matches the tag as a case-insensitive substring
  anywhere in the raw file text, so a string literal or prose mention of
  `@formatter:off` would disable formatting (an IntelliJ footgun). We
  recognize a marker only when the content of a `//` line comment or a
  `/* */` block comment equals the tag after trimming surrounding whitespace
  (case-insensitive, matching IntelliJ's case folding). This handles every
  real-world marker form and never triggers from strings or prose; the
  deliberate divergence is recorded so parity is understood to be
  comment-scoped.
- **Byte-for-byte preservation, line-aligned region — user choice.** The
  preserved region spans from the start of the off-marker line to the start of
  the on-marker line and is emitted exactly as written: no re-indentation even
  when the surrounding style would indent differently, no wrapping, no
  blank-line policy inside the region. This matches IntelliJ's "untouched"
  semantics.
- **Region boundaries are honoured at unit granularity (types, class members,
  statements) — technical consequence.** The emitter re-synthesizes each unit
  independently and has no linear output channel, so a unit whose source span
  overlaps a protected range is echoed whole from the source rather than
  re-synthesized. The common case — a region spanning whole statements /
  members / types — is exactly IntelliJ's result, and output stays idempotent
  (R6); a region that cuts through the middle of one expression protects the
  whole enclosing unit, freezing slightly more than IntelliJ would. This
  divergence is accepted: it is rare, safe (nothing is re-synthesized inside,
  R4/R5), and keeps the feature tractable. Reformatting the output reproduces
  it, because the markers survive and the region re-protects itself (R6).
- **Unmatched `off` protects to end of file; nested tags are inert — user
  choice.** A second `off` while off is ignored, an `on` without `off` is
  ignored, and a dangling `off` protects the remainder of the file, all
  matching IntelliJ. Tags inside a protected region do nothing, so there is no
  nesting.
- **`WRAP_LONG_LINES` skips protected lines — user choice (confirmed as
  required).** The post-pass consults the protected-range table so an
  over-margin line inside `off` / `on` is never hard-wrapped; this is forced
  by the byte-for-byte decision.
- **Line endings inside a protected region are still normalized to the
  configured `LINE_SEPARATOR` — technical consequence.** The finalisation step
  applies the scheme's separator to every line end, including protected
  content, exactly as it already does for preserved `import module …;` lines;
  text and interior whitespace are untouched. This keeps the root
  `LINE_SEPARATOR` contract (R22) uniform.
- **GUI placement: "Formatter tags" sub-group under `Comments` — user
  choice (defaulted).** The four options are root-level scheme options but
  conceptually comment/formatting control; a sub-group of the existing
  `Comments` group (config.rs `GROUPS` table) keeps the GUI panel stable. Any
  other placement (e.g. its own top-level section) is easy to move later.

Docs touched: `docs/settings/common.md` (the four `FORMATTER_*` rows flip from
`n/a` to ✅, with the comment-scoped and regexp-fallback notes),
`docs/requirements.md` (new R43 row for the formatter-tag feature, plus a note
under the requirement table recording the comment-scoped divergence from
IntelliJ's substring scan), `docs/dev/backlog/index.md` (this entry, on
delivery), and `docs/dev/changelog.md` (on delivery).

# Acceptance criteria

- With the default style, `// @formatter:off` … `// @formatter:on` region is
  preserved byte-for-byte: every byte between the start of the off line and
  the start of the on line is emitted exactly as in the input, including
  original indentation, blank lines, and over-margin lines.
- Both `// @formatter:off` (line) and `/* @formatter:off */` (block) markers
  work, case-insensitively; a marker inside a string literal or a prose
  comment mention never triggers.
- An `off` with no matching `on` protects to the end of the file; a second
  `off` is ignored; an `on` without `off` is ignored; tags inside a protected
  region are inert.
- `FORMATTER_TAGS_ENABLED=false` restores current behaviour byte-for-byte on
  the existing test suite (no golden changes), as do absent options and the
  built-in defaults.
- Custom `FORMATTER_OFF_TAG` / `FORMATTER_ON_TAG` values (e.g. `OFF` / `ON`)
  are honoured; an empty tag never matches.
- `FORMATTER_TAGS_ACCEPT_REGEXP=true` treats the configured tags as regexes
  (e.g. `@formatter:\s*off` matches `@formatter:off`); a malformed regex falls
  back to literal matching.
- The `WRAP_LONG_LINES` post-pass never wraps a line inside a protected
  region, and the output inside the region is a fixed point of formatting
  (R6): formatting the formatted output changes nothing.
- The four options parse from a `codestyle.xml` scheme (root-level `<option>`
  entries), serialize back, and appear in the GUI under the new "Formatter
  tags" group.
- Without any scheme (default style) the region works out of the box, and a
  file with no tags formats exactly as before.
- New tests follow the per-option suite convention:
  `crates/core/tests/options/formatter_tags.rs` with golden fixtures under
  `tests/java/formatter_tags/`, covering at least: default-tag off/on at class
  level, at member level, inside a method body, an unmatched off to EOF, block
  comments, case-insensitivity, a tag in a string that does not trigger,
  `FORMATTER_TAGS_ENABLED=false`, custom tags, regexp tags, a malformed regex,
  and `WRAP_LONG_LINES` skipping the region. The full existing suite must stay
  green (R6: no golden changes for tag-free input).

# Implementation plan

## Approach

### 1. Config: four root-level options (`crates/core/src/config.rs`)

Add a `--- formatter tags ---` field group to `JavaStyle`:

- `pub formatter_tags_enabled: bool` — default `true` (IntelliJ ≥ 2023.1).
- `pub formatter_off_tag: String` — default `"@formatter:off"`.
- `pub formatter_on_tag: String` — default `"@formatter:on"`.
- `pub formatter_tags_accept_regexp: bool` — default `false`.

Add four `OptionDef` entries (`section: Section::Root`, like `RIGHT_MARGIN` /
`LINE_SEPARATOR`): `FORMATTER_TAGS_ENABLED` (Bool), `FORMATTER_OFF_TAG` /
`FORMATTER_ON_TAG` (String, registry `default` = the real non-empty default so
`parse_codestyle` / `serialize_codestyle` round-trip is exact), and
`FORMATTER_TAGS_ACCEPT_REGEXP` (Bool). Add a `Group::FormatterTags` variant and a
`GROUPS` entry `{ id: FormatterTags, title: "Formatter tags", parent:
Some(Group::Comments) }`; the four options reference that group so the GUI
shows them under Comments → Formatter tags automatically. Put the `OptionDef`s
adjacent to the other `Section::Root` entries (SOFT_MARGINS / RIGHT_MARGIN /
LINE_SEPARATOR block around L4587).

### 2. Dependency: `regex` (`Cargo.toml`)

Add `regex = "1"` to `[workspace.dependencies]` in the root `Cargo.toml` and
reference it from `crates/core/Cargo.toml` (`regex = { workspace = true }`), for
`FORMATTER_TAGS_ACCEPT_REGEXP`.

### 3. Protected-range pre-scan (`crates/core/src/formatter.rs`)

Add `prepare_protected_ranges(src: &str, style: &JavaStyle) -> Vec<ProtectedRange>`,
mirroring `prepare_module_imports` placement in `format_java_diagnosed`:

- `ProtectedRange { start: usize, end: usize }` — byte offsets; `start` is the
  start of the line containing the off marker, `end` the start of the line
  containing the on marker (or `src.len()` for an unmatched off).
- Lexical line scan (a small scanner in the style of `scan_line`, tracking
  `Str` / `Char` / `LineComment` / `BlockComment` / `TextBlock` state) so a tag
  inside a string literal or prose is never a marker (comment-scoped). For each
  comment encountered, take its interior text: after `//` for line comments,
  between `/*` and `*/` for block comments, trimmed.
- Marker match: literal mode compares the trimmed comment text to the tag with
  `eq_ignore_ascii_case`; regexp mode (when `formatter_tags_accept_regexp`)
  compiles `Regex::new(&tag)` once (per format call) and uses `find()` on the
  comment text, falling back to literal comparison when the regex fails to
  compile (like IntelliJ). An empty tag never matches.
- State machine: off when not in a region starts a region at
  `line_start(comment)`; on while in a region closes it at `line_start(comment)`;
  a second off while off is ignored; an on without off is ignored; a trailing
  off closes at `src.len()`. `line_start(byte)` = `src[..byte].rfind('\n') + 1`
  (0 when absent). Ranges come out sorted and non-overlapping.
- Gate: when `!style.formatter_tags_enabled`, return `Vec::new()` (no protected
  ranges) — tags are inert and output is byte-identical to today.

### 4. Plumb ranges into `Fmt` and emit verbatim slices

- `struct Fmt` gains `protected: Vec<ProtectedRange>` and
  `protected_out: Cell<Vec<(usize, usize)>>` — output line ranges of emitted
  protected slices, filled while `out` is assembled, consumed by the
  `WRAP_LONG_LINES` pass (the only post-pass that could alter protected content;
  a small interior-mutable accumulator avoids threading a `&mut Vec` through the
  whole recursive emitter — documented in a comment).
- Helpers on `Fmt`:
  - `fn in_protected(&self, start: usize, end: usize) -> bool` — span overlaps
    any range.
  - `fn protected_slice(&self, unit_start: usize) -> Option<(usize, usize)>` —
    for a unit beginning at `unit_start` inside a range, the slice byte bounds
    covering the maximal protected run: from `line_start(unit_start)` to the
    line end of the last consecutive covered unit (which for line-aligned units
    equals the region's line-aligned end; a region that cuts through a unit
    freezes the whole enclosing unit's lines — the accepted divergence).
  - `fn verbatim_lines(&self, start: usize, end: usize) -> &'s str` — the raw
    source slice, trailing line end stripped so callers' `'\n'` postfix applies.
  - `fn push_protected(&self, out: &mut String, start: usize, end: usize)` —
    appends the verbatim lines to `out`, records the output line span in
    `protected_out`, and returns the new line count so callers can advance.
- **Emission sites** — each list-iteration loop checks the current unit and,
  when it overlaps a range, emits the maximal run of covered units as one
  verbatim slice (`indented: false`, `align: None` — the slice carries its own
  indentation and breaks columnar runs) instead of re-synthesizing units
  individually; the loop index advances past the run; `prev`/gap bookkeeping
  uses the run's byte bounds:
  - `program()` — the top-level `top_types` loop (region spanning whole types).
  - `class_body()` — the member loop (and its `leading` comment buffer: covered
    comments are inside the run).
  - `block()` — the statement loop (method bodies, lambda bodies, …).
  - `enum_body()` — both the constants loop and the member loop after `;`.
  - `switch_block()` body loop and `switch_group()` — a protected region inside
    a switch (groups / rules / comments covered) is emitted verbatim.
- **One-line collapse guards**: the collapse paths (`method_body`
  `keep_simple_methods_in_one_line`, `simple_class_one_line`,
  `enum_one_line_body`, `switch_rule` / `switch_group` inline bodies, the
  `KEEP_CONTROL_STATEMENT_IN_ONE_LINE` joins) already refuse multi-line bodies
  via `text.contains('\n')` — a protected slice contains the own-line marker
  comments, so these paths naturally bail out; add an explicit
  `in_protected(start, end)` guard at each entry point for the pathological
  single-line region so a protected construct is never collapsed.
- `switch_rule` / `switch_group` already echo whole nodes verbatim (R4) in
  their wrapped fallbacks; the overlap check makes that path consistent.
  `local_var` / `field_decl` verbatim echoes (comment-in-declaration) are
  unrelated and unaffected.

### 5. `WRAP_LONG_LINES` skips protected lines (`crates/core/src/formatter.rs`)

Change `wrap_long_lines(text, style)` to
`wrap_long_lines(text, style, protected_lines: &[(usize, usize)])`; in the
per-line loop, a line whose index falls in a protected output-line range is
pushed unchanged (same branch as block-comment / text-block span start). The
line ranges are `protected_out` collected during emission; `format_java_diagnosed`
reads them after `fmt.program(...)` (`if style.wrap_long_lines { out =
wrap_long_lines(&out, style, &fmt.protected_out.get()) }`).

### 6. `format_java_diagnosed` wiring

Compute `let protected = prepare_protected_ranges(source, style);` before
constructing `Fmt`; pass it in; keep the module-import masking unchanged (the
masked parse runs on `parse_src`, protected ranges live on the original
`source` — a protected region may contain `import module …;` lines, which are
preserved both ways).

### 7. Tests

New `crates/core/tests/options/formatter_tags.rs` (registered in
`tests/options.rs` alphabetically, after `for_statement_wrap`) with golden
fixtures under `tests/java/formatter_tags/`, using the shared `style()` /
`format_with()` helpers and a `golden` fixed-point assertion. Fixtures
(input `.java` + expected `.out.java`):

1. `class_members` — off/on around deliberately mis-indented members; region
   byte-for-byte (leading indentation, interior blank lines untouched), on
   line reformatted.
2. `method_body` — off/on inside a method body at statement granularity.
3. `top_level_types` — off/on spanning whole top-level declarations.
4. `unmatched_off` — off with no on protects to end of file.
5. `block_marker` — `/* @formatter:off */` / `/* @formatter:on */` markers.
6. `case_insensitive` — `// @FORMATTER:OFF` / `// @Formatter:On`.
7. `string_not_marker` — `String s = "@formatter:off";` and a prose comment
   mentioning the tag: nothing protected.
8. `on_without_off` — an `on` alone is ignored.
9. `second_off` — a second `off` while off is ignored.
10. `trailing_marker` — `int x = 1; // @formatter:off` freezes the whole
    line (region starts at the line start), including the statement.
11. `disabled` — `formatter_tags_enabled = false`: output equals normal
    formatting (default fixture formatting, no protection), and the default
    style formats a tag-free file byte-identically to today.
12. `custom_tags` — `formatter_off_tag = "OFF"`, `formatter_on_tag = "ON"`
    (`// OFF` … `// ON`).
13. `regexp_tags` — `formatter_tags_accept_regexp = true` with
    `formatter_off_tag = "@formatter:\\s*off"` matching `// @formatter:off`.
14. `malformed_regexp` — an invalid regex falls back to literal matching.
15. `wrap_long_lines_skip` — `wrap_long_lines = true`, small `right_margin`,
    an over-margin line inside the region stays untouched while an
    over-margin line outside the region wraps.

Each golden asserts the fixed point (`format(output) == output`, R6). Tag-free
fixtures must produce byte-identical output under the default style (the
`formatter_tags_enabled = true` default introduces no protected ranges). The
full workspace suite stays green: `cargo test --workspace` (existing core +
cli + gui tests unchanged), `cargo clippy --workspace --lib --bins --tests --
-D warnings`, `cargo fmt --all -- --check`.

### 8. Docs

- `README.md` — „Style files“ options table: add `FORMATTER_TAGS_ENABLED` /
  `FORMATTER_OFF_TAG` / `FORMATTER_ON_TAG` / `FORMATTER_TAGS_ACCEPT_REGEXP`
  rows next to `RIGHT_MARGIN` / `LINE_SEPARATOR`; add a „Formatting behaviour
  notes“ bullet describing the comment-scoped markers, the line-aligned
  byte-for-byte region, the unmatched-off-to-EOF rule, and the regexp
  fallback.
- `docs/settings/common.md` — flip the four `FORMATTER_*` rows from `n/a` to
  ✅ with a note on the comment-scoped recognition (divergence from IntelliJ's
  raw substring scan) and the regexp fallback.
- `docs/requirements.md` — add R43 (formatter control tags) citing R3/R4/R5/R6;
  record the comment-scoped divergence under the requirement table's notes.
- `docs/dev/backlog/index.md` — this entry on delivery.
- `docs/dev/changelog.md` — changelog bullet on delivery (step 10).

### 9. Verification

`cargo test --workspace`, `cargo clippy --workspace --lib --bins --tests --
-D warnings`, `cargo fmt --all -- --check`; confirm no pre-existing golden
changed (tag-free R6).

### 10. Closing

Set `state: done` + `verified`, update `docs/dev/backlog/index.md` and append
the changelog bullet per `fawi-implement`.

## Steps

- [ ] Add `regex` to the workspace and core manifests.
- [ ] Add the four `JavaStyle` fields with IntelliJ-default values.
- [ ] Add the four `Section::Root` `OptionDef` entries and the
      `Group::FormatterTags` group + `GROUPS` entry.
- [ ] Implement `prepare_protected_ranges` (lexical comment-scoped scan +
      regexp compile-or-fallback).
- [ ] Plumb `protected` / `protected_out` into `Fmt`; add the overlap / slice /
      push helpers.
- [ ] Wire the verbatim-run emission into `program`, `class_body`, `block`,
      `enum_body`, `switch_block` / `switch_group`; add the one-line-collapse
      guards.
- [ ] Make `wrap_long_lines` skip protected output-line ranges.
- [ ] Add `tests/options/formatter_tags.rs` + fixtures; register the module.
- [ ] Update README „Style files“ table + behaviour notes, docs/settings/common.md
      rows, docs/requirements.md R43 + note.
- [ ] Run fmt, clippy, the workspace tests; confirm no golden drift.
- [ ] Mark the request done (`state: done`, `verified`), update the backlog
      index, append the changelog bullet.
