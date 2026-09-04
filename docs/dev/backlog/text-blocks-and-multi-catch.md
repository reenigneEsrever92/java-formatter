---
type: ChangeRequest
kind: feature
title: Honour the text-block layout and multi-catch wrapping options
description: Implement the text-block alignment/whitespace options and multi-catch type-list wrapping/alignment.
state: planned
priority: low
tags: [dev, formatter]
owner: maintainer
---

# Problem

The "Text blocks" table (`ALIGN_MULTILINE_TEXT_BLOCKS`, `STRIP_WHITESPACE_FROM_BLANK_LINES_IN_TEXT_BLOCKS`) and the "Multi-catch" table (`MULTI_CATCH_TYPES_WRAP`, `ALIGN_TYPES_IN_MULTI_CATCH`) in `docs/settings/java.md` are all ❌. Text blocks are unmodelled constructs echoed verbatim like comments (R4), and a multi-catch parameter is copied as whitespace-normalised text (`normalise_ws` over the `catch_formal_parameter` in `crates/core/src/formatter.rs`) with no wrap or alignment — so a long `catch (A | B | … e)` never wraps, and schemes setting these four options are only partially honoured (safely ignored, R7).

# Proposal

Parse the four options into `config::JavaStyle` via the `OPTIONS` registry in `crates/core/src/config.rs` (`MULTI_CATCH_TYPES_WRAP` as a wrap-code int through the existing `WrapStyle` mapping; the bools with their table defaults) and apply them in `crates/core/src/formatter.rs`: model text-block nodes so the opening delimiter of a multiline text block is aligned per `ALIGN_MULTILINE_TEXT_BLOCKS` and blank lines inside the content lose their trailing whitespace per `STRIP_WHITESPACE_FROM_BLANK_LINES_IN_TEXT_BLOCKS`, and model the multi-catch type list so `catch (A | B | …)` wraps per `MULTI_CATCH_TYPES_WRAP` (codes `0`/`1`/`2`/`5`) with wrapped types aligned per `ALIGN_TYPES_IN_MULTI_CATCH`, on the record-header layout pattern. Comments and other unmodelled shapes keep today's verbatim echo (R4).

Docs touched: on delivery the implementation flips the four rows in `docs/settings/java.md` (❌ → ✅), updates the README formatting-behaviour notes (text blocks stay verbatim unless the strip option is set) and honoured-options table, adds a new requirement row to `docs/requirements.md`, and appends `docs/dev/changelog.md`.

# Decisions

- **Text blocks stay verbatim unless the scheme opts in.** `ALIGN_MULTILINE_TEXT_BLOCKS` moves only layout whitespace outside the content, which is safe (R5); `STRIP_WHITESPACE_FROM_BLANK_LINES_IN_TEXT_BLOCKS` edits whitespace inside the text-block content — part of the string literal's value — so honouring it is an intentional, opt-in deviation from byte-level preservation. It applies only when the scheme sets the option (default `false`), so today's verbatim echo, R5's whitespace-only rule, and the never-corrupt contract (README / `docs/requirements.md` R4) hold unless the scheme opts in; the strip is additionally limited to whitespace-only blank lines so no visible content is ever touched.
- **Defaults.** The text-block bools default `false`; `MULTI_CATCH_TYPES_WRAP`'s IntelliJ default is `1` (wrap as needed) and `ALIGN_TYPES_IN_MULTI_CATCH` `true`, but — as recorded for `RECORD_COMPONENTS_WRAP` in `docs/settings/java.md` — `JavaStyle::default()` ships `DoNotWrap` so an absent wrap option keeps today's single-line catch layout byte-identical (existing goldens stay green), while alignment only matters once a wrap engages; schemes setting the code explicitly parse identically to IntelliJ.
- **Multi-catch layout only.** Only the union type list of `catch_formal_parameter` is laid out; the parameter name, catch body and unmodelled catch shapes keep today's handling and verbatim echo (R4), and no token is reordered (R5).
- **One family, one request.** Unlisted text-block and catch options stay unimplemented and are ignored safely (R7).

# Acceptance criteria

- Golden fixture + test file per option under `crates/core/tests/options/`: an over-margin `catch (A | B | C …)` wraps per `MULTI_CATCH_TYPES_WRAP` codes `0`/`1`/`2`/`5` with types aligned when `ALIGN_TYPES_IN_MULTI_CATCH` is `true`; `ALIGN_MULTILINE_TEXT_BLOCKS` true/false fixtures; a `STRIP_WHITESPACE_FROM_BLANK_LINES_IN_TEXT_BLOCKS` fixture whose blank lines carry trailing spaces (true strips, false/absent preserves byte-for-byte).
- Absent-option and default schemes keep today's output — text blocks byte-identical, multi-catch single-line — and the whole suite stays green (`cargo test`); the new goldens are idempotent.
- The four `docs/settings/java.md` rows flip to ✅ and the README notes reflect the opt-in strip deviation.

# Implementation plan

## Approach

**Configuration — `crates/core/src/config.rs`.** `JavaStyle` (L105-150) is
constructed only via `Default` (no other struct literals exist), so add four
fields between the record fields (L146) and the imports field (L148), with a
`// --- text blocks & multi-catch (JavaCodeStyleSettings) ---` banner:
`align_multiline_text_blocks: bool` and
`strip_whitespace_from_blank_lines_in_text_blocks: bool` both `false`,
`multi_catch_types_wrap: WrapStyle` = `DoNotWrap`, and
`align_types_in_multi_catch: bool` = `true`. The two text-block bools default
`false` and `ALIGN_TYPES_IN_MULTI_CATCH` defaults `true` exactly as the
docs/settings/java.md tables state; `MULTI_CATCH_TYPES_WRAP` keeps the
`RECORD_COMPONENTS_WRAP` divergence (table default `1`, shipped default
`DoNotWrap`) so an absent wrap option keeps today's single-line catch layout
byte-identical and existing goldens stay green — recorded, not fixed, per the
Records-table precedent (docs/settings/java.md L136-141). Then add four
`OptionDef` entries to the `OPTIONS` registry (L232-567), placed between the
record entries (after `NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER`, L552) and the
imports banner (L553): all `Section::JavaCodeStyle` (these are Java-only
options — their tables live in docs/settings/java.md, and the record options
serialise under `<JavaCodeStyleSettings>` in codestyle.xml; confirm the two
catch options land there too when checking against an IntelliJ export),
grouped `"Text blocks"` then `"Multi-catch"`, each `get`/`set` over the new
field with `OptionDef.default` equal to the field default (registry
invariant). Because `parse_codestyle`, `serialize_codestyle` and the GUI all
iterate `OPTIONS`, parsing, minimal-scheme serialisation and the egui editor
pick the options up with no further wiring. Per `.agents/AGENTS.md` there are
no `parse_codestyle` tests; the mapping is exercised through the per-option
files via `style(...)` and an absent-option case formatted with
`default_style()`.

**Multi-catch rendering — `crates/core/src/formatter.rs`.** Grammar (confirmed
for the pinned tree-sitter-java 0.23.5): `catch_clause` →
`catch_formal_parameter` = optional `modifiers` + `catch_type` +
`_variable_declarator_id`; `catch_type` holds the type alternatives separated
by anonymous `|` tokens (there is no separate `union_type` node), so a
multi-catch is a `catch_type` with more than one type child. Today the
parameter is copied verbatim in the multi-line `try_stmt` path (L1726-1731,
`self.txt`) and whitespace-normalised in `try_one_line` (L1671-1682,
`normalise_ws` over the same text), never wrapped. Replace both with a new
renderer for `catch_formal_parameter` that models the pieces — optional
`modifiers` (`final`), the `catch_type` type children (rendered through the
shipped `flat_type` so multi-catch types get canonical spacing, matching the
R14 generic-spacing normalisation), ` | ` separators, and the parameter name
via the `name` field of the hidden `_variable_declarator_id` — and exposes a
flat (single-line) form plus a wrapped form. Single-type catches flow through
the same renderer but produce today's canonical one-line text (no golden
change for existing `try` fixtures). The multi-line path (`try_stmt`, which
covers plain and try-with-resources statements) wraps on the
`record_components` pattern (L642-691): flat text assembled from the parts;
`DoNotWrap` (and the absent default) always keep the flat form;
`WrapAlways` forces the break; `WrapIfLong` / `ChopDownIfLong` break when the
assembled ` catch (…)` line exceeds the margin at the column where the
parameter starts (the `catch` keyword follows the try/catch body's closing
`}`, so that column is `col_after` over the emitted body text); the first
type stays on the `catch (` line and each following type goes on its own
line. The `|` operator leads each continuation line (the codebase's
operator-placement convention, as shipped for `BINARY_OPERATION_WRAP`). The
continuation prefix is spaces to the first type's column when
`align_types_in_multi_catch` is set (the record `open_col + 1` idea, but for
the catch's `(`), else `self.cont(indent)`; alignment only matters once a
wrap engages, so the absent/default single-line layout is untouched. The
`try_one_line` path must keep its single-line contract: it uses the renderer's
flat form and, when the wrap code is not `DoNotWrap` and the wrap would engage
(`WrapAlways`, or the flat list overflows), `try_one_line` returns `None` so
the multi-line layout wraps the list instead of contradicting it inside a
one-line collapse. No token is reordered and only whitespace is inserted at
the `|` boundaries, so R5 holds by construction; catch bodies, comments and
other unmodelled shapes keep today's echo (R4). The exact wrapped columns and
operator placement are pinned by the goldens and, if an IntelliJ installation
is available to the implementer, verified against real IntelliJ output before
the golden is committed — the same mitigation the binary-expression and
record requests used.

**Text-block rendering — `crates/core/src/formatter.rs`.** Grammar: a text
block is not a distinct node kind — `string_literal` is a choice of the
single-line `_string_literal` and the multiline `_multiline_string_literal`
(which contains raw newlines and `multiline_string_fragment` children), so a
text block is a `string_literal` node whose text spans lines. Today it reaches
the `_ => self.txt(node).to_string()` fallbacks in `expr` (L2059) and `flat`
(L2883) and is echoed verbatim (R4). Add a dedicated text-block renderer
(detect: kind `string_literal` whose text contains a newline; a single-line
`"""…"""` and an ordinary string literal are untouched) and route it through
explicit `string_literal` arms at those two sites. Both options default
`false`, so the renderer returns the verbatim text unless a scheme opts in
(AC2, no existing output changes).

`STRIP_WHITESPACE_FROM_BLANK_LINES_IN_TEXT_BLOCKS` is position-independent:
split the literal text on newlines and trim every whitespace-only line to
empty (blank lines lose their trailing spaces; lines with visible content are
never touched). It applies wherever a text block is rendered, including the
`flat` echo sites. Because a whitespace-only line's excess whitespace can be
part of the literal's value, this is the intentional, opt-in deviation the
request records; absent/false keeps byte-for-byte echo.

`ALIGN_MULTILINE_TEXT_BLOCKS` applies only to multiline text blocks rendered
through the expression path (statement values, where the caller passes
`indent` and the current column): realign the block to the code the
formatter emits by shifting every non-opening line (content lines and the
closing-delimiter line) by one uniform delta so the first content line sits at
the canonical continuation column for the statement (`col_after(0,
self.cont(indent))`-style anchoring; the exact target column and
closing-delimiter shape are pinned by the goldens). A uniform shift changes
only incidental whitespace — relative indentation between lines is preserved
and the stripped string value is unchanged — so the transform is safe under
R5 and matches the decision that the option "moves only layout whitespace"
(content indentation differences that survive incidental stripping are
untouched). If a uniform shift is impossible (a content line with fewer
leading spaces than the delta), fall back to the verbatim echo rather than
alter the value; single-line text blocks and ordinary strings never shift.
In `flat` contexts (which cannot contain a re-indented multiline literal and
carry no column context) the block stays verbatim modulo the strip option,
and unmodelled shapes keep today's R4 echo. The strip and align transforms
afterwards must keep the new goldens idempotent (R6): aligning an already-
aligned block and stripping already-empty blank lines are no-ops.

**Tests.** Follow the hard rules of `.agents/AGENTS.md`: one golden-pair
module per option in `crates/core/tests/options/<xml_option>.rs`, wired via
`#[path]` in `crates/core/tests/options.rs` (alphabetical: the two
`align_*` modules slot between `align_multiline_records` and
`annotation_parameter_wrap`, `multi_catch_types_wrap` between
`method_parameters_wrap` and `new_line_after_lparen_in_record_header`,
`strip_whitespace_from_blank_lines_in_text_blocks` between `right_margin` and
`tab_size`), starting `use super::common::*;`, doc header `//! <XML_OPTION> —
…` plus `//! Fixtures live under tests/java/<option>/.`, fixtures under
`crates/core/tests/java/<option>/` referenced by relative `include_str!` with
shared input/golden stems, no inline Java strings, no new test helpers (only
`style`, `format_with`, `format`, `default_style` exist), no topic suites.
The fixtures drive the fields directly via `style(...)`; absent-option cases
use `default_style()` (i.e. `format(INPUT)`). Each new golden is checked
idempotent by formatting the `*.out.java` a second time during development
(no `assert_idempotent` helper exists or is added). Fixture contents: a
multi-catch `catch (A | B | … e)` with long exception names at a narrow
`right_margin` for wrap codes `0`/`1`/`2`/`5` (codes `1` and `5` produce the
same per-type break on a flat list, exactly as `record_components` treats
them; the goldens record that) plus an absent-option default check; the same
wrapped input for `ALIGN_TYPES_IN_MULTI_CATCH` on / off / absent (absent
defaults `true`, so it aligns — mirroring the default-`true` record-alignment
file); a misindented multiline text block for `ALIGN_MULTILINE_TEXT_BLOCKS`
on / off / absent (absent and off both echo verbatim; on produces the
aligned golden, verified against IntelliJ when available); and a text block
whose blank lines carry trailing spaces for the strip option true / false /
absent (true strips, false and absent preserve byte-for-byte).

**Docs on delivery.** `docs/settings/java.md`: flip the two Text-blocks rows
(L172-173) and the two Multi-catch rows (L190-191) to ✅, and add a recorded-
divergence note under Multi-catch mirroring the Records note (L136-141) for
`MULTI_CATCH_TYPES_WRAP` (table default `1`, `JavaStyle::default()` ships
`DoNotWrap`). `README.md`: add the four options to the honoured-options table
and extend the formatting-behaviour notes — text blocks stay verbatim unless
the scheme opts in, the strip option is a value-touching deviation limited to
whitespace-only blank lines, and multi-catch type lists wrap at the `|`
operators on the continuation convention. `docs/requirements.md`: add a
requirement row (next free number, R16, priority low) tying the four options
to R3/R5, phrasing like the R10 row. Append `docs/dev/changelog.md` with the
convention/verification outcome recorded as the earlier requests did.

## Steps

- [ ] `crates/core/src/config.rs`: add the four `JavaStyle` fields with the
defaults above (banner comment after the record fields) and the four
`OptionDef` entries (group `"Text blocks"` / `"Multi-catch"`,
`Section::JavaCodeStyle`, `get`/`set` closures) between the records group and
`CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND`; `cargo build` + `cargo test` stay
green — no behaviour change yet (AC2 absent/default mapping).
- [ ] `crates/core/src/formatter.rs`: add the `catch_formal_parameter`
renderer (modifiers + `flat_type` per `catch_type` type child + ` | `
separators + name, flat and wrapped forms) on the `record_components`
pattern, and use it in the `try_stmt` catch arm (L1723-1737) and as the flat
form in `try_one_line` (L1671-1682) with the one-line guard (wrap would
engage → `None`); single-type catches and the `DoNotWrap` default stay
byte-identical, so existing goldens stay green (AC2).
- [ ] `crates/core/src/formatter.rs`: add the text-block renderer (strip +
uniform realign with verbatim fallback) with explicit `string_literal` arms
in `expr` (L2059) and `flat` (L2883); options-off output is byte-identical
to today (AC2).
- [ ] Add `crates/core/tests/options/multi_catch_types_wrap.rs` + fixtures
under `tests/java/multi_catch_types_wrap/` (over-margin `catch (A | B | C …
e)` at a narrow `right_margin`, goldens for codes `0`/`1`/`2`/`5` and an
absent-option default golden) and wire the module in `tests/options.rs`;
check the new goldens are idempotent and, if IntelliJ is available, that the
wrapped shape matches it (AC1, AC2 idempotency).
- [ ] Add `crates/core/tests/options/align_types_in_multi_catch.rs` +
fixtures under `tests/java/align_types_in_multi_catch/` (wrap engaged:
aligned-under-first-type on, continuation-indent off, absent-default-aligned)
and wire it in `tests/options.rs` (AC1).
- [ ] Add `crates/core/tests/options/align_multiline_text_blocks.rs` +
fixtures under `tests/java/align_multiline_text_blocks/` (misindented
multiline text block: aligned golden on, verbatim golden off, verbatim
absent-default golden) and wire it in `tests/options.rs`; verify the aligned
shape against IntelliJ when available (AC1).
- [ ] Add `crates/core/tests/options/strip_whitespace_from_blank_lines_in_text_blocks.rs`
+ fixtures under
`tests/java/strip_whitespace_from_blank_lines_in_text_blocks/` (blank lines
with trailing spaces: stripped golden on, byte-identical goldens for false
and absent) and wire it in `tests/options.rs` (AC1).
- [ ] Full-suite gate: run `cargo test`; existing goldens stay byte-identical
under default/absent schemes (text blocks verbatim, multi-catch single-line)
and each new golden formats to itself on a second pass (AC2, idempotency).
- [ ] Docs: flip the four `docs/settings/java.md` rows ❌ → ✅ and add the
`MULTI_CATCH_TYPES_WRAP` recorded-divergence note under Multi-catch; add the
four options to the README honoured-options table and a formatting-behaviour
note covering the opt-in strip deviation and the multi-catch wrap convention;
add the requirement row to `docs/requirements.md` (R16); append
`docs/dev/changelog.md`; run `cargo test` once more to confirm the shipped
state is green (AC3).
