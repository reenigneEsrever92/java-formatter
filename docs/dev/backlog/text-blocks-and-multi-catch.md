---
type: ChangeRequest
kind: feature
title: Honour the text-block layout and multi-catch wrapping options
description: Implement the text-block alignment/whitespace options and multi-catch type-list wrapping/alignment.
state: proposed
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
