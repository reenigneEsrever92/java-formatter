---
type: ChangeRequest
kind: feature
title: Format Javadoc per the JD_* javadoc options
description: Implement the javadoc formatting options so doc comments are laid out per the scheme.
state: proposed
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

The whole "Javadoc" table in `docs/settings/java.md` — `ENABLE_JAVADOC_FORMATTING`, `CLASS_NAMES_IN_JAVADOC` and the `JD_*` options (`JD_ALIGN_PARAM_COMMENTS`, `JD_ALIGN_EXCEPTION_COMMENTS`, `JD_ADD_BLANK_AFTER_PARM_COMMENTS`, `JD_ADD_BLANK_AFTER_RETURN`, `JD_ADD_BLANK_AFTER_DESCRIPTION`, `JD_P_AT_EMPTY_LINES`, `JD_KEEP_INVALID_TAGS`, `JD_KEEP_EMPTY_LINES`, `JD_DO_NOT_WRAP_ONE_LINE_COMMENTS`, `JD_USE_THROWS_NOT_EXCEPTION`, `JD_KEEP_EMPTY_PARAMETER`, `JD_KEEP_EMPTY_EXCEPTION`, `JD_KEEP_EMPTY_RETURN`, `JD_LEADING_ASTERISKS_ARE_ENABLED`, `JD_PRESERVE_LINE_FEEDS`, `JD_PARAM_DESCRIPTION_ON_NEW_LINE`, `JD_INDENT_ON_CONTINUATION`) is all ❌. Comments today are never rewritten — block comments are echoed verbatim by the formatter (R4; the README never-corrupt contract) — so none of the javadoc options apply and a scheme that sets them is only partially honoured (safely ignored, R7). This family is deliberately one request because it needs a javadoc parsing/layout engine rather than a tweak at an existing construct.

# Proposal

Parse the javadoc options into `config::JavaStyle` via the `OPTIONS` registry in `crates/core/src/config.rs` (the `JD_*` XML names as written by the IDE, per the docs/settings note; `CLASS_NAMES_IN_JAVADOC` as an int with values `1`–`3`) and add a javadoc layout pass in `crates/core/src/formatter.rs` that runs when `ENABLE_JAVADOC_FORMATTING` is on: align `@param` / `@throws` descriptions per the `JD_ALIGN_*` options, add blank lines per the `JD_ADD_BLANK_*` options, keep or drop empty tags and lines per the `JD_KEEP_*` options, place `<p>` and leading asterisks per `JD_P_AT_EMPTY_LINES` / `JD_LEADING_ASTERISKS_ARE_ENABLED`, normalise `@exception` → `@throws` per `JD_USE_THROWS_NOT_EXCEPTION`, and wrap/indent per the remaining options. `JD_PRESERVE_LINE_FEEDS` and `JD_KEEP_EMPTY_LINES` govern how conservative the rewrite is; the initial slice may restrict itself to a safe subset — only reformat javadoc whose tags parse cleanly and echo everything else verbatim (R4) — with the full `JD_*` list above as the target, all within this one request.

Docs touched: on delivery the implementation flips the javadoc rows in `docs/settings/java.md` (❌ → ✅ for the applied subset, with a recorded-divergence note for the gating default), updates the README (comments-preserved-verbatim note, honoured-options table), adds a new requirement row to `docs/requirements.md`, and appends `docs/dev/changelog.md`.

# Decisions

- **Gated subsystem, one request.** `ENABLE_JAVADOC_FORMATTING` gates the whole engine, and the other options only shape the rewrite; delivering the engine is a single change request because splitting the `JD_*` options would leave the layout half-configured.
- **Opt-in gate, never-corrupt default.** `ENABLE_JAVADOC_FORMATTING`'s IntelliJ default is `true`, but comments are never rewritten today (R4) and default/absent schemes must stay byte-identical (existing goldens green) — so, mirroring the recorded `RECORD_COMPONENTS_WRAP` divergence in `docs/settings/java.md`, `JavaStyle::default()` ships the gate off (absent → javadoc stays verbatim; a scheme that sets the option explicitly, `true` or `false`, is honoured exactly). The remaining `JD_*` entries carry their table defaults (`JD_ALIGN_PARAM_COMMENTS` / `JD_ALIGN_EXCEPTION_COMMENTS` `true`, `JD_KEEP_*` `true`, `JD_LEADING_ASTERISKS_ARE_ENABLED` `true`, `JD_PRESERVE_LINE_FEEDS` `false`, …).
- **Safe subset first.** Only javadoc whose structure parses cleanly is reformatted; malformed, unusual, or one-line javadoc beyond the subset is echoed byte-for-byte (R4). Reformatting never drops or reorders prose/tags (R5), and `JD_PRESERVE_LINE_FEEDS` / `JD_KEEP_EMPTY_LINES` / `JD_KEEP_INVALID_TAGS` make the conservative knobs explicit, so output is idempotent (R6).
- **One family, one request.** Unlisted javadoc behaviour stays unimplemented and is ignored safely (R7).

# Acceptance criteria

- Fixtures + test files under `crates/core/tests/options/` for the applied subset (e.g. `javadoc_formatting.rs` covering `ENABLE_JAVADOC_FORMATTING` plus each exercised `JD_*` option at its interesting bool/int values): a cleanly parsed javadoc is laid out per the alignment/blank-line/keep defaults, and a messy or invalid-tag javadoc echoes byte-for-byte verbatim.
- With `ENABLE_JAVADOC_FORMATTING` absent (or `false`) javadoc output is byte-identical to today, and explicit-`true` schemes engage the engine.
- The whole suite stays green (`cargo test`) and the new goldens are idempotent.
- The applied `docs/settings/java.md` rows flip to ✅ (with the gate's divergence note), and README / `docs/requirements.md` / `docs/dev/changelog.md` are updated.
