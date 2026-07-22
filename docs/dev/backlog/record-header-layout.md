---
type: ChangeRequest
kind: feature
title: Honour the remaining record-header layout options
description: Implement the record-component wrapping/annotation/spacing sub-options not yet shipped.
state: proposed
priority: medium
tags: [dev, formatter]
owner: maintainer
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
