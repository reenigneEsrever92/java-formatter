---
type: ChangeRequest
kind: feature
title: egui codestyle editor (crates/gui)
description: Desktop GUI in crates/gui to view and edit every code style option java-formatter supports, backed by a registry-driven option model in core and IntelliJ-correct value encodings.
state: done
priority: high
tags: [dev, gui]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-03T14:32:37Z
---

# Problem

The style options java-formatter supports can only be expressed as an IntelliJ
`codestyle.xml` written by hand (or exported by the IDE). There is no way to
explore, tweak, and validate the supported options without editing XML. The
requested GUI — "edit a codestyle with all the options currently supported" —
makes the supported surface visible and lets a user compose a style file
interactively. Building it exposes two underlying problems:

1. **Option knowledge is scattered.** The supported options are implicit in
   `parse_codestyle` (src/config.rs: the XML name, its type, and the scheme
   section it lives in), in `JavaStyle::default()` (the IntelliJ defaults),
   and again in the README and `docs/settings/` tables. A GUI that renders
   editable controls for each option cannot reuse any of this without a
   declarative description of every option in one place.
2. **The tool's value encodings diverge from IntelliJ.** `config.rs` maps
   wrap codes (`2`/`5` → chop down, `3` → wrap always) and brace codes
   (`0` → end of line, `3` → next line if wrapped) differently from IntelliJ
   (`2` → wrap always, `5` → chop down; `1` → end of line, `5` → next line if
   wrapped). This is documented as a known gap in `docs/settings/index.md`
   (Caveats) and the README. A GUI that _writes_ codestyle files would
   silently produce files IntelliJ misreads, which defeats the purpose of
   producing an IntelliJ codestyle.

# Proposal

Add `crates/gui` (package `java-formatter-gui`, binary `java-formatter-gui` —
the skeleton is created by the workspace-split refactor): an egui (eframe)
desktop application that lets the user edit every currently supported code
style option and save it as a `codestyle.xml` usable by both java-formatter
and IntelliJ.

To make that possible, core (`java-formatter-core`) gains:

- **A declarative option registry** — one entry per supported option carrying
  its XML name, scheme section (root / `<JavaCodeStyleSettings>` /
  `<codeStyleSettings language="JAVA">` / `<indentOptions>`), type
  (`bool`, `u32`, wrap enum, brace enum), allowed values, IntelliJ default,
  display group, and a human description. `parse_codestyle` is refactored to
  decode XML options through the registry, so parsing, writing, and the GUI
  all share one definition of every option.
- **`serialize_codestyle(style) -> String`** — writes a minimal `<code_scheme>`
  containing the supported options. Only options whose value differs from the
  IntelliJ default are written (IntelliJ's own export convention), keeping the
  file minimal while remaining semantically identical; absent options fall
  back to defaults in both consumers.
- **IntelliJ-correct value encodings** — `WrapStyle` / `BraceStyle` integer
  mappings are fixed to IntelliJ's codes (wrap: `0` do-not, `1` wrap-if-long,
  `2` wrap-always, `5` chop-down-if-long, `4` never produced alone; brace:
  `1` end-of-line, `2` next-line, `3` next-line-shifted, `4`
  next-line-shifted2, `5` next-line-if-wrapped). This changes how schemes
  using the affected codes parse, so the README tables, the `docs/settings`
  caveats, and the fixtures that assert the old codes are updated in the same
  change.

The GUI itself: a "New (IntelliJ defaults)" / "Open codestyle…" file entry,
options rendered per type (bool → checkbox, `u32` → drag value, wrap/brace →
labeled combo of the IntelliJ meaning), a live preview pane that formats
pasted/loaded Java with the current style in-process, and "Save" writing a
codestyle file. File access uses a path text field and drag-and-drop in v1
(no native-dialog dependency).

Docs touched: `README.md` (new GUI section; wrap/brace tables corrected),
`docs/overview.md` and `docs/architecture.md` wording as the GUI ships,
`docs/settings/index.md` (Caveats rewritten/removed once encodings match
IntelliJ), `docs/dev/changelog.md` on completion.

# Decisions

- **Registry as single source of truth.** `parse_codestyle` becomes
  registry-driven and a new `serialize_codestyle` is added; the GUI renders
  from the same registry. Adding or removing a supported option must touch
  one definition only — decided with the user on 2026-09-03, so the GUI never
  duplicates option knowledge.
- **GUI scope (v1).** Load = both "New from IntelliJ defaults" and
  "Open… an existing `codestyle.xml`"; Save writes a minimal scheme with only
  the supported options (each written only when differing from the IntelliJ
  default); a loaded IntelliJ `Project.xml` does **not** round-trip losslessly
  — other-language blocks and unknown options are dropped on save (documented
  limitation; full-fidelity in-place XML editing is deliberately out of
  scope); a live formatting preview pane is included; file access is a path
  text field plus drag-and-drop, no `rfd` dependency in v1.
- **IntelliJ-correct encodings (option b).** Core mappings are fixed to
  IntelliJ's codes in this feature, and README/docs/fixtures that asserted
  the old codes are updated, so codestyle files written by the GUI are
  interpreted correctly by both java-formatter and IntelliJ. The existing
  sample `codestyle.xml` uses only codes that mean the same under both
  encodings (`2` for next-line braces, `5` for chop-down wraps), so it needs
  no value changes; the mapping tests in `tests/config.rs` that assert the
  old codes do.
- **GUI crate created by the refactor.** The workspace-split refactor lays
  down the `crates/gui` stub; this feature fills it with the egui app.
- **Dependencies.** egui via `eframe` (desktop app), added to the
  `java-formatter-gui` package only; core gains no new dependencies for the
  registry (the XML writer uses the `quick-xml` serialize support already
  pulled in by the config module). The v1 "no `rfd`" decision was revised
  during implementation at the user's request: opening a scheme now uses a
  native file chooser (`rfd`, GUI package only).

# Acceptance criteria

- `java-formatter-gui` builds and launches an egui window showing every
  currently supported option, grouped logically, with the correct control per
  type (bool → checkbox, `u32` → drag value, wrap/brace → labeled combo).
- The registry is the single source of truth: `parse_codestyle` and
  `serialize_codestyle` are registry-driven, and
  `parse(serialize(style)) == style` for arbitrary styles
  (round-trip test), with `serialize` emitting only non-default values and
  `parse` of a minimal scheme yielding IntelliJ defaults for absent options.
- Value encodings match IntelliJ as documented in `docs/settings/index.md`:
  wrap `2` → wrap always, `5` → chop down if long; brace `1` → end of line,
  `2` → next line, `5` → next line if wrapped — covered by updated mapping
  tests in `tests/config.rs`; the `docs/settings` Caveats section is
  rewritten or removed accordingly.
- GUI behaviour: "New" shows IntelliJ defaults; "Open…" reflects the loaded
  file's values; editing an option changes the live preview; "Save" writes a
  `codestyle.xml` that the CLI (`java-formatter --style …`) accepts and that
  parses back to the same `JavaStyle`.
- README documents the GUI (build/run, load/save, preview) and the corrected
  wrap/brace tables; the changelog entry is appended when the change ships
  (fawi-implement).
- Full test suite and benches pass after the encoding change.

# Implementation plan

## Approach

The change lands in two phases: first the core work in
`java-formatter-core` (value-encoding fix, declarative option registry,
`serialize_codestyle`), then the egui GUI in `java-formatter-gui`, followed by
contracts doc updates. The registry is built before the GUI so the GUI renders
controls from data, never from a second copy of option knowledge.

### Phase 1 — core: IntelliJ-correct encodings (`crates/core/src/config.rs`)

`WrapStyle::from_int` / `BraceStyle::from_int` are corrected to IntelliJ's
codes, and `to_int` counterparts are added (needed by `serialize_codestyle`):

- `WrapStyle`: `0` → `DoNotWrap`, `1` → `WrapIfLong`, `2` → `WrapAlways`,
  `5` → `ChopDownIfLong`, anything else (`3`, `4`, unknown) → `DoNotWrap`.
  `to_int`: `DoNotWrap` → `0`, `WrapIfLong` → `1`, `WrapAlways` → `2`,
  `ChopDownIfLong` → `5`.
- `BraceStyle` loses `EndOfLineIfNotEmpty` (it was a wrong mapping of `1`;
  IntelliJ has no such code) and gains `NextLineShifted` and
  `NextLineShifted2`: `0`/unknown → `EndOfLine`, `1` → `EndOfLine`, `2` →
  `NextLine`, `3` → `NextLineShifted`, `4` → `NextLineShifted2`, `5` →
  `NextLineIfWrapped`. `to_int` is the inverse (`EndOfLine` → `1`, …).

`crates/core/src/formatter.rs` matches on these enums and must be updated in
the same step: the one-line-block checks
(`method_body`, `braces_style_inline`) change from
`EndOfLine | EndOfLineIfNotEmpty` to `EndOfLine | NextLineIfWrapped` (both are
same-line styles under the corrected mapping); the brace-placement helpers
(`with_brace`, `brace_before_body`) treat `NextLine | NextLineShifted |
NextLineShifted2` as next-line and the rest as same-line. `NextLineShifted` /
`NextLineShifted2` parse correctly but are laid out like `NextLine` — the
shifted indentation is a layout refinement the formatter does not model
(unchanged contract: unmodelled detail is not invented).

### Phase 1 — core: declarative option registry (`crates/core/src/config.rs`)

A `pub` registry module (same file or a new `registry` section in `config.rs`)
declares one entry per supported option — the single source of truth for
parsing, writing, and the GUI:

```rust
pub enum Section { Root, JavaCodeStyle, CodeStyleJava, IndentOptions }
pub enum OptionValue { Bool(bool), UInt(u32), Wrap(WrapStyle), Brace(BraceStyle) }
pub struct OptionDef {
    pub xml_name: &'static str,      // e.g. "CLASS_BRACE_STYLE"
    pub section: Section,            // where the option lives in the scheme
    pub default: OptionValue,        // IntelliJ default (== JavaStyle::default())
    pub group: &'static str,         // GUI display group, e.g. "Braces"
    pub description: &'static str,   // human-readable, shown in the GUI
    pub get: fn(&JavaStyle) -> OptionValue,
    pub set: fn(&mut JavaStyle, OptionValue),
}
pub static OPTIONS: &[OptionDef] = &[ /* one entry per JavaStyle field */ ];
```

All 24 supported options are covered (indent/tab, `SOFT_MARGINS` →
`right_margin` at `Section::Root`, braces, call/method parameter wrapping,
chain/annotation/assignment/binary wrapping, one-liners at
`Section::CodeStyleJava`, records and import-on-demand at
`Section::JavaCodeStyle`). Each entry's `default` equals the corresponding
`JavaStyle::default()` value so the serialize/parse round-trip is exact.

### Phase 1 — core: registry-driven parse + `serialize_codestyle`

`parse_codestyle` keeps its `quick-xml` deserialization of the scheme into the
XML mirror types, but the per-option decoding (find option by name in the
right section's `OptionMap`, parse per type, assign via `set`) is replaced by
a loop over `OPTIONS` — one definition of every option instead of the current
hand-written lookups. `SOFT_MARGINS` keeps today's behaviour (single integer
value; the comma-separated list form is out of scope here).

A new `pub fn serialize_codestyle(style: &JavaStyle) -> String` writes a
minimal `<code_scheme name="Project" version="173">`: for each `OPTIONS`
entry whose `get(style)` differs from its `default`, an `<option>` is emitted
in the right section (root-level options at the top,
`<JavaCodeStyleSettings>` block, `<codeStyleSettings language="JAVA">` block
with nested `<indentOptions>`), using `WrapStyle`/`BraceStyle` `to_int` and
the `quick-xml` serialize support already enabled for the config module.
Absent options mean defaults, matching IntelliJ's own export convention.

### Phase 2 — GUI (`crates/gui`)

`crates/gui/Cargo.toml` gains `java-formatter-core` (path dependency) and
`eframe` (egui desktop app) — the only new third-party dependency, confined
to the GUI package per the request's Decisions. No `rfd` in v1.

`crates/gui/src/main.rs` becomes an eframe application:

- Top bar: file-path text field (plus drag-and-drop of a `.xml` file via
  egui's `dropped_files` input), **New** (resets to `JavaStyle::default()`),
  **Open…** (parses the file with `parse_codestyle`), **Save** (writes
  `serialize_codestyle(&style)` to the path).
- Left panel: options rendered from `OPTIONS`, grouped by `group`, one
  control per type — `Bool` → checkbox, `UInt` → `DragValue`, `Wrap` /
  `Brace` → labeled `ComboBox` whose labels are the IntelliJ meanings (e.g.
  brace `2` → "Next line", `5` → "Next line if wrapped"). Editing calls
  `set` on the current `JavaStyle`.
- Right panel: a live preview — a text area holding Java source plus the
  output of `formatter::format_java(&source, &style)`, re-run on every
  option edit and on paste/load.

The GUI state is `{ path: String, style: JavaStyle, source: String }`;
parse errors on open are shown in-app rather than exiting.

### Docs

Documented behaviour that changes: the README's wrap-code line (`2` / `5` =
chop down, `3` = wrap always → IntelliJ's `2` = wrap always, `5` = chop down
if long), the README Architecture section and `docs/overview.md` (GUI is no
longer a stub), and the `docs/settings/index.md` Caveats (rewritten/removed
now that the encodings match IntelliJ). The `docs/settings` tables' defaults
are cross-checked against the registry while writing them (e.g.
`RECORD_COMPONENTS_WRAP` is listed as default `1` in `docs/settings/java.md`
but `JavaStyle::default()` ships `DoNotWrap` — a pre-existing divergence to
record rather than fix here, since fixing it would change formatting
behaviour). `docs/dev/changelog.md` gets the entry when the change ships.

### Verification commands (actual workspace crate names)

- `cargo build -p java-formatter-core`
- `cargo build -p java-formatter-gui` (first build fetches eframe and needs
  network access to crates.io)
- `cargo test -p java-formatter-core` (and the rest of the workspace)
- `cargo bench` (Criterion suite, must stay green after the encoding change)

### Trade-offs / risks

- **eframe fetch**: first GUI build downloads a large dependency tree from
  crates.io; the fawi-implement run needs network approval for it.
- **Brace codes 3/4**: parse correctly but layout like `NextLine`; shifted
  indentation stays unmodelled (documented limitation, consistent with the
  formatter's never-invent contract).
- **`3` and `4` wrap codes** now fall back to `DoNotWrap`; they are never
  produced by the IDE, and the mapping tests asserting the old codes are
  updated in the same change.

## Steps

- [x] **1. Correct the value encodings** in `crates/core/src/config.rs`:
      fix `WrapStyle::from_int`, rewrite `BraceStyle` variants
      (`EndOfLineIfNotEmpty` out; `NextLineShifted`, `NextLineShifted2` in),
      fix `BraceStyle::from_int`, add `to_int` for both enums; update the
      `BraceStyle` matches in `crates/core/src/formatter.rs` (`method_body`,
      `braces_style_inline` → `EndOfLine | NextLineIfWrapped`;
      `with_brace`, `brace_before_body` → next-line set incl. shifted); update
      the mapping tests in `crates/core/tests/config.rs` (`2` → `WrapAlways`,
      `3` → `DoNotWrap` for wraps; `3` → `NextLineShifted` for braces). The
      sample `codestyle.xml` uses only codes identical under both encodings, so
      `parses_sample_codestyle` stays green unchanged. (AC3)
- [x] **2. Add the option registry** to `crates/core/src/config.rs`:
      `Section`, `OptionValue`, `OptionDef`, and `OPTIONS` covering all 24
      supported options with xml name, section, IntelliJ default (matching
      `JavaStyle::default()`), display group, description, and get/set
      functions. (AC2)
- [x] **3. Make `parse_codestyle` registry-driven**: replace the hand-written
      per-option lookups with a loop over `OPTIONS` (same XML mirror types,
      same section handling; behaviour unchanged apart from the corrected
      encodings). (AC2)
- [x] **4. Add `serialize_codestyle(style) -> String`**: minimal
      `<code_scheme>` writing each option whose value differs from its default,
      in its section, via `to_int` and quick-xml serialization. (AC2)
- [x] **5. Add round-trip and defaults tests** in `crates/core/tests/config.rs`:
      `parse(serialize(style)) == style` for a style with several non-default
      values, `serialize` of the default style emits only `SOFT_MARGINS`/nothing
      extra, and `parse` of a minimal scheme yields `JavaStyle::default()`.
      (AC2)
- [x] **6. Wire the GUI crate**: add `java-formatter-core` and `eframe` to
      `crates/gui/Cargo.toml`; keep the binary name `java-formatter-gui`. (AC1)
- [x] **7. Implement the egui app** in `crates/gui/src/main.rs`: top bar
      (path text field, drag-and-drop, New / Open… / Save), grouped option
      controls rendered from `OPTIONS` (checkbox / drag value / labeled combo),
      and a live preview pane using `formatter::format_java`. Save writes
      `serialize_codestyle` output; Open parses with `parse_codestyle`; errors
      surface in-app. (AC1, AC4)
- [x] **8. Verify the GUI builds**: `cargo build -p java-formatter-gui`
      (needs crates.io network access for the first eframe fetch). (AC1)
- [x] **9. Update the docs**: README wrap-code line, README Architecture
      section and `docs/overview.md` (GUI is real now, build/run/load/save/
      preview), `docs/settings/index.md` Caveats rewritten/removed; cross-check
      `docs/settings` defaults against the registry (note the
      `RECORD_COMPONENTS_WRAP` divergence). (AC5)
- [x] **10. Full validation**: `cargo build` (workspace), `cargo test`
      (all crates), `cargo bench` — all green after the encoding change. (AC6)
- [x] **11. Close out** (during fawi-implement): `state: done`, `verified`
      front matter, commit link, and a `docs/dev/changelog.md` entry. (AC5)

## Closing

Shipped in commits `6483a0b` (core registry + encodings, GUI app) and
`cb9d8d6` (native file chooser, docs, close-out); changelog entry added on
2026-09-03.
