---
type: ChangeRequest
kind: improvement
title: Normalise spacing around generic type arguments
description: Stop copying type-argument spacing from the source and emit canonical spacing instead.
state: done
verified: { by: maintainer, at: 2026-09-03T10:13:52Z }
priority: low
tags: [dev, formatter]
owner: maintainer
---

# Problem

Spacing inside generic type-argument lists is copied from the source rather
than normalised (README limitation). Inputs such as `Map<String ,Integer>`,
`List< String >`, or `Foo<Bar<Baz > >` keep their odd spacing, so formatting
does not converge to a canonical form for generics and hand-typed or
machine-generated spacing survives into the output.

# Proposal

Normalise whitespace inside generic type-argument lists when the formatter
renders type references: no space immediately inside the angle brackets,
no space before a comma, one space after a comma, and no stray spaces around
nested brackets — producing canonical forms like `Map<String, Integer>` and
`Foo<Bar<Baz>>`. Only spacing is touched; the types themselves are otherwise
formatted by the existing machinery.

Docs touched: `README.md` (limitations), `docs/requirements.md` (R14),
`docs/dev/changelog.md` on completion.

# Decisions

- **Canonical IntelliJ spacing.** The output matches IntelliJ's own generic
  spacing (no inner padding, single space after commas) rather than
  preserving any input variation.
- **Whitespace-only change.** No effect on semantics (R5); where a
  type-argument list cannot be parsed, the verbatim-echo path (R4) applies.

# Acceptance criteria

- Fixtures with irregular generic spacing (`List< String >`,
  `Map<String ,Integer>`, nested `>>` with spaces) produce the canonical
  form in their `*.out.java` golden output.
- Correctly spaced input is unchanged, so existing fixtures and the
  idempotency suite stay green (`cargo test`).
- The README's "Type-argument spacing … copied from the source" limitation is
  removed or updated to describe normalisation.

# Implementation plan

## Approach

Generic spacing is copied verbatim today because type text is echoed with
`self.txt(...)` at each type site rather than rendered from the tree. The
sites include: local-variable and field types (`local_var` L1321,
`field_decl` L806), parameter types (`flat_param` L1095, spread L1101),
enhanced-for types (L1441), cast types (L1653, `flat` L2223), `instanceof`
right-hand types (L1665/2235), class `extends`/`implements` and interface
`extends` lists (L349-360, L393-398), invocation/`new` `type_arguments`
(`flat_inv` L1710-1713, `inv_wrapped` L1737-1740, `new_expr` L1875-1878,
`flat_new` L2276-2279, chain links in `fmt_chain`), and declaration
`type_parameters` in class/interface/record/method/constructor headers
(L342-344, 384-386, 494-498, and the method/constructor variants). Correctly
spaced canonical input must remain byte-identical, so the canonical renderer
must only differ from `txt` where spacing is non-canonical.

The approach is a small type renderer instead of string post-processing:
`fn flat_type(&self, node: Node) -> String` handling the tree-sitter-java type
kinds (`type_identifier`, `scoped_type_identifier`, `generic_type`, nested
generics, arrays, primitive, wildcards `? extends`/`? super`), joining
type-argument lists canonically as `<A, B<C>>` — no space inside the angle
brackets, no space before a comma, one space after — and `fn
flat_type_args(&self, node: Node) -> String` for the `type_arguments` child
seen on invocations and `new`. Where a node's shape is unexpected (e.g. an
`ERROR` subtree), fall back to `self.txt` verbatim (R4). Then route each type
site above through the renderer, replacing the `self.txt`/raw-"type" reads.

A raw-string normaliser that edits bracket/commas in place is tempting but
unsafe inside generic _expressions_ nested in types is not the risk — the
real risk is text blocks, comments, and shift operators, which never appear
inside a `type`/`type_arguments`/`type_parameters` node — but the structural
renderer is still preferred because it also fixes nested spacing
(`Foo<Bar<Baz > >`) deterministically and is testable per kind. This is a
whitespace-only change (R5): golden output differs only for irregular input.

## Steps

- [x] Add `flat_type` (type kinds incl. wildcards and nested generics) and
      `flat_type_args` to src/formatter.rs; unknown shapes echo `txt` (R4).
- [x] Enumerate remaining verbatim type reads not listed above with a grep
      for `fld(.*"type")|type_arguments|type_parameters|extends_interfaces`
      and route each through the new renderers.
- [x] Route the declaration type-parameter lists (`<T extends X, U>`) through
      a canonical `flat_type_params` so header generics normalise too
      (methods, constructors, records, classes).
- [x] Fixture `tests/java/types/generic_spacing.java` + `.out.java` covering
      field/local/param/cast/extends/implements sites with irregular spacing
      (`List< String >`, `Map<String ,Integer>`, `Foo<Bar<Baz > >`)
      (AC1); keep correctly spaced input fixtures untouched.
- [x] Add tests in tests/types.rs (or a new tests/generics.rs): golden
      compare, idempotency on the messy fixture, and assert correctly spaced
      input is unchanged (AC1, AC2).
- [x] Run `cargo test`; all existing fixtures and goldens must stay green
      (AC2) — canonical input must be byte-identical to today.
- [x] Update the README (remove/rewrite the limitation) and
      docs/requirements.md (R14); changelog on ship.

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
