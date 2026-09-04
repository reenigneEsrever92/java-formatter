---
type: ChangeRequest
kind: feature
title: Honour the spacing-around-operators options
description: Apply the SPACE_AROUND_* operator-spacing options so binary/unary/assignment spacing follows the scheme.
state: done
verified: { by: maintainer, at: 2026-09-03T21:29:16Z }
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

# Implementation plan

## Approach

**Configuration (src/config.rs).** Twelve new `bool` fields on `JavaStyle` —
`space_around_assignment_operators`, `space_around_logical_operators`,
`space_around_equality_operators`, `space_around_relational_operators`,
`space_around_bitwise_operators`, `space_around_additive_operators`,
`space_around_multiplicative_operators`, `space_around_shift_operators`,
`space_around_unary_operator`, `space_around_lambda_arrow`,
`space_around_method_ref_dbl_colon`, `space_after_type_cast` — in a new
`// --- operator spacing ---` group of the struct and of `Default`, defaulted
per the settings table (decision 2): nine rows `true` equal today's emitted
single-space canonical (assignment, logical, equality, relational, bitwise,
additive, multiplicative, shift, lambda arrow), `space_after_type_cast` also
`true` (see below), and `space_around_unary_operator` +
`space_around_method_ref_dbl_colon` `false`. `JavaStyle` is built only via
`Default`, so no other literal site needs changes. Add one `OptionDef` per
option to the `OPTIONS` registry (group `"Spaces"`, `OptionValue::Bool`,
`Section::CodeStyleJava` — all twelve live in the JAVA `codeStyleSettings`
block, matching their table in `docs/settings/common.md`), each `default`
equal to the field default so the serialize/parse round-trip stays exact;
parsing and serialization are registry-driven, so `parse_codestyle` /
`serialize_codestyle` need no changes — absent → default falls out of
`OptionMap::get_bool`, and the GUI's option panel iterates `OPTIONS`, so the
new entries appear automatically as checkboxes — no GUI source change. Per
AGENTS.md there are no `parse_codestyle` tests and no config-XML topic suite;
the mapping is exercised through the per-option golden tests (absent-option
default = default-style golden).

One deliberate consequence to record: `SPACE_AFTER_TYPE_CAST` defaults `true`
(the table's IntelliJ value) while today's cast arms emit `(T)x` with no space
— so default/absent schemes now render `(int) x`, matching IntelliJ. This is a
fidelity fix, not a regression: no existing golden contains a cast or a method
reference (verified by grepping `tests/java/`), so the shipped suite stays
byte-identical; the changelog calls the new canonical out.

**Rendering (src/formatter.rs).** The space around each operator token follows
its toggle wherever the token is emitted, in both duplicated renderers — the
structured `expr` path and the newline-free `flat` path. Add a small helper
that classifies an operator token and returns the separator to emit:
assignment ops (`= += -= *= /= %= &= |= ^= <<= >>= >>>=`) →
`space_around_assignment_operators`; `&&` / `||` → logical; `==` / `!=` →
equality; `<` / `>` / `<=` / `>=` → relational; `&` / `|` / `^` → bitwise;
`+` / `-` → additive; `*` / `/` / `%` → multiplicative; `<<` / `>>` / `>>>` →
shift — returning `" "` when the toggle is on and `""` when off. Sites to
route through it:

- Binary: the flat text built in `Fmt::binary` and the `flat()` binary arm both
  hard-code `" {} {} "`-style joins; the wrapped layout shipped with
  `BINARY_OPERATION_WRAP` (`binary_spine` continuation lines
  `cont + op + ' ' + operand`, chop-down recursion through `binary_operand` →
  `binary`) must use the toggle for the space after the operator, so an
  additive-off wrapped sum renders `+beta()` with the operand glued on.
- Assignment: `assignment()`, the `flat()` assignment arm, the wrapped path
  `assign_expr()`, and the declarator-initialiser joins in `field_decl` /
  `local_var` (`"{} = {}"` plus their `assign_expr` prefix forms). The
  annotation `element_value_pair` `=` is deliberately unchanged.
- Unary / update: the `expr()` and `flat()` unary arms insert the separator
  between `operator` and `operand`. `update_expression` (prefix/postfix
  `++` / `--`; the grammar gives it no fields — a named `operand` child plus an
  anonymous `++` / `--` token, prefix vs postfix by token position) is today
  echoed verbatim, so `i ++` survives reformatting; rebuild it from its
  children so the toggle applies and spaced input canonicalises to `i++` /
  `++i`, keeping the verbatim echo (R4) for nodes with extra children such as
  comments.
- Lambda: `lambda()` and `flat_lambda()` emit `params -> body`; build the
  arrow separator from `space_around_lambda_arrow` (default keeps ` -> `, off
  gives `(a, b)->a + b`). The `c + params.len() + 4` body-column bookkeeping
  must derive from the emitted separator (`+ 2` when off) so wrap decisions
  stay exact. Switch-rule `case x ->` arrows are not lambdas and stay
  unchanged.
- Method reference: `expr()` and `flat()` currently echo the whole
  `method_reference` node, so source spacing inside `::` (`A :: new`) is
  preserved; rebuild `qualifier :: [type_arguments] name` from the node's
  children with the `space_around_method_ref_dbl_colon` separator (default /
  absent → `A::new`, on → `A :: new`), falling back to verbatim echo for
  unexpected shapes or comment-bearing nodes (R4).
- Cast: the `expr()` and `flat()` cast arms join `(type)` to the value with the
  separator from `space_after_type_cast` — default on now emits `(int) x`,
  off `(int)x`.

Every hard-coded column constant that assumes the old canonical spacing (`c +
ty.len() + 2` in the cast arms, `+ name.len() + 3` in the declarator joins,
`+ op.len() + 2` in `assign_expr`, `c + left.len() + 4` in `assignment`, `c +
params.len() + 4` in `lambda`) is replaced by arithmetic over the separator
actually emitted, so margin/wrap decisions stay exact. Deliberately unchanged:
ternary `?` / `:` (the separators CR), `instanceof`, annotation element-value
`=`, switch `->`, and generic-type spacing (the type-argument-spacing CR).
Changes are whitespace-only (R5); inserting/removing one space is idempotent
(R6).

**Tests (hard rules from AGENTS.md).** Twelve option files under
`crates/core/tests/options/<xml_option>.rs` (lower-snake of the XML name, e.g.
`space_around_method_ref_dbl_colon.rs`), each starting
`use super::common::*;` with the `//! <XML_OPTION> — …` / `//! Fixtures live
under tests/java/<option>/.` doc header, wired in `tests/options.rs` via
`#[path = "options/<name>.rs"] mod <name>;`, fixtures under
`tests/java/<option>/` embedded through relative `include_str!` paths. Each
file asserts two goldens of one input: the option toggled away from its default
(off for the ten default-`true` options, on for unary and method-ref) and the
absent-option default via `default_style()`, asserting the canonical output.
Inputs mix the option's operators with one other operator class so the golden
shows only the intended toggle moving (e.g. assignment off → `int total=a +
b;` while `+` stays spaced), and place a construct inside a call argument so
both the `expr` and the `flat` renderers are covered. Two files carry the AC3
interplay cases: `space_around_additive_operators` also has a wrapped long-sum
scenario (`right_margin` 40 + `WrapIfLong` + off → glued `+beta()`
continuation lines) and `space_around_lambda_arrow` uses one-line lambdas
(expression-bodied and simple block bodies under
`KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`). No inline Java strings, no new helpers, no
`parse_codestyle` tests.

**Docs.** `docs/settings/common.md` flips the twelve "Around operators" rows
from ❌ to ✅, the README honoured-options table gains the twelve options and a
formatting-behaviour note (spacing follows the toggles; defaults as above; the
type-cast canonical is now spaced), `docs/requirements.md` gains an R16 row and
mentions it in the milestones paragraph, and `docs/dev/changelog.md` is
appended.

## Steps

- [x] config.rs: add the twelve `bool` fields to `JavaStyle` with the table
      defaults and the twelve `OptionDef` entries (group "Spaces",
      `OptionValue::Bool`, `Section::CodeStyleJava`) (AC: absent → default
      mapping).
- [x] formatter.rs: add the operator-separator helper; route the binary
      emissions through it — the flat text in `Fmt::binary` and the `flat()`
      binary arm, and the wrapped continuation lines
      (`cont + op + sep + operand`) (AC2, AC3 wrapped).
- [x] formatter.rs: route assignment through it — `assignment()`, the `flat()`
      assignment arm, `assign_expr()`, and the `field_decl` / `local_var`
      declarator `=` joins (AC2).
- [x] formatter.rs: unary arms plus the `update_expression` rebuild, and the
      lambda `->` separators with corrected body columns (AC2, AC3 one-line
      lambdas).
- [x] formatter.rs: method-reference rebuild and the cast separators, with
      column bookkeeping derived from the emitted separators; run `cargo test`
      and confirm no existing golden changes (AC2, AC4).
- [x] Tests: create the eight operator-token option files (assignment plus the
      seven binary classes — logical, equality, relational, bitwise,
      additive, multiplicative, shift) with fixtures, goldens and
      `tests/options.rs` wiring, including the additive wrapped long-sum
      scenario (AC1, AC2, AC3).
- [x] Tests: create the remaining four option files (`space_around_unary_operator`,
      `space_around_lambda_arrow`, `space_around_method_ref_dbl_colon`,
      `space_after_type_cast`) with fixtures, goldens and wiring (AC1, AC2,
      AC3 default-false absent cases).
- [x] Verify: `cargo test` green across the workspace; every new golden
      idempotent under its own style; `cargo build` for the whole workspace
      (the GUI compiles with the new registry entries); if an IntelliJ
      installation is available, cross-check the cast / unary-on / `::`-on
      goldens against real output and record the outcome in the changelog
      (AC4).
- [x] Docs: flip the twelve marks in `docs/settings/common.md`, update the
      README honoured-options table and formatting-behaviour notes, add the
      requirement row and milestone mention to `docs/requirements.md`, append
      `docs/dev/changelog.md`, and re-run `cargo test` (AC5).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
