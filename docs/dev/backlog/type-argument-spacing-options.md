---
type: ChangeRequest
kind: feature
title: Honour the type-argument and type-parameter spacing options
description: Make generic spacing configurable via the four angle-bracket spacing options.
state: proposed
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
