---
type: ChangeRequest
kind: feature
title: Layout enum constant lists and enum spacing per the enum options
description: Implement ENUM_CONSTANTS_WRAP and SPACE_INSIDE_ONE_LINE_ENUM_BRACES.
state: planned
priority: low
tags: [dev, formatter]
owner: maintainer
---

# Problem

Enum bodies are emitted without any wrap handling: `ENUM_CONSTANTS_WRAP` (docs/settings/common.md "Enums" — a wrap-code int, default `0`) is ❌, so a long constant list is not wrapped per a wrap code even when it exceeds the margin.
One-line `enum E {A, B}` bodies get no inner-padding control either: `SPACE_INSIDE_ONE_LINE_ENUM_BRACES` (docs/settings/java.md "Miscellaneous spacing & blank lines" — bool, default `false`) is ❌ too, so schemes that set either option are only partially honoured — both are safely ignored today (R7).
Annotation-on-enum-constant placement (`ENUM_FIELD_ANNOTATION_WRAP`) is covered by the annotation-layout request, not here.

# Proposal

Add `enum_constants_wrap: WrapStyle` (default `DoNotWrap`) and `space_inside_one_line_enum_braces: bool` (default `false`) to `JavaStyle` via `OptionDef` entries in the `OPTIONS` registry in crates/core/src/config.rs — the wrap-code int parsed with the existing `get_wrap` / `WrapStyle::from_int` mapping, the bool with `get_bool`, absent → default — and apply them in crates/core/src/formatter.rs at enum rendering: a constant list that does not fit is wrapped per the wrap code (0 never, 1 if long, 2 always, 5 chop down), and a one-line body becomes `enum E { A, B }` only when the spacing option is on.

Docs touched: `docs/settings/common.md` "Enums" and `docs/settings/java.md` "Miscellaneous spacing & blank lines" marks flipped ❌→✅, the README honoured-options table and formatting-behaviour notes, `docs/requirements.md` (a new requirement row), and `docs/dev/changelog.md` on delivery.

# Decisions

1. **One family, one request.** Only the two listed options ship; other enum rows (`ENUM_FIELD_ANNOTATION_WRAP` and finer wrap sub-options) stay unimplemented and safely ignored (R7).
2. **Defaults.** Both fields take the IntelliJ built-in defaults from the docs/settings tables (`0`, `false`), so default and absent schemes keep byte-identical current output and the existing goldens stay green.
3. **Semantics.** R5 holds — only whitespace and line breaks are added; the constant list keeps its token order and separators. R4 echoes unmodelled enum shapes verbatim; R6 is pinned by re-formatting the new goldens.
4. **Registry.** Both options map onto existing `OptionValue` variants (a wrap code and a bool) in the current sections (`Section::CodeStyleJava` for `ENUM_CONSTANTS_WRAP`; the Java-specific block for the spacing bool), so no new value variant is needed and the serialize/parse round trip stays exact.

# Acceptance criteria

- `tests/options/enum_constants_wrap.rs` (fixtures under `tests/java/enum_constants_wrap/`) asserts goldens at wrap codes `0`/`1`/`2`/`5` for an enum whose constant list overflows a narrow margin, plus an absent-option default keeping today's single-line layout.
- `tests/options/space_inside_one_line_enum_braces.rs` asserts `{ A, B }` padding when the option is `true` and unchanged `{A, B}` when it is absent/`false`.
- Default-scheme output is unchanged and the whole suite stays green (`cargo test`); the new goldens are idempotent (R6).
- docs/settings marks are flipped ❌→✅; the README, `docs/requirements.md` and `docs/dev/changelog.md` are updated with the implementation.

# Implementation plan

## Approach

Two sides, as in the binary-expression change: configuration and rendering.

**Configuration (crates/core/src/config.rs).** Add two fields to `JavaStyle`
(which is constructed only via `Default`, so no literal-site changes are
needed): `enum_constants_wrap: WrapStyle` (default `WrapStyle::DoNotWrap`) and
`space_inside_one_line_enum_braces: bool` (default `false`), with matching
values in the `Default` impl. Add two `OptionDef` entries to the `OPTIONS`
registry (L232-567): `ENUM_CONSTANTS_WRAP` (`Section::CodeStyleJava`,
`OptionValue::Wrap`, default `0`, parsed via the existing `get_wrap` /
`WrapStyle::from_int` mapping — so 0 never, 1 if long, 2 always, 5 chop down)
and `SPACE_INSIDE_ONE_LINE_ENUM_BRACES` (`Section::JavaCodeStyle` — the
Java-specific block, per Decision 4 — `OptionValue::Bool`, default `false`,
parsed via `get_bool`). Both map onto existing `OptionValue` variants, so
`parse_codestyle` / `serialize_codestyle` and the registry-driven GUI pick
them up without further changes; give both the display group `"Enums"`.

**Rendering (crates/core/src/formatter.rs).** Today `enum_body` (L558-598)
always expands the constant list to one constant per line. Introduce the
one-line form at the `enum_decl` (L531-556) level, following the
flatten-then-fit pattern the other wraps use (`binary` L2377-2419): after the
`header` is built, compute the flat body — the constants (echoed verbatim via
`txt`, R4) joined with `", "`, wrapped as `{A, B}` or, when
`space_inside_one_line_enum_braces` is set, `{ A, B }` — and the flat
declaration `format!("{} {}", header, flat_body)`; then decide with
`c = self.col_after(0, &self.ind(indent))` (the `record_decl` pattern, L617)
and `fits(c, &flat)`:

- `DoNotWrap` (and the absent option / default style) → always the flat form —
  a list that overflows stays on one line, matching IntelliJ's "do not wrap"
  and the codebase's do-not-wrap convention
  (`do_not_wrap_keeps_single_line_even_when_long`);
- `WrapAlways` → always the expanded one-constant-per-line form;
- `WrapIfLong` / `ChopDownIfLong` → flat iff `fits`, else expanded.

The flat form is only produced when the body has at least one `enum_constant`
and no `enum_body_declarations` (members after `;` force the expanded layout);
empty bodies keep today's output. `enum_body` stays as the expanded renderer
and `with_brace` keeps handling header/brace placement per
`class_brace_style`, so `NextLine` styles still put `{` on its own line while
the constants remain joined. `ChopDownIfLong` behaves like `WrapIfLong` at
this stage because constants are echoed verbatim — chopping inside a
constant's own argument list is out of scope (as are
`ENUM_FIELD_ANNOTATION_WRAP` and finer sub-options, Decision 1). R5 holds by
construction: only whitespace and line breaks change; constant order and
separators are preserved.

**Default-layout caveat.** The default scheme now keeps a fitting constant
list on one line (`enum E {A, B}`) where today every enum is expanded — this
is what AC1's "absent-option default keeping today's single-line layout" and
AC2's `{A, B}` pin, and it matches IntelliJ. It cannot regress the existing
suite: no fixture or golden currently contains an enum (verified by grep), so
`cargo test` stays green and the behaviour change shows up only in the new
goldens. Every produced layout is idempotent (R6): the flat form fits and
stays flat, the expanded form does not fit and stays expanded, `DoNotWrap` is
always flat, `WrapAlways` always expanded.

**Tests.** Two new per-option test files and fixtures, per the AGENTS.md hard
rules (golden pairs, `include_str!` relative paths, no inline Java, no
`parse_codestyle` tests, no `assert_idempotent` — idempotency is asserted as
`format_with(GOLDEN, &style) == GOLDEN`):

- `tests/options/enum_constants_wrap.rs` with `tests/java/enum_constants_wrap/`:
  a long one-line constant list that overflows at `right_margin = 40`, with
  goldens for wrap codes 0 (one overflowing line), 1, 2 and 5 (one constant
  per line — 5 equals 1 here, pinned as its own golden), a fitting short enum
  whose absent-option default golden stays one-line (`{RED, GREEN, BLUE}`) and
  whose wrap-always golden breaks (distinguishing code 2 from 1), plus an
  idempotency assertion on the wrap-if-long golden.
- `tests/options/space_inside_one_line_enum_braces.rs` with
  `tests/java/space_inside_one_line_enum_braces/`: `{ A, B }` when `true`,
  unchanged `{A, B}` when absent / `false`, and a guard that the padding does
  not leak into the multi-line layout.
- Wire both modules in `tests/options.rs` (alphabetical: `enum_constants_wrap`
  after `continuation_indent_size`, `space_inside_one_line_enum_braces` after
  `right_margin`).

**Docs.** Flip `ENUM_CONSTANTS_WRAP` ❌→✅ in `docs/settings/common.md`
("Enums", L305-311) and `SPACE_INSIDE_ONE_LINE_ENUM_BRACES` ❌→✅ in
`docs/settings/java.md` ("Miscellaneous spacing & blank lines", L193-206); add
both options to the README honoured-options table with a formatting-behaviour
note (fitting lists stay one-line and are padded per the spacing option;
overflowing lists wrap per the wrap code); add a requirement row (R16) to
`docs/requirements.md` and mention it in the Milestones delivered list; append
the delivery entry to `docs/dev/changelog.md`, recording whether an IntelliJ
cross-check was possible (as the binary-expression entry does).

## Steps

- [ ] config.rs: add `enum_constants_wrap: WrapStyle` and
      `space_inside_one_line_enum_braces: bool` to `JavaStyle` with defaults
      `DoNotWrap` / `false`, and the two `OptionDef` registry entries
      (`ENUM_CONSTANTS_WRAP` in `Section::CodeStyleJava`, wrap variant;
      `SPACE_INSIDE_ONE_LINE_ENUM_BRACES` in `Section::JavaCodeStyle`, bool
      variant; group `"Enums"`); sanity-check that `serialize_codestyle`
      emits the new options in their correct blocks when non-default
      (AC: config mapping).
- [ ] formatter.rs: build the flat one-line enum body in `enum_decl` and
      select flat vs expanded per wrap code + `fits` as described in the
      approach; keep `enum_body` (expanded) and `with_brace` unchanged
      (AC1/AC2 semantics).
- [ ] Add fixtures under tests/java/enum_constants_wrap/: `long_constants.java`
      (overflows at `right_margin = 40`) with wrap0/wrap1/wrap2/wrap5 goldens,
      and `short_enum.java` (fits at the default margin) with a
      default/absent-option one-line golden and a wrap-always golden (AC1).
- [ ] Add tests/options/enum_constants_wrap.rs (doc header,
      `use super::common::*;`, `include_str!` paths): assert each wrap-code
      golden via `style(|s| { s.right_margin = 40; s.enum_constants_wrap = ...; })`,
      the absent-option default via `format`, and idempotency via
      `format_with(wrap1_golden, &narrow(WrapIfLong)) == wrap1_golden`
      (AC1, AC3 idempotency).
- [ ] Add fixtures under tests/java/space_inside_one_line_enum_braces/:
      `padding.java` with a spaces golden (`{ A, B }`) and a no-spaces golden
      (`{A, B}`), plus a long-enum golden pinning that the padding does not
      leak into the multi-line layout (AC2).
- [ ] Add tests/options/space_inside_one_line_enum_braces.rs: spaces golden
      with the option `true`; no-spaces golden with the absent option
      (`format`) and with explicit `false`; the no-padding guard (AC2).
- [ ] Wire both modules into tests/options.rs in alphabetical position
      (`enum_constants_wrap` after `continuation_indent_size`,
      `space_inside_one_line_enum_braces` after `right_margin`)
      (AGENTS.md wiring convention).
- [ ] Run `cargo test`: the whole suite stays green and no existing golden
      changes (only the new fixtures show the new layouts) (AC3).
- [ ] Update the docs: flip the two ❌→✅ marks in docs/settings/common.md and
      docs/settings/java.md, add both options to the README honoured-options
      table plus a formatting-behaviour note, add the R16 row to
      docs/requirements.md (and the Milestones delivered list), and append the
      delivery entry to docs/dev/changelog.md (noting whether an IntelliJ
      cross-check was possible); re-run `cargo test` to confirm the suite is
      still green (AC4, AC3).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
