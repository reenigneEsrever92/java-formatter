---
type: ChangeRequest
kind: feature
title: Honour the spacing-around-operators options
description: Apply the SPACE_AROUND_* operator-spacing options so binary/unary/assignment spacing follows the scheme.
state: proposed
priority: high
tags: [dev, formatter]
owner: maintainer
---

# Problem

Every row of the `docs/settings/common.md` "Spaces / Around operators" table is marked ❌: java-formatter parses none of them and instead emits a fixed canonical spacing — one space each side of binary and assignment operators, none around unary operators and the method-reference `::`, one after a type cast — that a scheme cannot adjust. A scheme that sets any of these options is only partially honoured; unimplemented options are safely ignored (R7 in `docs/requirements.md`).

# Proposal

Parse the following options into `config::JavaStyle` via the `OPTIONS` registry in `crates/core/src/config.rs`: `SPACE_AROUND_ASSIGNMENT_OPERATORS`, `SPACE_AROUND_LOGICAL_OPERATORS`, `SPACE_AROUND_EQUALITY_OPERATORS`, `SPACE_AROUND_RELATIONAL_OPERATORS`, `SPACE_AROUND_BITWISE_OPERATORS`, `SPACE_AROUND_ADDITIVE_OPERATORS`, `SPACE_AROUND_MULTIPLICATIVE_OPERATORS`, `SPACE_AROUND_SHIFT_OPERATORS`, `SPACE_AROUND_UNARY_OPERATOR`, `SPACE_AROUND_LAMBDA_ARROW`, `SPACE_AROUND_METHOD_REF_DBL_COLON`, `SPACE_AFTER_TYPE_CAST` — each an `OptionDef` with the IntelliJ built-in default from the table (absent → default), mapped to the existing `OptionValue::Bool`. In `crates/core/src/formatter.rs`, the space around each operator token follows its toggle where the token is emitted — including the wrapped binary layout shipped with `BINARY_OPERATION_WRAP` and the one-line lambda `->`.

Docs touched: on delivery the implementation flips the ❌ marks to ✅ for these rows in `docs/settings/common.md`, updates the README honoured-options table and formatting-behaviour notes, adds a requirement row to `docs/requirements.md`, and appends `docs/dev/changelog.md`.

# Decisions

1. **One family, one request.** Only the listed options are added; unlisted spacing options stay unimplemented and safely ignored (R7).
2. **Defaults.** IntelliJ built-in defaults: most default `true` — equal to today's canonical output, so absent/default schemes keep byte-identical goldens — while `SPACE_AROUND_UNARY_OPERATOR` and `SPACE_AROUND_METHOD_REF_DBL_COLON` default `false`.
3. **Semantics.** Whitespace-only change (R5); unmodelled shapes echoed verbatim (R4); adding/removing a single space is idempotent (R6).
4. **Per-operator granularity.** Each operator class is its own toggle, so e.g. logical spacing can differ from bitwise spacing, and `SPACE_AFTER_TYPE_CAST` governs only the gap after a `(Type) expr` cast.

# Acceptance criteria

- Per listed option: a golden fixture and per-option test file under `crates/core/tests/options/` testing the option's interesting values (on / off) plus the absent-option default.
- Toggle off → the affected operators render without the surrounding space in the `*.out.java` golden (e.g. `a+b` with additive off, `(int)x` with cast off); on (and by default) → today's canonical spacing.
- Spacing holds on wrapped binary expressions and one-line lambdas as well as flat expressions; `SPACE_AROUND_UNARY_OPERATOR` and `SPACE_AROUND_METHOD_REF_DBL_COLON` stay space-less when absent.
- Default-scheme output is unchanged and the whole suite is green (`cargo test`); the new goldens are idempotent.
- `docs/settings/common.md` marks are flipped to ✅ and the README / `docs/requirements.md` / `docs/dev/changelog.md` are updated.
