---
type: ChangeRequest
kind: feature
title: Honour the builder-method wrapping options
description: Implement BUILDER_METHODS and KEEP_BUILDER_METHODS_INDENTS for chained builder calls.
state: proposed
priority: low
tags: [dev, formatter]
owner: maintainer
---

# Problem

`METHOD_CALL_CHAIN_WRAP` already ships for generic chains (README honoured-options table, `JavaStyle.method_call_chain_wrap`), but the builder-method rows in docs/settings/common.md "Builder method calls" are ❌: `BUILDER_METHODS` (string, comma-separated method names, default `""`) and `KEEP_BUILDER_METHODS_INDENTS` (bool, default `false`).
A scheme that names specific methods gets no builder treatment for their chains, and because the registry's `OptionValue` has no String variant yet the name list cannot even be parsed into `JavaStyle`.
Every chain is formatted as one generic family, so schemes that set these options are only partially honoured — both are safely ignored today (R7).

# Proposal

Add the parsed `BUILDER_METHODS` list and `keep_builder_methods_indents: bool` (default `false`) to `JavaStyle` via `OptionDef` entries in the `OPTIONS` registry in crates/core/src/config.rs — this family introduces `OptionValue::String` (the comma-separated value is split into the `JavaStyle` field; absent → `""`) and the registry/serialize path must round-trip it — then apply them in crates/core/src/formatter.rs at chained-call rendering: chains whose calls match the listed names get the builder treatment for wrapping and indentation (composing with the shipped `METHOD_CALL_CHAIN_WRAP`), and with `KEEP_BUILDER_METHODS_INDENTS` the wrapped continuation lines keep the chain's indentation instead of stepping at the continuation indent.

Docs touched: `docs/settings/common.md` "Builder method calls" marks flipped ❌→✅, the README honoured-options table and formatting-behaviour notes, `docs/requirements.md` (a new requirement row), and `docs/dev/changelog.md` on delivery.

# Decisions

1. **One family, one request.** Only the two listed options ship; the sibling chain options belong to other requests — `WRAP_SEMICOLON_AFTER_CALL_CHAIN` to wrapping-expressions-and-statements and `CHAINED_CALL_INDENT` to the indentation request — and the remaining ❌ rows stay unimplemented and safely ignored (R7).
2. **Defaults.** The empty list and `false` are the IntelliJ built-ins from the table, so default and absent schemes format every chain exactly as `METHOD_CALL_CHAIN_WRAP` does today — byte-identical output and existing goldens stay green.
3. **Semantics.** R5 holds — the builder treatment only inserts line breaks at `.` boundaries; the call sequence and arguments are unchanged. R4 echoes unmodelled chain shapes verbatim; R6 is pinned by re-formatting the new goldens.
4. **Registry.** `OptionValue` currently carries only Bool/UInt/Wrap/Brace scalars; the String variant for `BUILDER_METHODS` (default `""`) must round-trip parse(serialize(style)) == style for both the empty default and a non-empty comma-separated list.

# Acceptance criteria

- `tests/options/builder_methods.rs` (fixtures under `tests/java/builder_methods/`) asserts that a chain of calls named in the list and overflowing the margin gets the builder-treatment golden, while the same fixture with the option absent follows the plain `METHOD_CALL_CHAIN_WRAP` layout.
- `tests/options/keep_builder_methods_indents.rs` asserts `true` vs `false`/absent differ only in the continuation-line indentation of a wrapped builder chain.
- parse(serialize(style)) == style for a non-empty `BUILDER_METHODS` list (`value="a,b"` round-trips) and for the `""` default; default-scheme output is unchanged and the whole suite stays green (`cargo test`).
- docs/settings marks are flipped ❌→✅; the README, `docs/requirements.md` and `docs/dev/changelog.md` are updated with the implementation; the new goldens are idempotent (R6).
