---
type: ChangeRequest
kind: feature
title: Layout enum constant lists and enum spacing per the enum options
description: Implement ENUM_CONSTANTS_WRAP and SPACE_INSIDE_ONE_LINE_ENUM_BRACES.
state: proposed
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
