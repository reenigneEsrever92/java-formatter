---
type: ChangeRequest
kind: feature
title: GUI editor polish — right-margin guide, scrollable panes, tree-sitter highlighting
description: Add a right-margin guide, two-way scrolling, and tree-sitter syntax highlighting to the crates/gui editor and preview panes.
state: done
priority: medium
tags: [dev, gui]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-10T00:00:00Z
---

# Problem

The desktop GUI (`crates/gui/src/main.rs`) is a codestyle editor with an options
panel beside a live editor/preview split, but its two editing surfaces are hard
to work with:

1. **No margin indicator.** Nothing shows where the configured right margin
   falls. `JavaStyle.right_margin` (default `120`) is the single field behind
   both `RIGHT_MARGIN` and `SOFT_MARGINS` in the `OPTIONS` registry, and the
   formatter's wrap decisions depend on it — but the GUI gives the user no
   visual reference for which lines reach it.
2. **Panes do not scroll.** The editor is an unscrolled
   `TextEdit::multiline(...).code_editor().desired_rows(24)` that grows with the
   source and overflows the central panel, and the preview is placed in a
   horizontal-only `ScrollArea`. Long files and long lines are not fully
   reachable.
3. **No syntax highlighting.** The preview is a single monospace
   `RichText` blob, so formatted output reads as an undifferentiated wall of
   text. The user cannot scan structure (keywords, types, strings, comments)
   while tuning a style.

# Proposal

Polish the GUI's editing surfaces in three ordered parts:

1. **Right-margin guide** — draw a vertical guide line in the preview pane at
   the `style.right_margin` column of the monospace output, using
   `Painter::vline` against the rendered text's rect. This is the effective
   margin, since `RIGHT_MARGIN` and `SOFT_MARGINS` map to the same
   `right_margin` field; nothing is drawn when it is `0` or the column is
   off-screen.
2. **Scrollable panes** — wrap both the editor and the preview in
   `ScrollArea::both()` and disable soft wrapping in both, so long files and
   long lines scroll vertically and horizontally rather than overflowing, while
   the guide stays at a fixed column.
3. **tree-sitter syntax highlighting** — add a public highlighting API to core
   (`crates/core/src/highlight.rs`, re-exported from `lib.rs`) that walks the
   same `tree-sitter-java` CST the formatter already parses (an existing core
   dependency) and returns byte-range spans tagged by token category. The GUI
   maps categories to `egui` theme colours and renders both panes from a
   `LayoutJob` — via `TextEdit::layouter` for the editor and a highlighted
   label for the preview — with results cached per source string so the tree is
   not re-parsed every frame.

Docs touched: `README.md` "Desktop GUI" section (guide, scrolling,
highlighting) and `docs/dev/changelog.md` on delivery.

# Decisions

1. **One request, three ordered parts** (agreed with the user on 2026-09-10).
   The changes are cohesive GUI polish and touch the same
   `CodestyleApp::preview_panel`, so they share one change request rather than
   three.
2. **The "margin" is the formatter's right margin.** `JavaStyle.right_margin`
   is the one field behind `RIGHT_MARGIN` and `SOFT_MARGINS`, so a single guide
   line reflects the effective margin (`120` by default). It is drawn in the
   **preview pane only** — the formatted output the margin governs; the editor
   shows the user's source, whose layout is not what the margin measures.
3. **No soft wrap; scroll in both directions.** Horizontal scrolling is what
   keeps the guide at a fixed, meaningful column; soft-wrapping a pane would
   make the guide's position vary per line. The existing side-by-side
   editor/preview columns are kept. Line numbers are deliberately out of scope.
4. **Highlight in both panes.** Both the editor and the preview benefit; the
   preview is the artefact the user tunes the style against, and the editor is
   where the source is reviewed.
5. **Highlighting logic lives in core, not the GUI.** A public
   `highlight_java` returns category-tagged byte-range spans, so the grammar
   stays in core alongside the formatter, the mapping is unit-testable without
   the GUI, and the GUI remains presentation-only (it needs no `tree-sitter`
   dependency of its own). tree-sitter node byte offsets feed
   `LayoutSection::byte_range` directly.
6. **Categories and colours.** Categories are keyword, string, character/number
   literal, comment, annotation, type, and method name. Colours are derived
   from the active `egui` theme (`visuals`), so light/dark switching works
   without a bundled palette; shipping a configurable colour scheme is out of
   scope.
7. **Robust to incomplete input.** tree-sitter recovers from syntax errors, so
   a half-typed buffer still highlights; tokens outside the modelled
   categories fall back to the default text colour. Highlight spans are cached
   by source string so typing does not re-parse on every frame.

# Acceptance criteria

- The preview pane draws a vertical guide line at the configured right-margin
  column; changing `RIGHT_MARGIN`/`SOFT_MARGINS` in the options panel moves it,
  and `right_margin == 0` draws none.
- The editor and preview each scroll vertically and horizontally: a source
  longer than the pane and lines wider than the pane are fully reachable, and
  neither pane soft-wraps.
- Java source in both panes is highlighted by category (keywords, strings,
  numbers, comments, annotations, types, methods) using colours taken from the
  active `egui` theme, in both light and dark mode; incomplete or unparsable
  source still renders using the default colour rather than failing.
- Highlighting is driven by tree-sitter through a public core API with unit
  tests covering the category mapping — a fixture containing a keyword, string,
  number, comment, annotation, type and method name maps each token to its
  category.
- The change is confined to the GUI and the new core module: no formatter
  output changes, `cargo test --workspace` stays green, and
  `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
  `cargo fmt --all -- --check` stay clean.
- The README "Desktop GUI" section documents the margin guide, two-way
  scrolling, and highlighting, and a changelog entry is appended on delivery
  (fawi-implement).

# Implementation plan

## Approach

Three parts, one new core module plus changes to the single GUI file.

**Core — new `crates/core/src/highlight.rs`, declared in `lib.rs`.** A small,
presentation-agnostic API:

- `pub enum HighlightKind { Keyword, String, Number, Comment, Annotation,
Type, Method }`;
- `pub struct HighlightSpan { pub start: usize, pub end: usize, pub kind:
HighlightKind }` (half-open byte offsets);
- `pub fn highlight_java(source: &str) -> Vec<HighlightSpan>` — builds a
  parser with `tree_sitter_java::LANGUAGE` exactly as `format_java_diagnosed`
  does, parses, walks the CST with a recursive `visit`, and returns the spans
  sorted by start offset.

`visit` matches the grammar's node kinds (verified against `tree-sitter-java`
0.23.5's `node-types.json` and `grammar.js`):

- terminal kinds push and stop — `line_comment`/`block_comment` → Comment;
  `string_literal` (covers text blocks too)/`character_literal` → String;
  `decimal_integer_literal`/`hex_integer_literal`/`octal_integer_literal`/
  `binary_integer_literal`/`decimal_floating_point_literal`/
  `hex_floating_point_literal` → Number; `type_identifier`/
  `scoped_type_identifier`/`integral_type`/`floating_point_type`/
  `boolean_type`/`void_type` → Type;
- composite kinds tag their `name` field child (the field is inherited through
  the inlined `_method_declarator`/`_constructor_declarator`/`_method_header`
  hidden rules) then still recurse — `method_declaration`/
  `constructor_declaration`/`method_invocation` → Method name,
  `annotation`/`marker_annotation` → Annotation name, and the declaration kinds
  (`class_declaration`, `interface_declaration`, `enum_declaration`,
  `record_declaration`, `annotation_type_declaration`) → Type name;
- any remaining leaf token whose `kind()` is a Java keyword (`public`, `class`,
  `if`, `new`, `this`, `true`, `false`, `null_literal`, …) → Keyword;
- otherwise recurse into children. Incomplete input parses partially, so only
  the regions tree-sitter resolved are tagged; the rest keeps the caller's
  default colour.

Unit tests live in an inline `#[cfg(test)] mod tests` in the module (the
project's other core tests are fixture-based integration tests; this maps
node kinds against the grammar, so a self-contained unit test is the right
fit). They assert that a fixture containing a keyword, string, character,
number, line/block comment, annotation, type and method name maps each token
to its category, and that text blocks and `null`/`true` are covered.

**GUI — `crates/gui/src/main.rs`.**

- Two cache fields on `CodestyleApp` (`editor_highlight`, `preview_highlight`),
  each an `Option<(String, Vec<HighlightSpan>)>` keyed by the highlighted text.
  A `highlight_job(ui, text, cache) -> LayoutJob` helper reuses cached spans
  when the text is unchanged (otherwise calls `highlight_java`), sets
  `wrap.max_width = f32::INFINITY` so neither pane soft-wraps, and appends one
  `LayoutSection` per span plus a default-format fill for the gaps. Colours come
  from `highlight_color(ui, kind)`, a palette selected by `visuals.dark_mode`
  (so light/dark works without a bundled theme); the font comes from
  `TextStyle::Monospace.resolve(ui.style())`.
- The editor gets `.desired_width(f32::INFINITY).layouter(&mut ...)` using that
  helper and is wrapped in `ScrollArea::both()`. The layouter closure and the
  `TextEdit` must borrow disjoint fields, so the pane helper destructures
  `self` (`&mut self.source` vs `&mut self.highlight_cache`).
- The preview builds the job for the formatted text, renders it as a
  non-wrapping label inside its own `ScrollArea::both()`, and then draws the
  right-margin guide: `Painter::vline` at `margin * char_width` (char width from
  `fonts_mut(|f| f.glyph_width(&mono_font, ' '))`), spanning the label's
  `y_range`, skipping entirely when `style.right_margin == 0`. The guide column
  is reserved in the scroll content (a zero-height allocation of that width) so
  it stays reachable even when no line reaches the margin.

No new dependencies: core already carries `tree-sitter` and
`tree-sitter-java`; the GUI only needs the new core API.

Docs: the README "Desktop GUI" section gains the guide/scrolling/highlighting
behaviour, and `docs/dev/changelog.md` gets an entry under 2026-09-10.

## Steps

- [x] Add `crates/core/src/highlight.rs` (enum, span, `highlight_java`,
      keyword set, inline unit tests) and `pub mod highlight;` in `lib.rs`.
- [x] Add the two highlight caches plus the `highlight_job` and
      `highlight_color` helpers to `CodestyleApp`.
- [x] Make the editor pane scrollable (`ScrollArea::both`) and highlighted via
      the `layouter`.
- [x] Make the preview pane scrollable (`ScrollArea::both`) and highlighted,
      and draw the right-margin guide (skipped when `right_margin == 0`).
- [x] Run `cargo test --workspace` — the new unit tests pass and no existing
      test changes.
- [x] Run `cargo clippy --workspace --lib --bins --tests -- -D warnings`,
      `cargo fmt --all`, and `cargo build -p java-formatter-gui`.
- [x] Update the README "Desktop GUI" section and append a
      `docs/dev/changelog.md` entry.
- [x] Mark the request `done` with `verified`, and set its backlog index row to
      `done`.

## Closing

Shipped on 2026-09-10; the changelog entry was added the same day. Verified
with `cargo test --workspace` (805 option tests plus four new `highlight` unit
tests and two new GUI widget tests), `cargo clippy --workspace --lib --bins
--tests -- -D warnings` and `cargo fmt --all -- --check`. The GUI binary
builds (`cargo build -p java-formatter-gui`) but was not launched
interactively; the margin guide's reachability is pinned by a headless egui
widget test (`crates/gui/src/main.rs`), while its on-screen appearance
(placement, colours) is verified by construction against the egui API. No
commit was made as part of this work.
