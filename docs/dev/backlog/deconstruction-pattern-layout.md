---
type: ChangeRequest
kind: feature
title: Honour the deconstruction-pattern layout options (Java 21)
description: Implement wrapping, alignment and spacing for record deconstruction patterns.
state: proposed
priority: low
tags: [dev, formatter]
owner: maintainer
---

# Problem

The "Deconstruction patterns (Java 21)" table in `docs/settings/java.md` — `DECONSTRUCTION_LIST_WRAP` (wrap-code int), `ALIGN_MULTILINE_DECONSTRUCTION_LIST_COMPONENTS`, `NEW_LINE_AFTER_LPAREN_IN_DECONSTRUCTION_PATTERN`, `RPAREN_ON_NEW_LINE_IN_DECONSTRUCTION_PATTERN`, `SPACE_WITHIN_DECONSTRUCTION_LIST`, `SPACE_BEFORE_DECONSTRUCTION_LIST` — is all ❌. Record patterns such as `case A(int x, String s) -> …` appear as pattern-matching `switch` labels; java-formatter formats switch bodies (shipped switch-formatting) but the label is echoed verbatim (`self.txt` on the `switch_label` in `crates/core/src/formatter.rs`), so the pattern list keeps its source layout and none of the six list-layout options apply — schemes setting them are only partially honoured (safely ignored, R7).

# Proposal

Parse the six options into `config::JavaStyle` via the `OPTIONS` registry in `crates/core/src/config.rs` (`DECONSTRUCTION_LIST_WRAP` as a wrap-code int reusing the `WrapStyle` mapping; the bools with their table defaults) and model record-pattern labels in `crates/core/src/formatter.rs` on the shipped record-component header template (`record_components` honours `RECORD_COMPONENTS_WRAP`, `ALIGN_MULTILINE_RECORDS`, `NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER`): wrap the deconstruction list per `DECONSTRUCTION_LIST_WRAP` with components aligned per `ALIGN_MULTILINE_DECONSTRUCTION_LIST_COMPONENTS`, put `(` / `)` on their own lines per the two paren options, and apply `SPACE_WITHIN_DECONSTRUCTION_LIST` (`case A( int x )`) and `SPACE_BEFORE_DECONSTRUCTION_LIST` (`case A (int x)`) around the rendered list. Any pattern shape outside the modelled form falls back to the verbatim echo (R4).

Docs touched: on delivery the implementation flips the six rows in `docs/settings/java.md` (❌ → ✅), updates the README formatting-behaviour notes for switch/pattern labels and the honoured-options table, adds a new requirement row to `docs/requirements.md`, and appends `docs/dev/changelog.md`.

# Decisions

- **The record header is the structural template.** The deconstruction list is a pattern-list analogue of the shipped record-component header: same wrap/alignment/paren-on-own-line mechanics, with the paren options read from this family's own defaults (both default `true` here) and `RPAREN_ON_NEW_LINE` carried by this request even though its record-header counterpart is not yet shipped.
- **Defaults.** Option entries carry the table's IntelliJ defaults; as with the recorded `RECORD_COMPONENTS_WRAP` divergence, `JavaStyle::default()` ships `DECONSTRUCTION_LIST_WRAP` = `DoNotWrap` so an absent option keeps today's single-line labels byte-identical (existing goldens stay green); the default-`true` align/paren options only matter once a wrap engages, and the spacing bools default `false` (today's compact `case A(int x)` form).
- **Modelled like records, verbatim otherwise.** Once modelled, pattern labels are laid out like other modelled constructs (records, switch); unmodelled patterns still echo verbatim (R4) and only whitespace/layout changes (R5).
- **One family, one request.** Unlisted switch/pattern options (e.g. other pattern forms) stay unimplemented and are ignored safely (R7).

# Acceptance criteria

- Golden fixture + test file per option under `crates/core/tests/options/`: an over-margin record pattern in a `case` label wraps per `DECONSTRUCTION_LIST_WRAP` codes `0`/`1`/`2`/`5`, with components aligned under `ALIGN_MULTILINE_DECONSTRUCTION_LIST_COMPONENTS`, `(` / `)` on their own lines under the two paren options (both `true`), and `SPACE_WITHIN_DECONSTRUCTION_LIST` / `SPACE_BEFORE_DECONSTRUCTION_LIST` = `true` fixtures; the absent-option compact form is asserted unchanged.
- Pattern labels that do not fit the modelled record-pattern shape still round-trip verbatim.
- Absent-option and default schemes keep today's output and the whole suite stays green (`cargo test`); the new goldens are idempotent.
- The six `docs/settings/java.md` rows flip to ✅ and the README switch/pattern notes are updated.
