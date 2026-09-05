---
type: ChangeRequest
kind: feature
title: Honour the remaining record-header layout options
description: Implement the record-component wrapping/annotation/spacing sub-options not yet shipped.
state: done
priority: medium
tags: [dev, formatter]
owner: maintainer
verified:
  by: Zed coding agent
  at: 2026-09-05T16:00:00Z
---

# Problem

`RECORD_COMPONENTS_WRAP`, `ALIGN_MULTILINE_RECORDS` and `NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER` already ship ✅ (docs/settings/java.md "Records"), so a wrapped record header is largely handled — but the rest of the family is ❌: `RPAREN_ON_NEW_LINE_IN_RECORD_HEADER`, `SPACE_WITHIN_RECORD_HEADER` and `ANNOTATION_NEW_LINE_IN_RECORD_COMPONENT` (Records table, bools default `false`) plus `BLANK_LINES_BETWEEN_RECORD_COMPONENTS` ("Miscellaneous spacing & blank lines", int default `0`).
A scheme that sets them gets no closing-paren-on-new-line toggle, no inner header spacing, no per-component annotation placement and no blank lines between components.
These rows are safely ignored today (R7), so record-layout fidelity is only partial.

# Proposal

Add `rparen_on_new_line_in_record_header`, `space_within_record_header` and `annotation_new_line_in_record_component` (bools default `false`) and `blank_lines_between_record_components` (default `0`) to `JavaStyle` via `OptionDef` entries in the `OPTIONS` registry in crates/core/src/config.rs, in the same Java-specific section the shipped record options already use (absent → default), and apply them in crates/core/src/formatter.rs in the record-header path: the `)` of a wrapped header on its own line, `record R( String s )` inner padding, each component's annotation placed on its own line, and blank lines inserted between components of a wrapped header when the count is non-zero.

Docs touched: `docs/settings/java.md` "Records" and "Miscellaneous spacing & blank lines" marks flipped ❌→✅, the README honoured-options table and formatting-behaviour notes, `docs/requirements.md` (a new requirement row), and `docs/dev/changelog.md` on delivery.

# Decisions

1. **One family, one request.** Only the four listed options ship; other annotation-placement variants (for example on enum constants) belong to the annotation-layout request, and the remaining ❌ rows stay unimplemented and safely ignored (R7).
2. **Defaults.** The fields take the IntelliJ built-ins from the tables (`false` / `0`), so default and absent schemes keep the shipped record-header output byte-identical and the existing goldens stay green.
3. **Semantics.** R5 holds — the changes are whitespace, line breaks and blank lines only; a component's annotation tokens are moved but kept verbatim (R4), and R6 is pinned by re-formatting the new goldens.
4. **Registry.** The three bools and one count fit existing `OptionValue` variants, so no new value variant is needed; the rparen and blank-line options only take effect on a wrapped/multiline header, so a header that fits the margin keeps today's single-line rendering.

# Acceptance criteria

- `tests/options/rparen_on_new_line_in_record_header.rs` asserts a wrapped header's `)` on its own line at `true`, and the shipped layout at `false`/absent.
- `tests/options/space_within_record_header.rs` asserts `record R( String s )` padding at `true` and unchanged `record R(String s)` at `false`/absent.
- `tests/options/annotation_new_line_in_record_component.rs` asserts each component annotation on its own line at `true` (annotation tokens verbatim) and inline at `false`/absent.
- `tests/options/blank_lines_between_record_components.rs` asserts no blank lines at the absent default `0` and one blank line between wrapped components at `1`.
- Default-scheme output is unchanged and the whole suite stays green (`cargo test`); the new goldens are idempotent (R6); docs/settings marks are flipped and the README / `docs/requirements.md` / `docs/dev/changelog.md` are updated.

# Implementation plan

## Approach

Configuration and rendering, following the sibling record-family request
`binary-expression-wrapping` (engine + fixtures + golden tests) and the
`annotation-layout` option-family plans (per-option files under
`tests/options/`). The four options default to the IntelliJ built-ins (three
bools `false`, one count `0`), so default and absent schemes keep today's
record-header output byte-identical and every existing golden stays green.

**Configuration — `crates/core/src/config.rs`.** `JavaStyle` is constructed
only via `Default` (no other struct literals exist), so add four `pub` fields
to the record-specific block (L143-146) after
`new_line_after_lparen_in_record_header` — `rparen_on_new_line_in_record_header`
(bool), `space_within_record_header` (bool),
`annotation_new_line_in_record_component` (bool),
`blank_lines_between_record_components` (`u32`) — with `Default` values
`false`/`false`/`false`/`0`. Add four `OptionDef`s to the `OPTIONS` registry
right after the `NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER` def (L540-552), all
`Section::JavaCodeStyle` (the same Java-specific section the shipped record
options use), group "Records & annotations", typed `OptionValue::Bool` for the
three toggles and `OptionValue::UInt(0)` for the count, with `default` equal to
the field default so the `parse(serialize(style)) == style` round trip stays
exact (registry-driven parse/serialize and the registry-rendering GUI need no
other change; an absent scheme option keeps the field default, R7). Because the
defaults equal the IntelliJ built-ins no divergence note is needed (unlike
`RECORD_COMPONENTS_WRAP`).

**Rendering — `crates/core/src/formatter.rs` `record_components` (L642-691).**
Today the function flattens each component with `flat_param` (L648), decides
wrap from `record_components_wrap` vs the margin at the opening-paren column
(L653-659), and then emits one of two hard-coded wrapped shapes depending on
`new_line_after_lparen_in_record_header`: with the option on (L672-677) every
component goes on its own line at `inner_indent` and `)` closes alone at
`self.ind(indent)`; with it off (L678-690) the first component stays inline
after `(` and `)` is glued to the last component. Rework the wrapped path to
render per-component **blocks** — each component becomes `Vec<String>` of whole
lines at its column — so a component can contribute more than one line and the
separators/closing line are controllable. The four new options then slot in:

1. `SPACE_WITHIN_RECORD_HEADER` — when set, exactly one space sits just inside
each `(` / `)` that shares its line with a component: the flat form (L649)
becomes `( parts.join(", ") )`, the lparen-off first line `( first,`, and a
`)` glued to the last component line `last )`. A paren alone on a line (the
lparen-on layout) gets no pad. The margin decision (`fits` at L658) measures
the padded flat, and the alignment column (`open_col + 1`, L666-668) shifts by
the pad when the first component is inline, so alignment stays under the first
component as it prints.
2. `RPAREN_ON_NEW_LINE_IN_RECORD_HEADER` — in the wrapped layouts the closing
paren goes either glued after the last block (`false` → today's shape) or on
its own line at `self.ind(indent)` (`true`). The lparen-on branch already ends
with `)` alone at `self.ind(indent)`, so under both values that path is
unchanged and the existing goldens stay byte-identical; the option visibly
moves the `)` of the lparen-off layout.
3. `ANNOTATION_NEW_LINE_IN_RECORD_COMPONENT` — when set and a component is laid
out on its own line in the wrapped path and is a `formal_parameter` whose
`modifiers` child carries `annotation`/`marker_annotation` children, the block
becomes one line per annotation (formatted with the existing annotation
rendering, tokens verbatim per R4) followed by the declaration core (keyword
modifiers + type + name — the `flat_param` formal_parameter assembly with the
annotation text removed); all lines sit at the component's column. Components
without annotations, non-`formal_parameter` shapes (e.g. `spread_parameter`),
the first inline component of the lparen-off layout (which shares the `(`
line), and any header that fits the margin and stays single-line keep today's
inline rendering.
4. `BLANK_LINES_BETWEEN_RECORD_COMPONENTS` — when non-zero and the header is
wrapped, `n` bare blank lines (`\n` with no indent) are inserted between
consecutive component blocks (the inter-block separator `,\n` becomes `,`
followed by `n + 1` newlines). A non-zero count has no effect on a header that
is not wrapped (decision 4).

`blank_lines_between_record_components`, `rparen_on_new_line_in_record_header`
and `annotation_new_line_in_record_component` are only reachable after
`should_wrap`; `space_within_record_header` also applies to the flat form.
Update the function's doc comment (L635-641) to list all seven record options.
All four additions are pure whitespace/line-break changes (R5) gated on
defaults that match today's layout, so no existing golden may shift.

**Tests.** Follow the `.agents/AGENTS.md` hard rules: one golden-pair module
per option in `crates/core/tests/options/<xml_option>.rs`, wired via
`#[path]` in `crates/core/tests/options.rs` (alphabetical:
`annotation_new_line_in_record_component` before `annotation_parameter_wrap`,
`blank_lines_between_record_components` between `binary_operation_wrap` and
`brace_style`, and `rparen_on_new_line_in_record_header` +
`space_within_record_header` between `right_margin` and `tab_size`), each
starting `use super::common::*;`, doc header `//! <OPTION> — …` plus
`//! Fixtures live under tests/java/<option>/.`, fixtures under
`crates/core/tests/java/<option>/` reached by relative `include_str!` with a
shared input/golden stem, no inline Java strings and no new helpers. New
`.out.java` goldens are generated by formatting the fixture under the style,
sanity-checked against the option's semantics, and re-formatted a second time
during development to confirm idempotency (R6). Where a corner is not
IntelliJ-verifiable in the environment (annotation placement of the first
inline component, the `)` column when alignment is on), pin the golden from the
codebase's existing continuation conventions and, when an IntelliJ install is
available, cross-check and adjust — recording the outcome in the changelog, as
the sibling requests did.

## Steps

- [x] `crates/core/src/config.rs`: add the four `JavaStyle` fields +
      `Default` values (three bools `false`, count `0`) in the record block,
      and the four `OptionDef`s after `NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER`
      (`Section::JavaCodeStyle`, group "Records & annotations",
      `Bool`/`Bool`/`Bool`/`UInt`, registry defaults equal to the fields);
      `cargo build` and the suite stay green (AC5 defaults; R7 absent →
      default).
- [x] `crates/core/src/formatter.rs`: rework `record_components` per the
      approach (per-component blocks; `space_within_record_header` pad;
      rparen closing line; annotation own-line blocks; blank-line separators)
      and update its doc comment; confirm the whole existing suite stays
      byte-identical before adding new tests (AC5).
- [x] RPAREN fixture + test: `tests/java/rparen_on_new_line_in_record_header/component_wrap.java`
      (three components) wrapped via `WrapStyle::WrapAlways` + alignment off;
      goldens for the option set (`)` alone at the record indent) and
      absent/false (today's glued shape); new
      `tests/options/rparen_on_new_line_in_record_header.rs`; wire in
      `tests/options.rs` (AC1).
- [x] SPACE fixture + test: `tests/java/space_within_record_header/header.java`
      short record (e.g. `record R(String key, int value) {}`); goldens for
      the option set (`record R( String key, int value ) {}`) and
      absent/false (unchanged input); new
      `tests/options/space_within_record_header.rs`; wire in `tests/options.rs`
      (AC2).
- [x] ANNOTATION fixture + test: `tests/java/annotation_new_line_in_record_component/component_wrap.java`
      whose annotated components each sit on their own line in the wrapped
      layout (`WrapStyle::WrapAlways` + `new_line_after_lparen_in_record_header`
      true + alignment off; marker annotations kept verbatim); goldens for the
      option set (each annotation above its component) and absent/false
      (inline); new `tests/options/annotation_new_line_in_record_component.rs`;
      wire in `tests/options.rs` (AC3).
- [x] BLANK fixture + test: same wrapped three-component header under
      `tests/java/blank_lines_between_record_components/`; goldens at the
      absent default `0` (no blank lines) and `1` (one bare blank line between
      components); new `tests/options/blank_lines_between_record_components.rs`;
      wire in `tests/options.rs` (AC4).
- [x] Run `cargo test`; confirm the suite is green, every existing golden is
      byte-identical, and each new golden formats to itself on a second pass
      (AC5; R6).
- [x] Update docs: flip the three ❌ Records rows (`RPAREN_ON_NEW_LINE_IN_RECORD_HEADER`,
      `SPACE_WITHIN_RECORD_HEADER`, `ANNOTATION_NEW_LINE_IN_RECORD_COMPONENT`)
      and the ❌ `BLANK_LINES_BETWEEN_RECORD_COMPONENTS` row in
      `docs/settings/java.md` to ✅; add the four options to the README
      honoured-options table plus a record-header formatting-behaviour bullet;
      add a record-header-layout requirement row (R33 — the next free number
      after R32 at the time of shipping) to `docs/requirements.md` and note it
      in the milestones paragraph; append the `docs/dev/changelog.md` entry
      (recording that no IntelliJ install was available to cross-check the
      goldens); the shipped state is green (`cargo test --workspace`, 655
      tests).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
