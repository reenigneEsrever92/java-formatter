---
type: ChangeRequest
kind: feature
title: Honour the annotation placement and annotation-body layout options
description: Implement annotation-on-separate-line placement and annotation parameter wrapping/alignment options.
state: done
verified: { by: maintainer, at: 2026-09-05T15:30Z }
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

The annotation rows are ❌ in both settings docs — the placement options `METHOD_ANNOTATION_WRAP`, `CLASS_ANNOTATION_WRAP`, `FIELD_ANNOTATION_WRAP`, `PARAMETER_ANNOTATION_WRAP` and `VARIABLE_ANNOTATION_WRAP` in docs/settings/common.md "Annotations", and `ENUM_FIELD_ANNOTATION_WRAP`, `ALIGN_MULTILINE_ANNOTATION_PARAMETERS`, `NEW_LINE_AFTER_LPAREN_IN_ANNOTATION`, `RPAREN_ON_NEW_LINE_IN_ANNOTATION`, `SPACE_AROUND_ANNOTATION_EQ`, `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION` and `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION_IN_PARAMETER` in docs/settings/java.md "Annotations" — valid IntelliJ options java-formatter neither parses nor applies, so schemes setting them are only partially honoured and the options are ignored (R7). Today annotations are rendered inline with their declaration (the `modifiers` renderer joins them with spaces) regardless of the placement wrap code, and annotation argument lists are laid out without alignment, `(`/`)` placement, `=` spacing or single-annotation exemptions — `ANNOTATION_PARAMETER_WRAP` (argument-list wrapping) already ships and is the model for the Java-block options.

# Proposal

Parse the common-block placement options (`Section::CodeStyleJava`, built-in defaults `2`/`2`/`2`/`0`/`0` per the tables — wrap-always for methods, classes and fields — absent → default, reusing the existing `WrapStyle` mapping) and the Java-block options (`Section::JavaCodeStyle`, defaults `0`/`false`/`false`/`false`/`true`/`false`/`false`) into `JavaStyle` via the OPTIONS registry in crates/core/src/config.rs — `OptionDef` entries with `OptionValue::Wrap` or `OptionValue::Bool` accordingly. Apply them in crates/core/src/formatter.rs: place each annotation on its own line before the declaration per the placement wrap code, lay out annotation arguments per the Java-block toggles (alignment, `(`/`)` on their own lines, spaces around `=`), put enum-field annotations on their own lines per `ENUM_FIELD_ANNOTATION_WRAP`, and exempt a lone annotation from the line break per the two `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION` options.

Docs touched: `docs/settings/common.md` and `docs/settings/java.md` (❌ → ✅ for these rows), `README.md` (honoured-options table rows and formatting-behaviour notes), `docs/requirements.md` (a new requirement row) and `docs/dev/changelog.md` on completion.

# Decisions

1. **One family, one request.** Only the listed placement and body-layout options are added; other annotation/enum options still ❌ stay unimplemented and are ignored safely (R7).
2. **Defaults — a deliberate default-behaviour change.** IntelliJ's built-in defaults include wrap-always (`2`) for `METHOD_ANNOTATION_WRAP`, `CLASS_ANNOTATION_WRAP` and `FIELD_ANNOTATION_WRAP`, so default/absent schemes move those annotations onto their own lines; goldens touching annotated declarations are updated once in this CR, and all other output stays byte-identical (existing goldens otherwise stay green).
3. **Semantics — placement relocates layout tokens only.** R5: putting an annotation on its own line, or moving `(`/`)`/`=` in an argument list, inserts or relocates only whitespace and newlines around existing tokens, never changing the annotation or its declaration; unmodelled shapes stay verbatim (R4); updated goldens pin R6 idempotency.
4. **Encodings.** The `*_ANNOTATION_WRAP` placement options use the wrap codes `0`/`1`/`2`/`5` shared across the `*_WRAP` options; the Java-block options are plain bools, and the `(`/`)`-on-new-line bools affect only wrapped annotation argument lists.

# Acceptance criteria

- Golden fixture + test file per option under crates/core/tests/options/ at the interesting values (wrap codes `0`/`1`/`2`/`5` for the placement options, both bool states) plus an absent-option default case.
- Annotated methods, classes, fields, parameters, local variables and enum fields place annotations on separate lines per their wrap code; annotation arguments align / place `(`/`)` / space `=` per the Java-block toggles; single-annotation no-wrap exemptions are honoured.
- Whole suite green (`cargo test`); goldens changed by the built-in wrap-always defaults are updated deliberately and re-formatting them is a no-op (R6).
- `docs/settings` marks flipped (❌ → ✅); README honoured-options table / behaviour notes and `docs/requirements.md` updated.
- New goldens idempotent: formatting the output again is a no-op.

# Implementation plan

## Approach

Configuration and rendering, as with the sibling option-family requests
(`binary-expression-wrapping`, `blank-line-policy`).

**Configuration — `crates/core/src/config.rs`.** `JavaStyle` is constructed
only via `Default` (no other struct literals exist), so add twelve fields with
`Default` values and one `OptionDef` per field; `parse_codestyle` /
`serialize_codestyle` both iterate the `OPTIONS` registry, so both directions
come for free and absent-from-scheme options keep the field default (R7). No
GUI change is needed — the GUI renders the registry.

- Five common-block placement options → `Section::CodeStyleJava`, typed
  `OptionValue::Wrap` (reuse `WrapStyle::from_int`), defaults from the
  docs/settings/common.md "Annotations" table: `METHOD_ANNOTATION_WRAP`,
  `CLASS_ANNOTATION_WRAP`, `FIELD_ANNOTATION_WRAP` = `WrapAlways` (`2`),
  `PARAMETER_ANNOTATION_WRAP`, `VARIABLE_ANNOTATION_WRAP` = `DoNotWrap` (`0`).
- Seven Java-specific options → `Section::JavaCodeStyle`, defaults from the
  docs/settings/java.md "Annotations" table: `ENUM_FIELD_ANNOTATION_WRAP` =
  `OptionValue::Wrap` `DoNotWrap` (`0`); `ALIGN_MULTILINE_ANNOTATION_PARAMETERS`
  = `false`; `NEW_LINE_AFTER_LPAREN_IN_ANNOTATION` = `false`;
  `RPAREN_ON_NEW_LINE_IN_ANNOTATION` = `false`; `SPACE_AROUND_ANNOTATION_EQ` =
  `true`; `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION` = `false`;
  `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION_IN_PARAMETER` = `false`.

The registry `default` must equal the field default so the
`parse(serialize(style)) == style` round trip stays exact. Group the new defs
next to their kin: the JavaCodeStyle ones immediately after
`ANNOTATION_PARAMETER_WRAP` (config.rs L501-513, group "Records &
annotations") and the CodeStyleJava ones after the other `*_WRAP` rows (group
"Wrapping"). Every default is an IntelliJ built-in, so no divergence note is
needed (unlike `RECORD_COMPONENTS_WRAP`, whose divergence is documented in
docs/settings/java.md).

**Placement rendering — `crates/core/src/formatter.rs`.** Today each
annotation site hard-codes one of two layouts:

- `modifiers()` (L979-1007) — used by every member/type declaration
  (`class_decl`, `iface_decl`, `enum_decl`, `record_decl`, `method_decl`,
  `constructor_decl`, `compact_constructor_decl`, `field_decl`) — always
  emits each annotation on its own line followed by `\n` + `ind(indent)`, then
  the keyword modifiers. That is exactly the wrap-always (`2`) layout, so the
  method/class/field defaults already match IntelliJ and the new option merely
  makes the layout configurable.
- `flat_mods()` (L1231-1245) — used by parameters (`flat_param` L1195-1215)
  and local variables (`local_var` L1428-1441) — joins annotations and
  keywords inline with single spaces, exactly the do-not-wrap (`0`) layout
  that matches the parameter/variable defaults.
- enum constants are echoed verbatim (`enum_body` L566-574), annotations
  included, so `ENUM_FIELD_ANNOTATION_WRAP` needs a small renderer (below).

Wrap-code semantics at every placement site (shared codes `0`/`1`/`2`/`5`):

- `DoNotWrap`: annotations joined with single spaces on the declaration's
  first line, before the keyword modifiers / type / name — `@Deprecated
  public void run()` — keeping the codebase's canonical
  annotations-before-modifiers order.
- `WrapAlways`: one annotation per line above the declaration (today's
  `modifiers()` shape — `@Deprecated\npublic void run()`).
- `WrapIfLong` / `ChopDownIfLong`: keep the inline form unless the composed
  first line — annotations + keywords + type + name + parameters/declarators
  rendered with the existing flat helpers — exceeds the margin (`fits` /
  `col_after` at the declaration's start column), then fall back to one
  annotation per line. The two codes behave identically at this granularity (a
  declaration's annotation list has no sub-items to chop down).

The two single-annotation exemptions override the wrap decision when the
declaration or parameter carries exactly one annotation node:
`DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION` (the member/type/local-variable
placement sites, matching its docs row) and
`DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION_IN_PARAMETER` (parameters): the lone
annotation stays inline regardless of the wrap code.

Implementation direction: give `modifiers()` the governing `WrapStyle` (each
caller passes its option: the four type declarations →
`class_annotation_wrap`; methods + constructors + compact constructors →
`method_annotation_wrap`; fields → `field_annotation_wrap`) plus the caller's
inline non-modifier header tail so codes `1`/`5` can measure the composed
first line; the helper returns either the inline or the one-per-line form and
the callers keep their existing `ends_with(' ' / '\n')` space handling. Local
variables switch from `flat_mods` to the same decision driven by
`variable_annotation_wrap` inside `local_var`. Parameters honour
`parameter_annotation_wrap` in the per-line parameter layout produced by
`formal_params` / `flat_param` (annotation break so the parameter type/name
continues on the next line); when the enclosing list is rendered on one line
the own-line placement is not expressible, so the wrap demand should also take
the list to its wrapped one-parameter-per-line layout — the exact choice is
verified against IntelliJ when one is available and otherwise pinned by the
`parameter_annotation_wrap` goldens. Record components and receiver
parameters stay inline (records have their own unimplemented
`ANNOTATION_NEW_LINE_IN_RECORD_COMPONENT`). Because every placement default
equals today's hard-coded layout, existing goldens stay byte-identical under
the default style.

**Enum constants.** `ENUM_FIELD_ANNOTATION_WRAP` cannot be honoured through
the verbatim `self.txt(child)` echo. When an `enum_constant` carries
annotation children (confirm whether tree-sitter-java models them as direct
`annotation` / `marker_annotation` children of the constant or wrapped in a
`modifiers` child — `get_mods` is the existing probe), render the annotation
prefix per the wrap code (default `DoNotWrap` → inline `@A A`) and echo the
remainder of the constant — name, `(arguments)` and any constant class `body`
— verbatim from the source bytes (R4/R5). Constants without annotations keep
the current verbatim echo, so unannotated enum output is untouched.

**Argument-body layout.** The four layout toggles govern only the *wrapped*
argument list produced by `annotation()` → `annotation_expanded()`
(L1011-1139), which today hard-codes one stacked shape — `@Name(` then each
argument on its own line at `ind(indent + 1)`, then `)` alone on its own line
at the annotation's indent — for both the single-pair-with-array branches
(L1090-1126) and the multi-argument branch (L1128-1138). `flat` contexts never
expand (`flat_annotation` L2887-2894 always renders flat). Follow the
record-header model (`record_components` L642-691 — the closest implemented
analogue, same option family and naming):

- `NEW_LINE_AFTER_LPAREN_IN_ANNOTATION` (default `false`): `false` keeps the
  first argument on the `(` line (later arguments on their own lines); `true`
  starts every argument on its own line after the `(`.
- `RPAREN_ON_NEW_LINE_IN_ANNOTATION` (default `false`): `false` places `)`
  directly after the last argument's line; `true` places it on its own line at
  the annotation's indent.
- `ALIGN_MULTILINE_ANNOTATION_PARAMETERS` (default `false`): `false` indents
  wrapped argument lines with `cont(indent)`; `true` aligns them under the
  first argument, one column after the `(` (the `open_col + 1` treatment
  `record_components` already uses).
- `SPACE_AROUND_ANNOTATION_EQ` (default `true`): `true` keeps today's
  `key = value`; `false` emits `key=value`. Apply wherever an
  `element_value_pair` is rendered — `flat_ann_arg` (L1071-1084, shared by
  `annotation()` and `flat_annotation()`) and the expanded single-pair branch
  (L1090-1109).

Today's stacked shape is the all-toggles-true corner; the IntelliJ defaults
(`false`/`false`/`false`) reshape the expanded multi-argument layout (first
argument joins the `(` line, `)` attaches to the last argument). That is a
deliberate default-behaviour change: the four `annotation_parameter_wrap`
goldens under `tests/java/annotation_parameter_wrap/`, which pin the old
hard-coded shape, are regenerated once in this CR and re-formatting them must
be a no-op (R6); every other existing golden stays byte-identical. Verify the
reshaped goldens against a real IntelliJ install when one is available to the
implementer, as the sibling requests did, and record the outcome in the
changelog.

**Tests.** Follow the hard testing conventions of `.agents/AGENTS.md`: one
golden-pair test module per option in `crates/core/tests/options/` named after
the XML option, wired in `tests/options.rs` via `#[path]`, starting with `use
super::common::*;`, doc header `//! <OPTION> — …` plus a fixture-path comment,
fixtures under `crates/core/tests/java/<option>/` reached by relative
`include_str!` with a shared input/golden stem, no inline Java strings and no
`parse_codestyle` tests. New goldens are checked idempotent with a second
manual format pass during development (no `assert_idempotent` helper exists).
Interesting values: the five placement options and `ENUM_FIELD_ANNOTATION_WRAP`
at wrap codes `0`/`1`/`2`/`5` (codes `1`/`5` exercised at a narrow
`right_margin` so the wrap decision fires), the six bools at both states, each
plus an absent-option check (`format(fixture)` against a
`<stem>_default.out.java` golden) pinning the IntelliJ default. Twelve option
files result; the argument-list fixtures must force the wrapped layout
(`annotation_parameter_wrap` or a narrow margin) for the paren/alignment
tests, and the exemption fixtures use declarations/parameters carrying exactly
one annotation under a wrap-always placement option.

## Steps

- [x] `crates/core/src/config.rs`: add the twelve fields to `JavaStyle` with
      the `Default` values above (five placement `WrapStyle`s plus
      `enum_field_annotation_wrap` under an annotation banner, the six bools
      under a Java-specific banner), then the twelve `OptionDef`s (sections
      and groups per the approach; `OptionValue::Wrap` for the wrap defs,
      `OptionValue::Bool` for the bools). `cargo build` and the suite stay
      green (AC1 config mapping; R7 absent → default).
- [x] Placement engine for member/type declarations: parameterise
      `modifiers()` by the governing wrap option + single-annotation
      exemption and return either the inline or the one-per-line form; wire
      each caller (`class_decl`/`iface_decl`/`enum_decl`/`record_decl` →
      `class_annotation_wrap`; `method_decl`/`constructor_decl`/
      `compact_constructor_decl` → `method_annotation_wrap`; `field_decl` →
      `field_annotation_wrap`), measuring codes `1`/`5` against the composed
      inline header. Confirm the default style still emits today's own-line
      layout (AC2).
- [x] Parameters and local variables: give `formal_params`/`flat_param` the
      `parameter_annotation_wrap` +
      `do_not_wrap_after_single_annotation_in_parameter` behaviour (own-line
      annotation placement in the per-line parameter layout) and `local_var`
      the `variable_annotation_wrap` +
      `do_not_wrap_after_single_annotation` behaviour; keep record components
      and receiver parameters inline (AC2).
- [x] Enum fields: when an `enum_constant` has annotations, render them per
      `enum_field_annotation_wrap` and echo the rest of the constant verbatim;
      otherwise keep today's verbatim echo (AC2; R4/R5).
- [x] Argument-body layout: extend `annotation_expanded()` (multi-argument
      branch) with `align_multiline_annotation_parameters`,
      `new_line_after_lparen_in_annotation` and
      `rparen_on_new_line_in_annotation` per the approach, and route
      `space_around_annotation_eq` through `flat_ann_arg` and the expanded
      single-pair branch so every `element_value_pair` honours it (AC2).
- [x] Add fixtures + golden-pair tests for the six placement-related options
      (`tests/options/method_annotation_wrap.rs`, `class_annotation_wrap.rs`,
      `field_annotation_wrap.rs`, `parameter_annotation_wrap.rs`,
      `variable_annotation_wrap.rs`, `enum_field_annotation_wrap.rs`): inputs
      mixing inline and already-wrapped annotations, single and multiple
      annotations, with and without keyword modifiers; goldens at codes
      `0`/`1`/`2`/`5` (narrow `right_margin` for `1`/`5`) plus an
      absent-option default golden; wire the modules in `tests/options.rs`
      (AC1, AC2).
- [x] Add fixtures + golden-pair tests for the six Java-block bool options
      (`tests/options/align_multiline_annotation_parameters.rs`,
      `new_line_after_lparen_in_annotation.rs`,
      `rparen_on_new_line_in_annotation.rs`, `space_around_annotation_eq.rs`,
      `do_not_wrap_after_single_annotation.rs`,
      `do_not_wrap_after_single_annotation_in_parameter.rs`): the
      paren/alignment fixtures force the wrapped argument layout and toggle
      each bool both ways; the exemption fixtures carry exactly one annotation
      under a wrap-always placement option; each file ends with an
      absent-option default golden (AC1, AC2).
- [x] Run `cargo test`; regenerate the goldens that encode the pre-option
      engine — the `annotation_parameter_wrap` goldens whose expanded layout
      moves to the option-default shape (first argument on the `(` line, `)`
      attached) — and confirm every other golden is byte-identical and every
      changed/new golden formats to itself on a second pass (AC3, AC5; R6).
- [x] Update docs: flip ❌ → ✅ for the five rows in the "Annotations" table of
      docs/settings/common.md and the seven rows in the "Annotations" table of
      docs/settings/java.md; add the twelve options to the README
      honoured-options table plus a formatting-behaviour note on annotation
      placement and wrapped argument layout; add a new annotation-layout
      requirement row (R16) to docs/requirements.md and extend the milestone
      paragraph; append a changelog entry to docs/dev/changelog.md (recording
      whether an IntelliJ install was available to cross-check the goldens);
      run `cargo test` once more to confirm the shipped state is green (AC4,
      AC3).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
