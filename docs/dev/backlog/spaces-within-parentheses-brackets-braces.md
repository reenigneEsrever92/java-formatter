---
type: ChangeRequest
kind: feature
title: Honour the within-parentheses/brackets/braces spacing options
description: Apply the SPACE_WITHIN_* options so padding inside parens, brackets, braces and array initialisers follows the scheme.
state: done
verified: { by: maintainer, at: 2026-09-03T22:31:07Z }
priority: high
tags: [dev, formatter]
owner: maintainer
---

# Problem

Every row of the `docs/settings/common.md` "Spaces / Within parentheses, brackets, braces" table is marked ❌: java-formatter parses none of them and emits no inner padding — conditions are rendered with exactly their own parentheses (README formatting-behaviour notes) and calls, declarations, casts, brackets, braces and array initialisers come out tight, a fixed canonical style that cannot be adjusted. A scheme that sets any of these options is only partially honoured; unimplemented options are safely ignored (R7 in `docs/requirements.md`).

# Proposal

Parse the following options into `config::JavaStyle` via the `OPTIONS` registry in `crates/core/src/config.rs`: `SPACE_WITHIN_PARENTHESES`, `SPACE_WITHIN_METHOD_CALL_PARENTHESES`, `SPACE_WITHIN_EMPTY_METHOD_CALL_PARENTHESES`, `SPACE_WITHIN_METHOD_PARENTHESES`, `SPACE_WITHIN_EMPTY_METHOD_PARENTHESES`, `SPACE_WITHIN_IF_PARENTHESES`, `SPACE_WITHIN_WHILE_PARENTHESES`, `SPACE_WITHIN_FOR_PARENTHESES`, `SPACE_WITHIN_TRY_PARENTHESES`, `SPACE_WITHIN_CATCH_PARENTHESES`, `SPACE_WITHIN_SWITCH_PARENTHESES`, `SPACE_WITHIN_SYNCHRONIZED_PARENTHESES`, `SPACE_WITHIN_CAST_PARENTHESES`, `SPACE_WITHIN_BRACKETS`, `SPACE_WITHIN_BRACES`, `SPACE_WITHIN_ARRAY_INITIALIZER_BRACES`, `SPACE_WITHIN_EMPTY_ARRAY_INITIALIZER_BRACES`, `SPACE_WITHIN_ANNOTATION_PARENTHESES` — each an `OptionDef` with the IntelliJ built-in default from the table (absent → default), mapped to the existing `OptionValue::Bool`. In `crates/core/src/formatter.rs`, each toggle inserts padding just inside the paren / bracket / brace kind it names, with the empty-vs-nonempty variants distinct where IntelliJ splits them.

Docs touched: on delivery the implementation flips the ❌ marks to ✅ for these rows in `docs/settings/common.md`, updates the README honoured-options table and formatting-behaviour notes, adds a requirement row to `docs/requirements.md`, and appends `docs/dev/changelog.md`.

# Decisions

1. **One family, one request.** Only the listed options are added; unlisted spacing options stay unimplemented and safely ignored (R7).
2. **Defaults.** All default `false`, which equals today's no-padding output, so absent or default schemes keep byte-identical goldens.
3. **Semantics.** Whitespace-only change (R5); unmodelled shapes echoed verbatim (R4); padding is idempotent (R6).
4. **Per-construct granularity.** Each paren / bracket / brace kind is its own toggle, so e.g. `SPACE_WITHIN_IF_PARENTHESES` affects only `if (...)`, and the empty variants (`SPACE_WITHIN_EMPTY_METHOD_CALL_PARENTHESES`, `SPACE_WITHIN_EMPTY_METHOD_PARENTHESES`, `SPACE_WITHIN_EMPTY_ARRAY_INITIALIZER_BRACES`) are independent of their non-empty counterparts.

# Acceptance criteria

- Per listed option: a golden fixture and per-option test file under `crates/core/tests/options/` testing the option's interesting values (on / off) plus the absent-option default.
- Toggle on → the governed paren / bracket / brace is padded in the `*.out.java` golden (`if( x )`, `f( args )`, `f( )`, `( Type ) expr`, `{ 1, 3, 5 }`, `a[ 0 ]`); off (and by default) → output stays tight.
- Default-scheme output is unchanged and the whole suite is green (`cargo test`); the new goldens are idempotent.
- `docs/settings/common.md` marks are flipped to ✅ and the README / `docs/requirements.md` / `docs/dev/changelog.md` are updated.

# Implementation plan

## Approach

**Configuration side — `crates/core/src/config.rs`.** Add 18 `pub bool`
fields to `JavaStyle` (L105-150) in a new `// --- spacing within parens /
brackets / braces ---` block, named from the XML options
(`space_within_parentheses`, `space_within_method_call_parentheses`,
`space_within_empty_method_call_parentheses`, `space_within_method_parentheses`,
`space_within_empty_method_parentheses`, `space_within_if_parentheses`,
`space_within_while_parentheses`, `space_within_for_parentheses`,
`space_within_try_parentheses`, `space_within_catch_parentheses`,
`space_within_switch_parentheses`, `space_within_synchronized_parentheses`,
`space_within_cast_parentheses`, `space_within_brackets`, `space_within_braces`,
`space_within_array_initializer_braces`,
`space_within_empty_array_initializer_braces`,
`space_within_annotation_parentheses`), each defaulting to `false` in `Default`
(L152-182) — equal to today's tight output, so absent/default schemes keep
byte-identical goldens (Decision 2). Then add 18 `OptionDef` entries to the
`OPTIONS` registry (L232-567), in the settings-table order: exact XML `name`,
`section: Section::CodeStyleJava` (all live in the JAVA `codeStyleSettings`
block), `default: OptionValue::Bool(false)`, a new `group: "Spaces"`, and a
description echoing the table's "Applies to" column. Parsing, serialization
and the GUI are all registry-driven (`parse_codestyle` L696-718,
`serialize_codestyle` L740-761; the GUI groups `OPTIONS` by `def.group`), so a
registered option needs no other wiring — round-trip and checkbox support come
for free. Per the AGENTS hard rules there are **no** `parse_codestyle` tests;
the mapping is exercised end-to-end through the per-option goldens.

**Rendering side — `crates/core/src/formatter.rs`.** Today every paren /
bracket / brace pair is emitted tight (`format!("({})", inner)` and
friends). Padding must be **per construct**, not blanket, because the same
tree-sitter node kind is reused across constructs: `parenthesized_expression`
is the condition of `if` (L1493-1498), `while` / `do` (L1606-1616,
L1646-1649), `switch` (L1813-1825) and `synchronized` (L1778-1790) as well as
an ordinary parenthesised sub-expression, and each is governed by its own
toggle (Decision 4). Two idioms cover every site:

- **Structured sites** (the formatter assembles the delimiters): add a small
  private helper on `Fmt`, e.g. `fn within(open: char, close: char, pad:
  bool, inner: &str) -> String` plus an empty-aware variant taking separate
  non-empty / empty booleans. Rules: one space per side; when `inner` is
  empty the pair stays bare unless the construct has an empty variant in the
  request (method-call / method-declaration parens, array-initialiser
  braces); when `inner` starts or ends with `'\n'` that side is left bare so
  no trailing whitespace is produced (wrapped argument lists). Only
  whitespace between tokens changes, so R5 holds and re-formatting the
  padded output re-parses the same tree and reproduces the same string —
  R6 idempotency by construction, no sniffing needed.
- **Keyword-condition sites**: the builders must render the condition's
  *inner* expression and wrap the paren pair themselves with the keyword's
  boolean (they must not let the condition node fall through `expr`/`flat`,
  whose plain-paren arm would apply `SPACE_WITHIN_PARENTHESES` to e.g. an
  `if` condition, violating Decision 4). Small per-keyword wrappers — one
  for the multi-line path and one flat variant for the one-line collapse
  paths (`if_one_line` L1339-1358, one-line `while`/`do`/`synchronized`,
  `switch_one_line` L1924-1987) — destructure the outer
  `parenthesized_expression` and pad with the keyword's toggle, so the
  collapsed candidate matches the multi-line padding. Nested parentheses
  *inside* a condition keep flowing through `expr`/`flat` and are padded as
  plain parentheses.
- **Textual sites** (the pair comes from source text, not assembly): the
  classic `for` header (L1532-1538, canonicalised via `normalise_ws`) and
  the try-with-resources `resource_specification` (L1656-1664, L1707-1714).
  Padding targets only the *outermost* paren pair after the keyword, with
  an idempotent insertion — add a space only when the neighbour is not
  already a space — so a padded header reformats to itself.

Emission-site map (option → site(s); line numbers as of today):

| Option | Site(s) in `formatter.rs` |
| --- | --- |
| `SPACE_WITHIN_PARENTHESES` | plain `parenthesized_expression` arms of `expr` (L2049-2055) and `flat` (L2854-2860) |
| `SPACE_WITHIN_METHOD_CALL_PARENTHESES` / `…_EMPTY_…` | `flat_args` (L2101-2109), `args_wrapped` flat + wrapped layouts (L2132-2179), `flat_inv` / `inv_wrapped` bare-`()` fallbacks (L2097, L2127) — covers calls, chains (`fmt_chain` L2233-2247) and `new` constructor args (`new_expr` L2268-2282, `flat_new` L2905-2908); empty when the `arguments` node has no named children |
| `SPACE_WITHIN_METHOD_PARENTHESES` / `…_EMPTY_…` | `formal_params` (L1143-1193) incl. its empty branch (L1146-1148), used by `method_decl` (L795) and `constructor_decl` (L838). Lambda `formal_parameters` (`flat_formal_params` L2525-2533) and record headers (`record_components` L642-691) stay tight — records have their own backlog CR; if an IntelliJ installation is available, verify the lambda/record treatment and adjust, otherwise leave them tight |
| `SPACE_WITHIN_IF_PARENTHESES` | `if_stmt` condition (L1495-1498) and `if_one_line` (L1341-1348) |
| `SPACE_WITHIN_WHILE_PARENTHESES` | `while_stmt` condition (L1606-1609) + one-line (L1593-1599); `do_while` trailing condition (L1646-1649) + one-line (L1623-1627) — the `do`-`while` parens share the while toggle (no separate do option exists; cross-check with IntelliJ if available) |
| `SPACE_WITHIN_FOR_PARENTHESES` | classic `for_stmt` header text (L1532-1538) + one-line candidate (L1542-1543); `enhanced_for` (L1570, L1587) |
| `SPACE_WITHIN_TRY_PARENTHESES` | try-with-resources text in `try_stmt` (L1707-1714) and `try_one_line` (L1658-1664) |
| `SPACE_WITHIN_CATCH_PARENTHESES` | `catch ({})` assembly in `try_stmt` (L1736) and `try_one_line` (L1681) |
| `SPACE_WITHIN_SWITCH_PARENTHESES` | `switch_stmt` condition (L1814-1825) incl. the empty-body `switch (c) {}` form; `switch_one_line` (L1925, L1986); `switch_expr` routes to these |
| `SPACE_WITHIN_SYNCHRONIZED_PARENTHESES` | `sync_stmt` lock (L1779-1790) + one-line (L1766-1772) |
| `SPACE_WITHIN_CAST_PARENTHESES` | `cast_expression` arms of `expr` (L2025-2035) and `flat` (L2832-2842) |
| `SPACE_WITHIN_BRACKETS` | `array_access` arms of `expr` (L1999-2009) and `flat` (L2746-2756); array-*type* dimensions stay tight (the table's scope is `[ expr ]` indexing) |
| `SPACE_WITHIN_ARRAY_INITIALIZER_BRACES` / `…_EMPTY_…` | `flat_arr_init` (L2968-2979) and the flat branch of `array_init` (L2560-2563); the multi-line branch (L2564-2576) pads only sides adjacent to content. Shared with annotation array initialisers (`element_value_array_initializer`) |
| `SPACE_WITHIN_BRACES` | inline code-block / body braces: the empty `{}` returned by `block` (L1252), `class_body` (L708), `flat_block` (L2958) and the empty switch body (L1824) → `{ }`. Single-line `{ … }` bodies (`one_line_body` L1334) already carry one inner space and stay unchanged (`SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT` — which would remove them — is a different, unlisted option). Cross-check with IntelliJ when available |
| `SPACE_WITHIN_ANNOTATION_PARENTHESES` | flat `annotation` (L1024-1025) + `annotation_expanded` (L1086-1139) + `flat_annotation` (L2887-2894); a bare `@A()` stays tight (no empty variant in the request) |

**Tests.** Follow the AGENTS hard rules exactly: golden pairs only, one test
file per option under `crates/core/tests/options/<XML_OPTION>.rs` wired via
tests/options.rs, fixtures under `tests/java/<option>/` referenced with
`include_str!("../java/<option>/<scenario>.java")`, no `assert_idempotent` /
no topic suites / no inline Java strings. Each option file asserts its
interesting values: toggle on → padded `*.out.java` golden, toggle off /
absent default → tight golden (default case via `format(IN)` against a
`*_default.out.java`, mirroring `binary_operation_wrap.rs`). Idempotency of the
new goldens is guaranteed by construction; additionally re-format each new
`.out.java` once during development and confirm it is byte-identical (a local
check, not a committed helper). If an IntelliJ installation is available,
cross-check the goldens against real IntelliJ output and adjust if it differs
(operator/edge semantics flagged above), as done for the binary-wrap golden.

## Steps

- [x] `crates/core/src/config.rs`: add the 18 `space_within_*` `bool` fields
      to `JavaStyle` and set each to `false` in `Default` (Decision 2 —
      default output stays tight).
- [x] `crates/core/src/config.rs`: add the 18 `OptionDef` entries to
      `OPTIONS` (JAVA `codeStyleSettings` block, `Bool(false)`,
      group "Spaces", table-order) so parse / serialize / GUI pick them up
      (Decision 1 — unlisted spacing options stay unimplemented, R7).
- [x] `formatter.rs`: add the structured `within` padding helpers (empty /
      non-empty variants, no-trailing-whitespace rule) and wire the three
      self-contained expression arms — plain `parenthesized_expression`
      (`expr` L2049, `flat` L2854), `cast_expression` (`expr` L2025, `flat`
      L2832) and `array_access` brackets (`expr` L1999, `flat` L2746).
      Register `tests/options/space_within_parentheses.rs`,
      `space_within_cast_parentheses.rs` and `space_within_brackets.rs` with
      their fixtures + goldens (on / off / absent default) in tests/options.rs
      (AC: per-option golden + `if( x )`-family padding for plain parens,
      `( Type ) expr`, `a[ 0 ]`).
- [x] Method-call parens: pad through `flat_args` / `args_wrapped` / the
      bare-`()` fallbacks so calls, chains and `new` constructor args pad as
      `f( args )`, with the empty variant `f( )` split per
      `SPACE_WITHIN_EMPTY_METHOD_CALL_PARENTHESES`. Register
      `space_within_method_call_parentheses.rs` and
      `space_within_empty_method_call_parentheses.rs` + fixtures (AC: golden
      pairs for both toggles and defaults).
- [x] Method-declaration parens: pad `formal_params` (flat + wrapped +
      empty branches) so `void f( params )` and `void f( )` follow
      `SPACE_WITHIN_METHOD_PARENTHESES` / `…_EMPTY_…`; leave lambda and
      record-header parens tight. Register
      `space_within_method_parentheses.rs` and
      `space_within_empty_method_parentheses.rs` + fixtures (AC: golden
      pairs).
- [x] Keyword-condition parens: add per-keyword wrappers (multi-line + flat)
      that destructure the outer `parenthesized_expression`, and use them in
      `if_stmt`/`if_one_line`, `while_stmt` + one-line, `do_while` (trailing
      `while`, sharing the while toggle), `switch_stmt`/`switch_one_line`
      (incl. the empty-body form) and `sync_stmt` + one-line. Register
      `space_within_if_parentheses.rs`, `space_within_while_parentheses.rs`,
      `space_within_switch_parentheses.rs` and
      `space_within_synchronized_parentheses.rs` + fixtures (AC:
      `if( x )`-style goldens per keyword; plain `SPACE_WITHIN_PARENTHESES`
      must not affect them — Decision 4).
- [x] `for` / `try` / `catch` parens: pad the *outermost* paren pair of the
      canonicalised classic-`for` header and of the `enhanced_for` assembly
      (`SPACE_WITHIN_FOR_PARENTHESES`), the try-with-resources
      `resource_specification` text (`SPACE_WITHIN_TRY_PARENTHESES`), and the
      `catch ({})` assembly (`SPACE_WITHIN_CATCH_PARENTHESES`), including the
      one-line paths, using the idempotent insertion rule. Register
      `space_within_for_parentheses.rs`, `space_within_try_parentheses.rs`
      and `space_within_catch_parentheses.rs` + fixtures (AC: golden pairs).
- [x] Array-initialiser and code-block braces: pad `flat_arr_init` / the
      flat `array_init` branch per `SPACE_WITHIN_ARRAY_INITIALIZER_BRACES`
      (`{ 1, 3, 5 }`) and the empty branch per
      `SPACE_WITHIN_EMPTY_ARRAY_INITIALIZER_BRACES` (`{ }`); pad the inline
      empty code-block / body braces per `SPACE_WITHIN_BRACES` (`{}` →
      `{ }` at the `block` / `class_body` / `flat_block` / empty-switch
      sites). Register `space_within_array_initializer_braces.rs`,
      `space_within_empty_array_initializer_braces.rs` and
      `space_within_braces.rs` + fixtures (AC: golden pairs).
- [x] Annotation parens: pad `@Anno( args )` in `annotation` (flat),
      `annotation_expanded` and `flat_annotation` per
      `SPACE_WITHIN_ANNOTATION_PARENTHESES`; `@A()` stays tight. Register
      `space_within_annotation_parentheses.rs` + fixtures (AC: golden pair).
- [x] Run `cargo test` and confirm the whole suite is green with no existing
      golden changed (all defaults are `false`); re-format each new
      `.out.java` once and confirm byte-identical output (R6 idempotency,
      local check only); if IntelliJ is available, cross-check the goldens
      and align any edge semantics (records/lambdas, `do`-`while`, empty
      blocks, wrapped-layout padding), recording the outcome (AC: green
      suite, idempotent goldens).
- [x] Docs: flip the 18 ❌ marks to ✅ in the `docs/settings/common.md`
      "Spaces / Within parentheses, brackets, braces" table; add the 18 rows
      to the README honoured-options table and a formatting-behaviour note
      (padding is off by default; each `SPACE_WITHIN_*` inserts one space
      just inside the delimiter pair it names, whitespace-only); add a
      requirement row (R16) to `docs/requirements.md`; append a changelog
      entry to `docs/dev/changelog.md`; re-run `cargo test` (AC: docs marks
      flipped ✅, README / requirements / changelog updated, suite green).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
