---
type: ChangeRequest
kind: feature
title: Honour the type-argument and type-parameter spacing options
description: Make generic spacing configurable via the four angle-bracket spacing options.
state: planned
priority: low
tags: [dev, formatter]
owner: maintainer
---

# Problem

The "Type arguments and parameters" table in `docs/settings/java.md` marks `SPACES_WITHIN_ANGLE_BRACKETS`, `SPACE_AFTER_CLOSING_ANGLE_BRACKET_IN_TYPE_ARGUMENT`, `SPACE_BEFORE_OPENING_ANGLE_BRACKET_IN_TYPE_PARAMETER` and `SPACE_AROUND_TYPE_BOUNDS_IN_TYPE_PARAMETERS` all ❌. Since R14 (generic-type-argument-spacing) the `flat_type` machinery in `crates/core/src/formatter.rs` renders generic type sites in IntelliJ's canonical form — no space inside the angle brackets, one space after commas, single spaces around `extends` / `&` bounds (README notes) — so generic spacing is fixed rather than scheme-configurable: schemes that set any of these four bools are only partially honoured (safely ignored, R7).

# Proposal

Parse the four options into `config::JavaStyle` via the `OPTIONS` registry in `crates/core/src/config.rs`, with the IntelliJ built-in defaults from the table (all `false` except `SPACE_AROUND_TYPE_BOUNDS_IN_TYPE_PARAMETERS`, default `true`), and make the R14 canonical renderers honour them: `flat_type_args` / `flat_type_params` pad inside the angle brackets when `SPACES_WITHIN_ANGLE_BRACKETS` is set (`< T >`), the renderers add the configurable space after a closing `>` in type-argument position and before a type-parameter list's `<` per the two bracket options, and `flat_type_param` / `flat_type_bound` drop the surrounding bound spacing when the bounds option is `false`. Only whitespace decisions change; R14's per-kind structure (nested generics, wildcards, annotated types, arrays) and its verbatim fallback are preserved.

Docs touched: on delivery the implementation flips the four rows in `docs/settings/java.md` (❌ → ✅), updates the README generic-spacing note (canonical by default, configurable per these options), adds a new requirement row to `docs/requirements.md`, and appends `docs/dev/changelog.md`.

# Decisions

- **Defaults reproduce today's canonical output.** With the defaults above an absent option renders exactly as R14 ships (`<T>`, `T extends A & B`), so default-scheme output is byte-identical and existing goldens stay green.
- **Parameterise, don't fork.** The bools are flags on the existing `flat_type` / `flat_type_args` / `flat_type_params` / `flat_type_bound` join points, not a second type renderer.
- **Whitespace only (R5).** Only inter-token space is added or removed; a mandatory separating space is kept wherever two tokens would otherwise merge (e.g. after `extends`), and unmodelled type shapes still echo verbatim (R4). Exact IntelliJ preview semantics of each option are pinned against real IntelliJ output at plan time.
- **One family, one request.** Unlisted generic-spacing options stay unimplemented and are ignored safely (R7).

# Acceptance criteria

- Golden fixture + test file per option under `crates/core/tests/options/` at both bool values: `SPACES_WITHIN_ANGLE_BRACKETS` = `true` pads `< T >` in type arguments and parameters, the bracket-space options add their space, and `SPACE_AROUND_TYPE_BOUNDS_IN_TYPE_PARAMETERS` = `false` compresses bound spacing — each with a matching `*.out.java` golden.
- Absent-option and default schemes produce the current canonical output unchanged and the whole suite stays green (`cargo test`); the new goldens are idempotent.
- The four `docs/settings/java.md` rows flip to ✅ and the README generic-spacing note is updated.

# Implementation plan

## Approach

Two sides, as with the sibling spacing requests: configuration in
`src/config.rs` and rendering in `src/formatter.rs`, with the exact IntelliJ
semantics pinned first (Decision 3).

**Pin the semantics (Decision 3).** Before wiring anything, verify the four
options against real IntelliJ — export a scheme with each option toggled and
reformat a probe snippet — to pin (a) the XML option names, (b) which scheme
block they serialize into, and (c) the precise rendered effect at each join
site below. The docs-table descriptions are the assumed semantics if IntelliJ
is unavailable: `< T >` for `SPACES_WITHIN_ANGLE_BRACKETS`, a space after the
closing `>` in type-argument position for
`SPACE_AFTER_CLOSING_ANGLE_BRACKET_IN_TYPE_ARGUMENT`, a space before a
type-parameter list's `<` for
`SPACE_BEFORE_OPENING_ANGLE_BRACKET_IN_TYPE_PARAMETER`, and bound spacing
`T extends A & B` for `SPACE_AROUND_TYPE_BOUNDS_IN_TYPE_PARAMETERS` — the
assumption is recorded in the changelog.

**Configuration (src/config.rs).** Add four `bool` fields to `JavaStyle` in a
new `// --- generic type spacing ---` group —
`spaces_within_angle_brackets`, `space_after_closing_angle_bracket_in_type_argument`
and `space_before_opening_angle_bracket_in_type_parameter` → `false`,
`space_around_type_bounds_in_type_parameters` → `true` (the IntelliJ built-in
defaults from the table). `JavaStyle` is constructed only via `Default`, so no
literal-site changes are needed. Add one `OptionDef` per option to the
`OPTIONS` registry (group `"Spaces"`, `OptionValue::Bool`;
`Section::JavaCodeStyle` — the java.md page documents the
`<JavaCodeStyleSettings>` block — adjusted per the pin). Parsing and
serialization are registry-driven, so `parse_codestyle` /
`serialize_codestyle` need no changes (absent → default falls out of
`OptionMap::get_bool`), and the GUI renders the new entries automatically as
checkboxes.

**Rendering (src/formatter.rs).** The bools are flags on the existing R14
join points, not a second type renderer (Decision 2). `flat_type_args` /
`flat_type_params` (L2655 / L2684) pad inside the angle brackets when
`spaces_within_angle_brackets` is set (`< T >`); because both renderers
recurse through `flat_type`, nested generics get the padding at every level.
The closing-`>` space threads through the type-argument join sites where a
rendered `<…>` directly abuts a following token — the invocation/`new`
explicit-type-argument joins (`flat_inv` L2084, `inv_wrapped` L2111,
`fmt_chain` L2223, `new_expr` L2256, `flat_new` L2896: `a.<T>b()` →
`a.<T> b()`) and, per the pin, the nested `>`-in-`>` join (`List<List<String>>`
→ `List<List<String> >`). The type-parameter `<` gap goes in `class_decl` /
`iface_decl` / `record_decl` where the name directly abuts
`flat_type_params` (L474 / L511 / L613: `class Foo<T>` → `class Foo <T>`);
generic method/constructor lists already follow the modifiers' space
(L778 / L829) and are left alone unless the pin says otherwise.
`flat_type_param` / `flat_type_bound` (L2694 / L2708) drop the optional bound
spacing when `space_around_type_bounds_in_type_parameters` is `false` — the
`&` join loses its spaces (`T extends A & B` → `T extends A&B`) while the
mandatory separating spaces around `extends` stay (Decision 3: a separating
space is kept wherever two tokens would otherwise merge). Only whitespace
changes (R5); unmodelled shapes still echo verbatim (R4); the defaults
reproduce R14's canonical output byte-for-byte, so existing goldens stay
green.

**Coordination with the sibling request.** `spaces-around-separators.md`
(planned) claims `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` (the comma join inside
`flat_type_args`) and `SPACE_BEFORE_TYPE_PARAMETER_LIST` (described in the
common table with the same "space between a class / method name and its
type-parameter list" effect as this request's
`SPACE_BEFORE_OPENING_ANGLE_BRACKET_IN_TYPE_PARAMETER`). Both are out of scope
here: this request wires only its four angle-bracket options and leaves the
`flat_type_args` comma join untouched. If the step-1 pin (or the
implementation order) shows `SPACE_BEFORE_TYPE_PARAMETER_LIST` and
`SPACE_BEFORE_OPENING_ANGLE_BRACKET_IN_TYPE_PARAMETER` to be the same IntelliJ
property at the same join, coordinate with the sibling request so a single
join honours both fields (both default `false`, so canonical output is
unaffected either way).

**Tests (hard rules from AGENTS.md).** Four new option files under
`crates/core/tests/options/<xml_option>.rs` (lower-snake of the XML name:
`spaces_within_angle_brackets.rs`,
`space_after_closing_angle_bracket_in_type_argument.rs`,
`space_before_opening_angle_bracket_in_type_parameter.rs`,
`space_around_type_bounds_in_type_parameters.rs`), each starting
`use super::common::*;`, wired in `tests/options.rs` via
`#[path = "options/<name>.rs"] mod <name>;`, with fixtures under
`tests/java/<option>/` embedded through relative `include_str!`
(`../java/<option>/<scenario>.java`) — input `x.java` + byte-exact
`x.out.java`. Each file holds goldens at both bool values (the toggle away
from the default and the default itself): `< T >` vs `<T>`, the after-`>`
space on/off, `class Foo <T>` vs `class Foo<T>`, and `T extends A&B` vs
`T extends A & B`, plus the absent-option default via the default style
asserting today's canonical output is unchanged. No inline Java strings, no
new helpers, no `parse_codestyle` tests. Single-space insertions/removals are
idempotent by construction (R6); verify by formatting each golden with its
own style during development.

**Docs.** The four `docs/settings/java.md` "Type arguments and parameters"
rows flip ❌ → ✅; the README honoured-options table gains the four options
and the generic-spacing formatting note is updated (canonical by default,
configurable per these options); `docs/requirements.md` gains a new
requirement row; `docs/dev/changelog.md` is appended.

## Steps

- [ ] Pin the four options' exact IntelliJ semantics (XML names, scheme
      block, rendered effect at each join site) against a real exported
      scheme / preview; record the before/after pairs the goldens will
      assert; if IntelliJ is unavailable, implement per the docs-table
      descriptions and note the assumption in the changelog (Decision 3;
      shapes the AC1 goldens).
- [ ] config.rs: add the four `bool` fields to `JavaStyle` with the table
      defaults (`false`, `false`, `false`, `true`) and the four `OptionDef`
      entries (group "Spaces", `OptionValue::Bool`, `Section::JavaCodeStyle`
      pending the pin) (AC: absent → default mapping).
- [ ] formatter.rs: `flat_type_args` / `flat_type_params` pad inside the
      angle brackets per `spaces_within_angle_brackets` (`< T >`); default
      keeps `<T>` byte-identical (AC1 for `SPACES_WITHIN_ANGLE_BRACKETS`).
- [ ] formatter.rs: insert the optional space after a closing `>` in
      type-argument position per
      `space_after_closing_angle_bracket_in_type_argument` at the
      invocation/`new` joins (`flat_inv`, `inv_wrapped`, `fmt_chain`,
      `new_expr`, `flat_new`) and the nested `>`-in-`>` join per the pin
      (AC1 for `SPACE_AFTER_CLOSING_ANGLE_BRACKET_IN_TYPE_ARGUMENT`).
- [ ] formatter.rs: optional space before a type-parameter list's `<` per
      `space_before_opening_angle_bracket_in_type_parameter` at the
      `class_decl` / `iface_decl` / `record_decl` name→`<…>` joins
      (methods / constructors only if the pin says so); coordinate with the
      sibling CR's `SPACE_BEFORE_TYPE_PARAMETER_LIST` if it proves to be the
      same join (AC1 for `SPACE_BEFORE_OPENING_ANGLE_BRACKET_IN_TYPE_PARAMETER`).
- [ ] formatter.rs: `flat_type_param` / `flat_type_bound` drop the optional
      bound spacing when `space_around_type_bounds_in_type_parameters` is
      `false` (`T extends A & B` → `T extends A&B`), keeping the mandatory
      spaces around `extends` (AC1 for
      `SPACE_AROUND_TYPE_BOUNDS_IN_TYPE_PARAMETERS`).
- [ ] Tests: create the four option files under `crates/core/tests/options/`,
      wire them in `tests/options.rs`, and add the
      `tests/java/<option>/` fixture + `*.out.java` golden pairs — each
      file asserts the toggle-away-from-default rendering and the
      default/absent-option output (AC1, AC2).
- [ ] Verify: `cargo test` green with no existing golden changed, each new
      golden idempotent under its own style, and `cargo build` for the whole
      workspace (the GUI compiles with the new registry entries) (AC2).
- [ ] Docs: flip the four rows in `docs/settings/java.md` to ✅, update the
      README honoured-options table and the generic-spacing note, add the
      requirement row to `docs/requirements.md`, append
      `docs/dev/changelog.md`, and re-run `cargo test` (AC3).
