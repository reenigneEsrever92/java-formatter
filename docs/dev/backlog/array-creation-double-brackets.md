---
type: ChangeRequest
kind: bug
title: Array creation new T[expr] is rendered with doubled brackets (new T[[expr]])
description: new FormFieldApiDto[0] formats to new FormFieldApiDto[[0]] because array_creation and flat_arr_creation wrap the whole dimensions_expr node in brackets and format it via the text-echo fallback; fix both paths to format only the inner expression.
state: done
verified: { by: maintainer, at: 2026-09-11T00:00:00Z }
priority: high
tags: [dev, bug, formatter]
owner: maintainer
---

# Problem

Array creation expressions with explicit dimension expressions are mangled by
the formatter: `new FormFieldApiDto[0]` is emitted as `new FormFieldApiDto[[0]]`.
The defect is not valid Java, so the formatted output breaks the file it was
applied to — a correctness bug, not cosmetic. It affects every `new T[expr]`
form: one-dimensional (`new int[3]`), multi-dimensional (`new int[2][3]` →
`new int[[2]][[3]]`), in any context (field initialiser, local initialiser,
sub-expression such as `new int[1].length`). Only the explicit-dimension form
close to the type is affected (`new int[] { … }` with an initializer is fine).

# Reproduction

Given:

```java
class Foo {
    int[] a = new int[3];
    int[][] b = new int[2][3];
    FormFieldApiDto[] c = new FormFieldApiDto[0];
}
```

`java-formatter Foo.java` emits (observed against the current build):

```java
class Foo {
    int[] a = new int[[3]];
    int[][] b = new int[[2]][[3]];
    FormFieldApiDto[] c = new FormFieldApiDto[[0]];
}
```

Expected: the dimension expressions keep exactly one bracket pair per

dimension — `new int[3]`, `new int[2][3]`, `new FormFieldApiDto[0]` — while
whitespace/layout around them follows the style, as before.

# Proposal

Fix the two render paths in `crates/core/src/formatter.rs` that build the
dimension list of an `array_creation_expression`:

- `array_creation` (line ~7944, the column-aware / wrapped path) and
- `flat_arr_creation` (line ~8723, the one-line path).

Both collect the node's `dimensions_expr` / `dimensions` children and, for a
`dimensions_expr`, currently do `format!("[{}]", self.expr(n))` / `self.flat(n)`
— formatting the **whole `dimensions_expr` node** (`[0]`), which hits the
`_ => self.txt(node)` fallback in `expr_ac` and echoes the node's source text
including its own brackets, hence `[[0]]`. The fix formats the **inner
expression child** of the `dimensions_expr` instead:

```rust
let inner = n.named_children(&mut n.walk()).last();
format!("[{}]", inner.map_or_else(|| self.txt(n).to_string(), |e| self.expr(e, indent, c)))
```

(The last named child of `dimensions_expr` is its `expression`; annotations are
legal in a dimension and precede the expression.) The `dimensions` node branch
(`new int[] { … }` — `[]` echoed as `txt`) is already correct and untouched.

# Decisions

- **Fix both render paths (user choice).** The same defect exists in
  `array_creation` and `flat_arr_creation`; both are reachable (the wrapped
  path is the one exercised by the reproduction), so both get the
  inner-expression fix. Patch-the-symptom (special-casing `dimensions_expr` in
  `expr_ac`) was rejected: `dimensions_expr` is not an expression, it is a
  structural wrapper, and its dimensions belong to the array-creation
  rendering.
- **Format the inner expression, not the node (root-cause fix).** Rendering
  the `dimensions_expr`'s own `expression` child restores the pre-existing
  whitespace/layout behaviour of the dimension expression itself (spacing,
  wrapping options apply to it as before) and exactly one bracket pair per
  dimension. The `dimensions` branch (empty `[]` pairs for `new int[] { … }`)
  is untouched.
- **Regression test under the per-option-style golden suite (user choice).**
  Add a dedicated `array_creation` module in `crates/core/tests/options/`
  registered in `options.rs`, with fixtures under
  `crates/core/tests/java/array_creation/` golden pairs (input `.java` +
  expected `.out.java`), covering the reported case and the full shape space:
  one dimension with a literal `[0]`, multi-dimension `[2][3]`, a dimension
  driven by an expression (`new int[size]`), `new int[] { … }` (the unaffected
  initializer form, as a guard), and a re-format no-op (R6 idempotency).
  No existing golden changes: the suite has no `new T[expr]` fixtures today
  (searched `crates/core/tests/**` for `new [A-Za-z_]+[0-9]`).
- **No doc changes.** No doc under `docs/` describes array-creation dimension
  rendering (R-count rows, settings reference, README behavioural notes), so
  this fix changes no documented behaviour beyond correcting the erroneous
  output; only the changelog records the fix.

# Acceptance criteria

- `java-formatter` on the reproduction input emits exactly
  `new int[3]`, `new int[2][3]` and `new FormFieldApiDto[0]` (one bracket pair
  per dimension, inner expression intact) and valid Java in every context
  (field initialiser, local initialiser, `new int[1].length` sub-expression).
- Multi-dimensional creation renders `new int[2][3]`, not `new int[[2]][[3]]`.
- `new int[] { … }` with an array initializer keeps today's output
  byte-for-byte (the `dimensions` branch is untouched).
- The new `array_creation` golden tests pass (input → expected output, plus a
  re-format no-op assertion), no pre-existing golden changes anywhere.
- `cargo test --workspace` (817 + new goldens), `cargo clippy --workspace
--lib --bins --tests -- -D warnings`, and `cargo fmt --all -- --check` are
  green; the changelog records the fix.

# Implementation plan

## Approach

### 1. Fix `array_creation` (`crates/core/src/formatter.rs`, ~L7944)

The wrapped/column-aware path currently builds each dimension with
`format!("[{}]", self.expr(n, indent, c))` where `n` is the **`dimensions_expr`
node itself**; `expr` falls through to `txt(n)` and echoes `[0]` including its
brackets. Replace the mapping with inner-expression extraction and a running
column, so margin/wrap decisions inside a dimension (e.g. `new int[a +
b]` under a tight margin) use the true column after the `[`:

```rust
let mut dims_col = c + self.col_after(c, &format!("new {}", ty));
let dims: Vec<_> = self
    .named(node)
    .into_iter()
    .filter(|n| matches!(n.kind(), "dimensions_expr" | "dimensions"))
    .map(|n| {
        let s = if n.kind() == "dimensions_expr" {
            let inner =
                self.named(n).pop().expect("dimensions_expr has an expression child");
            format!("[{}]", self.expr(inner, indent, dims_col + 1))
        } else {
            self.txt(n).to_string()
        };
        dims_col = self.col_after(dims_col, &s);
        s
    })
    .collect();
```

`self.named(n)` returns named children in source order and
`dimensions_expr: seq(repeat($._annotation), '[', $.expression, ']')` has the
`expression` as its last named child, so `pop()` yields it; the `dimensions`
branch (`new int[] { … }`) keeps `txt(n)` unchanged. The later `brace_col`
computation for the initialiser already recomputes the column from
`format!("new {}{}", ty, dims.join(""))` and needs no change.

### 2. Fix `flat_arr_creation` (`crates/core/src/formatter.rs`, ~L8723)

The one-line path has the identical defect with `self.flat(n)`. Same
inner-expression extraction, using `self.flat(inner)` (literals and simple
sub-expressions are handled by `flat`'s fallback `txt`):

```rust
if n.kind() == "dimensions_expr" {
    let inner =
        self.named(n).pop().expect("dimensions_expr has an expression child");
    format!("[{}]", self.flat(inner))
} else {
    self.txt(n).to_string()
}
```

### 3. Regression tests (per-option golden suite)

- New fixtures under `crates/core/tests/java/array_creation/`:
  - `dims.java` / `dims.out.java` — the reported shape space: `new int[3]`,
    `new int[2][3]`, `new FormFieldApiDto[0]`, `new int[1].length`
    (sub-expression context), and an expression-driven dimension
    (`new int[count]`); expected output keeps exactly one bracket pair per
    dimension, only whitespace/layout changes.
  - `init.java` / `init.out.java` — the unaffected `new int[] { … }`
    initializer form as a guard that the `dimensions` branch stays byte-identical.
- New module `crates/core/tests/options/array_creation.rs` following the
  per-option pattern (`use super::common::*;`, `include_str!` the pair,
  `assert_eq!(format(DIMS), DIMS_OUT)`), registered alphabetically in
  `crates/core/tests/options.rs` between `annotation_parameter_wrap` and
  `array_element_indent`; include both fixtures and an idempotency
  assertion (`format(DIMS_OUT) == DIMS_OUT`, R6).
- The goldens are produced by the fixed implementation and reviewed by hand
  against the expected shapes above; no pre-existing golden changes (the
  suite has no `new T[expr]` fixtures).

### 4. Docs

The bug report records no doc changes (no doc describes array-creation
rendering); only the changelog notes the fix on delivery.

## Steps

- [x] Fix the `dimensions_expr` mapping in `array_creation` (inner-expression
      extraction + running column) in `crates/core/src/formatter.rs`.
- [x] Fix the `dimensions_expr` mapping in `flat_arr_creation` (inner-expression
      extraction + `flat`) in `crates/core/src/formatter.rs`.
- [x] Add `crates/core/tests/java/array_creation/{dims,init}.java` and
      `{dims,init}.out.java` golden pairs (the second as the `new int[] { … }`
      guard), generated from the fixed implementation and hand-verified.
- [x] Add `crates/core/tests/options/array_creation.rs` (golden assertions +
      idempotency) and register it in `crates/core/tests/options.rs`.
- [x] Re-run the reproduction input and confirm `new int[3]` / `new int[2][3]`
      / `new FormFieldApiDto[0]`; run `cargo test --workspace` (817 + new
      goldens), `cargo clippy --workspace --lib --bins --tests -- -D warnings`,
      `cargo fmt --all -- --check`; append the changelog entry and close the
      request.

## Closing

Reproduction fixed (verified against the built binary): `new int[3]`,
`new int[2][3]`, `new FormFieldApiDto[0]`, `new int[1].length` and
`new int[size]` all render with exactly one bracket pair per dimension; the
`new int[] { … }` initializer form is byte-identical. Three new golden tests
(`array_creation`) registered in `options.rs`; `cargo test --workspace` is
green (820 core goldens — 817 + 3 new — plus 15 cli and 6 GUI tests),
clippy `-D warnings` and `cargo fmt --check` clean. Changes uncommitted in
the worktree (no commit hash); changelog entry added 2026-09-11.
