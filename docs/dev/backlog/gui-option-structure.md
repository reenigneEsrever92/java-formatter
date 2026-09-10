---
type: ChangeRequest
kind: improvement
title: Structure the GUI config options into ordered, collapsible sections
description: Give the core OPTIONS registry an explicit ordered section/sub-section model and make the GUI render each section once, fixing the repeated "Wrapping"/"Enums" headings and grouping the 258 options logically.
state: done
priority: medium
tags: [dev, improvement, gui]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-10T00:00:00Z
---

# Problem

The desktop GUI's options panel (`crates/gui/src/main.rs::options_panel`)
renders a heading only when an entry's `group` differs from the _previous_
entry, so it assumes every group's options are contiguous in the `OPTIONS`
registry (`crates/core/src/config.rs`). They are not: the registry is ordered by
topic and by the order options were added, so `"Wrapping"` is emitted **four**
times (runs of 1, 23, 11 and 1 entries) and `"Enums"` twice, split around a
block of `"Records & annotations"`. The panel therefore shows the same section
title scattered down its length, and a user cannot form a map of what lives
where.

The grouping is also flat and unbalanced, and occasionally wrong. A single
`group: &'static str` per option yields 19 groups ranging from 73 entries
(`"Spaces"`) and 36 (`"Wrapping"`) down to 2 (`"Enums"`, `"Text blocks"`,
`"Multi-catch"`, `"Builder methods"`), with no hierarchy and an ordering that
depends only on array position — `"Alignment"` was in fact inserted between the
wrapping entries on purpose, which is exactly why the wrapping family keeps
fragmenting. A few options sit in surprising sections: the switch/case
indentation options and `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS` are under
`"Braces"`, and `SPACE_INSIDE_ONE_LINE_ENUM_BRACES` is under
`"Records & annotations"`.

# Proposal

Give the registry an explicit, ordered section model and make the GUI a robust
consumer of it. Add an ordered group identity to `OptionDef` — a `Group` enum
whose declaration order defines display order, plus a table giving each group a
title and an optional parent — and change `options_panel` to collect options per
group and render each top-level section (and each sub-section) exactly once, so
registry order can never split a heading again.

Re-group the options into that model: keep the current IntelliJ-ish top-level
names, nest the two mega-groups (`Spaces`, `Wrapping`) into named sub-sections,
fold the small language-feature families (Records & annotations, Enums,
Deconstruction patterns, Text blocks, Multi-catch) under one "Language features"
parent, and move the mis-placed options to where users expect them. The section
order follows IntelliJ's _Code Style → Java_ tab order, and the panel gains
collapsible sections and a search box that filters options by XML name and
description.

Docs touched: `README.md` "Desktop GUI" section and `docs/dev/changelog.md` on
delivery.

# Decisions

1. **Classification: improvement, not a fix or refactor** (agreed with the user
   on 2026-09-10). This enhances the existing GUI's usability and the registry's
   presentation data without adding a capability and without restructuring
   modules, so it is `kind: improvement`.
2. **The structure lives in core.** Grouping stays a property of the registry in
   `crates/core/src/config.rs`, keeping `OPTIONS` the single source of truth for
   the GUI (and reusable by CLI/docs), per the existing "registry as single
   source of truth" decision. The GUI only renders it.
3. **Explicit ordered ids instead of array position.** `OptionDef.group`
   becomes a `Group` enum, with a `GROUPS` table that declares the sections in
   display order and supplies each group's title and optional parent. Display
   order therefore comes from the table, not from the array position of an
   option. This removes the root cause of the repeats: a heading can no longer be
   split by where an unrelated entry was appended. Nesting is one level
   (top-level section → sub-section) — deep trees are deliberately out of scope.
4. **GUI groups by id, order-independently.** `options_panel` first collects
   `OPTIONS` into per-group lists, then walks the ordered group table and renders
   each section once. No formatter or registry order can change the rendered
   structure. Sections become `egui::CollapsingHeader`s (all expanded by
   default) and a search box filters rows by `xml_name` and `description`,
   hiding sections that have no matches.
5. **Taxonomy: keep the top-level names, add one level of nesting, fix the
   misplacements.** Top-level order follows IntelliJ: Indentation, Spaces,
   Wrapping, Braces, Alignment, Blank lines, Comments, Javadoc, One-liners,
   Imports, Language features, Margins. `Spaces` splits into Around operators /
   Separators / Type parameters & arguments / Parentheses & brackets / Braces;
   `Wrapping` splits into Parameters (folding the current "Call parameters" and
   "Method parameters") / Method call chains (folding "Builder methods" plus
   `WRAP_SEMICOLON_AFTER_CALL_CHAIN`) / Expressions & statements / Arrays &
   initializers / Declarations / Annotations / Switch / Long lines. The current
   `"Records & annotations"`, `"Enums"`, `"Deconstruction patterns"`,
   `"Text blocks"` and `"Multi-catch"` groups become sub-sections under
   "Language features". `INDENT_CASE_FROM_SWITCH`,
   `CASE_STATEMENT_ON_NEW_LINE`, `INDENT_BREAK_FROM_CASE` and
   `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS` move from "Braces" to "Indentation";
   `SPACE_INSIDE_ONE_LINE_ENUM_BRACES` moves from "Records & annotations" to
   "Enums". This refines the Builder-methods placement sketched when settling
   this request: it belongs with the call-chain wrapping it governs rather than
   with the language-feature families. The exact per-option assignment of all
   258 entries is finalised in the implementation plan (`fawi-plan`).
6. **What deliberately stays the same.** The formatter's output does not change;
   `parse_codestyle`/`serialize_codestyle` stay registry-driven and
   order-independent by name, so a saved scheme is semantically identical even
   though non-default options may be emitted in a different XML order. The
   import-layout table editor and option presets remain out of scope, and no
   option is added or removed.
7. **How the improvement is measured.** Every section title appears exactly
   once; section order is driven by the declared group order (a core test proves
   each group is declared once with a title and that parents precede children,
   and a GUI widget test proves the rendered section list is unique and ordered
   regardless of `OPTIONS` order); the 258 options stay reachable and evenly
   distributed across sections; and a user can locate an option by typing part
   of its name.

# Acceptance criteria

- Each section title renders exactly once: `"Wrapping"` and `"Enums"` each
  produce a single top-level heading (with sub-sections), and no group heading
  repeats anywhere in the panel.
- Display order comes from the registry's declared group order, not from
  `OPTIONS` array position: a core unit test asserts every group referenced by
  `OPTIONS` is declared exactly once in the `GROUPS` table with a non-empty
  title and that every parent precedes its children, and a GUI widget test
  asserts the collected section list is unique and in the declared order.
- All 258 supported options remain present, reachable and editable, each under
  exactly one section; `Spaces` and `Wrapping` expose their sub-sections; the
  language-feature families sit under a single "Language features" parent;
  switch/case indentation and `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS` are under
  "Indentation"; `SPACE_INSIDE_ONE_LINE_ENUM_BRACES` is under "Enums".
- Sections are collapsible (expanded by default), and a search box filters
  options by XML name and description across sections, hiding empty sections;
  clearing it restores every row.
- Editing an option still updates the live preview, and a saved minimal
  `codestyle.xml` is semantically unchanged — `parse_codestyle(serialize_codestyle(s)) == s`
  still holds and only the order of emitted `<option>` elements may differ.
- No formatter output changes: `cargo test --workspace` is green, and
  `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
  `cargo fmt --all -- --check` stay clean.
- The README "Desktop GUI" section documents the sectioned, collapsible,
  searchable options panel, and a changelog entry is appended on delivery
  (`fawi-implement`).

# Implementation plan

## Approach

The change is confined to the core registry and its single GUI consumer (plus
docs); no formatter code, no new dependency.

**Core — `crates/core/src/config.rs`.**

- Add a `Group` enum next to the existing `Section` enum. Its variants are the
  top-level sections and the one-level sub-sections, and it derives
  `Debug, Clone, Copy, PartialEq, Eq`. Display order is **not** the declaration
  order.
- Add `pub struct GroupDef { pub id: Group, pub title: &'static str, pub parent:
Option<Group> }` and `pub static GROUPS: &[GroupDef]`, listed parent-first in
  display order: Indentation (Tabs / Indents / Per-construct indents / Members &
  control statements), Spaces (Around operators / Separators / Type parameters &
  arguments / Parentheses & brackets / Braces & clause keywords), Wrapping
  (Parameters / Method call chains / Expressions & statements / Arrays &
  initialisers / Declarations / Annotations / Switch / Long lines), Braces,
  Alignment, Blank lines, Comments, Javadoc, One-liners, Imports, Language
  features (Records / Annotation layout / Enums / Deconstruction patterns / Text
  blocks / Multi-catch), Margins. Titles are unique (the second annotation group
  is titled "Annotation layout").
- Change `OptionDef.group` from `&'static str` to `Group` and re-order the
  `OPTIONS` array so entries of a group are contiguous in `GROUPS` order (a
  scripted regroup of all 258 entries, preserving each group's original relative
  order and the `// --- … ---` section comments). Nine options move section: the
  switch/case indentation trio and `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS` to
  Indentation, `SPACE_INSIDE_ONE_LINE_ENUM_BRACES` to Enums, `PREFER_PARAMETERS_WRAP`
  into Wrapping/Parameters, and the annotation wrap options into
  Wrapping/Annotations.
- Register each option exactly once against its group; add an inline
  `#[cfg(test)] mod tests` (the precedent set by `highlight.rs`) asserting every
  `OPTIONS` group is declared exactly once, every declared group has at least one
  option, titles are unique, there are twelve top-level sections, and every
  parent precedes its children.

**GUI — `crates/gui/src/main.rs`.**

- Turn `option_row` into a free function `fn option_row(ui, style: &mut
JavaStyle, def: &OptionDef)` (it only touches the style), so the panel can
  render while holding a single `&mut self.style` borrow.
- Add an `options_filter: String` field to `CodestyleApp` and a pure
  `panel_layout(filter) -> Vec<PanelSection>` helper (used by the panel **and**
  its test) that walks `GROUPS` parent-first, filters options by `xml_name` /
  `description` (case-insensitive), drops empty sections, and returns each
  top-level section with its direct options and its sub-sections.
- Rewrite `options_panel` to draw a filter text box and, inside the scroll area,
  one `egui::CollapsingHeader` per top-level section (open by default) containing
  its direct option rows and a nested `CollapsingHeader` per sub-section. Each
  title is rendered from `GROUPS`, so registry order can never split a heading.
- Extend the existing inline `mod tests` with section-shape tests over
  `panel_layout`: no duplicate section or sub-section titles, `"Wrapping"` and
  `"Enums"` appear once, and a filter query keeps only matching sections.

**Docs.** `README.md` "Desktop GUI" gains the sectioned / collapsible / searchable
panel description; `docs/dev/changelog.md` gets an entry under 2026-09-10.

## Steps

- [x] Add the `Group` enum, `GroupDef`, and the `GROUPS` table to
      `crates/core/src/config.rs`.
- [x] Change `OptionDef.group` to `Group` and regroup / re-order all 258
      `OPTIONS` entries into the new sections.
- [x] Add the core registry invariant tests.
- [x] Add `panel_layout` / `option_matches`, make `option_row` free, and add the
      filter field to `CodestyleApp`.
- [x] Rewrite `options_panel` with collapsible sections and the filter box, and
      add the GUI section-shape tests.
- [x] Run `cargo test --workspace` — the registry/GUI invariants pass and no
      existing test changes.
- [x] Run `cargo clippy --workspace --lib --bins --tests -- -D warnings`,
      `cargo fmt --all`, and `cargo build -p java-formatter-gui`.
- [x] Update the README "Desktop GUI" section and append a
      `docs/dev/changelog.md` entry.
- [x] Mark the request `done` with `verified`, and set its backlog index row to
      `done`.

## Closing

Shipped on 2026-09-10. Verified with `cargo test --workspace` (810 core tests,
including five new registry invariant tests in `config.rs`, plus six GUI tests,
four of them new), `cargo clippy --workspace --lib --bins --tests -- -D warnings`
and `cargo fmt --all -- --check`. The GUI binary builds
(`cargo build -p java-formatter-gui`) but was not launched interactively; the
sectioned layout is pinned by the headless `panel_layout` tests, while its
on-screen appearance is verified by construction against the egui API. No commit
was made as part of this work.
