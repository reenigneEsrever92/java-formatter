---
type: ChangeRequest
kind: bug
title: The permits clause of sealed classes and interfaces vanishes when formatting
description: format_java drops the `permits` clause of sealed class and interface declarations — `public sealed interface T permits A, B {` formats to `public sealed interface T {` — silently turning valid Java into un-compilable output, violating the never-corrupt contract.
state: proposed
priority: high
tags: [dev, bug, type-declaration]
owner: maintainer
---

# Problem

Formatting a sealed class or sealed interface silently drops its `permits`
clause. `public sealed interface TestitemTaskType permits
TestitemFilterCriteriaType, CustomTestitemType { … }` formats to
`public sealed interface TestitemTaskType { … }`, and a sealed class loses its
`permits` list identically (`public sealed abstract class Shape permits Circle,
Square { … }` → `public sealed abstract class Shape { … }`).

This is silent data loss: the output no longer compiles (a sealed type without
its permitted subtypes declared is invalid Java), and the drop happens without
any warning. It also breaks the never-corrupt contract documented in
`README.md` ("anything the formatter does not model — valid or not — is
preserved verbatim, never dropped or invented"). The `sealed` / `non-sealed`
modifiers survive; only the `permits` clause is lost.

Root cause: the type-declaration header renderers in
`crates/core/src/formatter.rs` — `class_decl` (~L2920) and `iface_decl`
(~L2994) — rebuild the header from the fields they know (`name`,
`type_parameters`, `superclass` / `interfaces`, `extends_interfaces`) and never
render the `permits` field. The tree-sitter-java 0.23.5 grammar parses
`permits` correctly (`class_declaration` and `interface_declaration` both carry
`optional(field('permits', $.permits))`, shape `'permits'` + `type_list`), so
no parse error or missing token is reported and no `ParseDiagnostic` fires —
the clause simply never reaches the output.

# Reproduction

Style: default (`JavaStyle::default()`).

Input:

```java
public sealed interface TestitemTaskType permits TestitemFilterCriteriaType, CustomTestitemType {
    void run();
}
```

Observed output:

```java
public sealed interface TestitemTaskType {
    void run();
}
```

Expected output — the `permits` clause preserved on the header line:

```java
public sealed interface TestitemTaskType permits TestitemFilterCriteriaType, CustomTestitemType {
    void run();
}
```

The same drop happens for a sealed class (`public sealed abstract class Shape
permits Circle, Square { … }` → the `permits` list disappears) and for nested
sealed types inside a type body (they route through the same renderers).

# Proposal

Render the `permits` field in both `class_decl` and `iface_decl`, placed after
the interfaces clause in source order (name → type parameters → superclass /
`extends` → `implements` / `extends_interfaces` → `permits` → body). Reuse the
existing clause-list machinery, `append_type_clause` with keyword `permits`:
the clause has exactly the shape the helper already models (`keyword` +
`type_list`), so it renders flat ` permits A, B` under the defaults and wraps
per the same scheme options as `implements` — `EXTENDS_LIST_WRAP`,
`EXTENDS_KEYWORD_WRAP`, `ALIGN_MULTILINE_EXTENDS_LIST` — which is also how
these schemes treat the clause on devices without a dedicated permits option.
The `flat_type_list` verbatim fallback adds `permits` to the keyword prefixes
it strips, so the degenerate (no-`type_list`) path cannot double the keyword.

`simple_class_one_line` and the modifier-inline paths consume the header built
by the `tail_with` closures, so adding the clause inside `tail_with` covers
the one-line collapse and the annotation-inline layouts too.

Tests: a `sealed_permits` golden-pair suite pinning the preservation baseline
(default style; interface and class; with and without other header clauses such
as `extends` / `implements` / type parameters), plus sealed `permits` fixtures
folded into the three clause-layout option suites (`extends_list_wrap`,
`extends_keyword_wrap`, `align_multiline_extends_list`) so the wrap behaviour
is pinned per option, including self-goldens (reformatting wrapped output is a
no-op).

# Decisions

1. **Both classes and interfaces** (agreed with the user on 2026-09-11): the
   grammar carries `permits` on both declarations and the same drop reproduces
   for a sealed class, so both `class_decl` and `iface_decl` get the render.
2. **Follow the clause-list machinery, not an always-flat echo** (agreed with
   the user): `append_type_clause(header, "permits", …)` — flat under the
   defaults, and `permits` lists wrap under `EXTENDS_LIST_WRAP` /
   `EXTENDS_KEYWORD_WRAP` / `ALIGN_MULTILINE_EXTENDS_LIST` just like
   `implements` lists. No new options; IntelliJ schemes have no dedicated
   permits option, and this keeps the clause in the same layout family.
3. **Tests for the bug wherever they apply** (agreed with the user): a small
   `sealed_permits` golden-pair suite for the preservation baseline (the
   `comments-in-comma-separated-lists` bug set the precedent for a
   bug-driven fixture suite) plus permits fixtures in the three clause-layout
   option suites, including idempotent self-goldens for wrapped output.
4. **Docs update included** (agreed with the user): `README.md`'s
   "never dropped or invented" note stays true once the bug is fixed (no text
   change required unless the fix surface grows), and
   `docs/settings/common.md`'s clause rows
   (`EXTENDS_LIST_WRAP`, `EXTENDS_KEYWORD_WRAP`,
   `ALIGN_MULTILINE_EXTENDS_LIST`) mention `permits` in their effect column.
   The changelog entry is appended on delivery (`fawi-implement`).
5. **`flat_type_list` strips `permits` too**: the fallback path already strips
   `implements` / `extends` from the clause text; `permits` is added so a
   malformed `permits` node without a usable `type_list` echoes as
   ` permits A, B`, not ` permits permits A, B`.

# Acceptance criteria

- `public sealed interface TestitemTaskType permits TestitemFilterCriteriaType,
  CustomTestitemType { … }` formats with the `permits` clause preserved on the
  header line under the default style (byte-stable, idempotent).
- A sealed class's `permits` clause is preserved identically, and the clause
  survives alongside `extends`, `implements` / `extends_interfaces` and type
  parameters on the same header.
- Nested sealed types inside a class / interface body keep their `permits`
  clause.
- Under an over-margin scheme, a `permits` list wraps per `EXTENDS_LIST_WRAP`
  with `EXTENDS_KEYWORD_WRAP` and `ALIGN_MULTILINE_EXTENDS_LIST` applied,
  matching the `implements` layout for the same scheme; reformatting the
  wrapped output is a no-op.
- The `sealed` / `non-sealed` modifiers and the `permits` keyword continue to
  highlight as keywords (`crates/core/src/highlight.rs` unchanged).
- New goldens: `tests/java/sealed_permits/` golden pairs (interface + class,
  with and without other header clauses, under the default style) and
  `permits` fixtures in `tests/java/extends_list_wrap/`,
  `tests/java/extends_keyword_wrap/` and
  `tests/java/align_multiline_extends_list/`, wired per the existing
  per-option test layout in `tests/options.rs`.
- `cargo test --workspace` is green with `cargo clippy --workspace --lib --bins
  --tests -- -D warnings` and `cargo fmt --all -- --check` clean.
- `docs/settings/common.md` clause rows mention `permits`; a changelog entry is
  appended on delivery (`fawi-implement`).
