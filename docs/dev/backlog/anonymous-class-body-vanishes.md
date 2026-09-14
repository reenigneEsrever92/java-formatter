---
type: ChangeRequest
kind: bug
title: An anonymous class body vanishes when the creation is rendered through the flat path
description: The flat renderers resolve an anonymous body with a field-name lookup that never matches, so `new X() { … }` in any flat-rendered position (call / `new` arguments, chain links, ternary, binary, lambda body) silently drops the whole implementation block — `stream.map(new Function<String, String>() { … })` formats to `stream.map(new Function<String, String>())`.
state: done
priority: high
tags: [dev, bug, class]
owner: maintainer
verified: { by: Zed coding agent, at: 2026-09-14T12:44:33Z }
---

# Problem

Formatting a `new` expression with an anonymous class body silently drops the
body whenever that expression is rendered through the flat (one-line) path. The
reported case is an anonymous class passed as a method-call argument:
`stream.map(new Function<String, String>() { @Override … })` formats to
`stream.map(new Function<String, String>())` — the whole implementation block is
gone, so the call instantiates the base type instead of the anonymous subclass
and the output no longer compiles or behaves as the input.

The same loss happens in every context that renders the creation flat: another
argument of a call or `new`, a chain receiver, a ternary side, a binary operand,
and a lambda body. Only contexts that go through `expr_ac` → `new_expr` — a
field or local-variable initialiser, a `return`, an array-initialiser element
and a bare expression statement — keep the body. This is silent data loss with
no `ParseDiagnostic`: the input parses cleanly, so the never-corrupt contract
(R4: anything the formatter does not model is preserved verbatim, never dropped
or invented) and semantic equivalence (R5) are both violated.

Root cause: the two flat renderers in `crates/core/src/formatter.rs` —
`flat_new` (`~L10482`, body lookup `~L10507`) and `flat_new_chain` (`~L8677`,
body lookup `~L8700`) — resolve the anonymous body with
`self.fld(node, "class_body")`, but `object_creation_expression` exposes the body
as an **unnamed positional child**; `new_expr` (`~L9242`) knows this and finds it
with `self.all_ch(node).find(|c| c.kind() == "class_body")`. `fld` is
`child_by_field_name`, so the lookup is always `None`, the body (and the
`{ ... }` margin placeholder that was meant to keep the fit estimate honest) is
never emitted, and the resulting short "flat" text passes every `fits` check, so
`method_inv_ac` (`~L8494`), `args_wrapped` (`~L8787`, `~L8807`), `ternary_ac`
(`~L9648`) and `binary_operand` (`~L9538`) emit it as the final output.

This is the same damage class as the shipped
[lambda body flat join](lambda-body-flat-join.md) bug: an unflattenable body in
flat / one-line position. That fix already built the machinery that should have
caught this — `flat_unflattenable` (`~L10572`) and the `flat_ok` /
`contains('\n')` guards that stop a multi-line "flat" string from being emitted,
plus the "single unflattenable argument glued after `(`" branch in `args_wrapped`
(`~L8819`) — but it only applies once the construct _reports_ itself as
unflattenable. `flat_new` never does, because it never sees the body.

# Reproduction

Style: default (`JavaStyle::default()`).

Input:

```java
import java.util.function.Function;

class Test {
    void m() {
        stream.map(new Function<String, String>() {
            @Override
            public String apply(String s) {
                return s;
            }
        });
    }
}
```

Observed output — the implementation block is dropped:

```java
import java.util.function.Function;

class Test {
    void m() {
        stream.map(new Function<String, String>());
    }
}
```

Expected output — the body is kept, glued on the call line (as IntelliJ lays it
out):

```java
import java.util.function.Function;

class Test {
    void m() {
        stream.map(new Function<String, String>() {
            @Override
            public String apply(String s) {
                return s;
            }
        });
    }
}
```

Under the default style the body is lost in every flat-rendered position
(`{ … }` stands for the whole implementation block):

| Input (as an expression)                   | Observed                             |
| ------------------------------------------ | ------------------------------------ |
| `submit(new Runnable() { … })`             | `submit(new Runnable())`             |
| `submit(1, new Runnable() { … }, 2)`       | `submit(1, new Runnable(), 2)`       |
| `new Outer(new Runnable() { … })`          | `new Outer(new Runnable())`          |
| `new Thread(new Runnable() { … }).start()` | `new Thread(new Runnable()).start()` |
| `b ? new Runnable() { … } : null`          | `b ? new Runnable() : null`          |
| `flag == new Runnable() { … }`             | `flag == new Runnable()`             |
| `() -> new Runnable() { … }`               | `() -> new Runnable()`               |

The following contexts already keep the body and must stay byte-identical:
a field initialiser, a local-variable initialiser, a `return`, an
array-initialiser element, and a bare expression statement
(`new Runnable() { … };`).

# Proposal

Make every flat rendering of an anonymous class preserve the body, so the
never-corrupt contract holds no matter where the creation appears. The flat
renderers stop claiming a body-free one-line form they cannot produce:

- `flat_new` and `flat_new_chain` find the `class_body` child positionally
  (exactly as `new_expr` does) and, when it is present, return the node's source
  text verbatim — the R4 fallback `flat_block` already uses for a block with no
  safe one-line form. The returned text is multi-line, so the existing
  `flat_ok` / `contains('\n')` guards in `method_inv_ac`, `args_wrapped`,
  `flat_inv` and `new_expr` route the construct to its multi-line layout instead
  of emitting the "flat" string; `chain_link_args` (`~L9210`) already carries the
  same guard and needs no change.
- `args_wrapped` widens its "single unflattenable argument glued after `(`"
  branch (currently `lambda_expression`-only, `~L8819`) to also cover an
  `object_creation_expression` with an anonymous body, so a single anonymous
  argument renders glued on the call line. The branch's existing
  `entries.len() == 1` guard keeps a multi-argument list on the generic wrapped
  layout, matching how the lambda case already behaves.
- The contexts that emit a `flat()` result directly rather than through the
  guarded argument renderers — the ternary sides (`ternary_operand` `~L9644`),
  the binary operands (`binary_operand` `~L9538`) and a lambda block body
  (`flat_lambda` `~L10553`) — fall back to the canonical `expr` rendering when
  the flat text is multi-line, so the preserved body is re-indented at its real
  column instead of pasted from the source.

Because `new_expr` already renders the body correctly, every fallback re-uses
it: the multi-line `new Runnable() { … }` text comes from the same renderer the
working initialiser / `return` contexts use.

# Decisions

1. **Fix the whole flat-path class, not just the reported call argument**
   (agreed with the user on 2026-09-14): the call / `new` argument, chain-link,
   ternary, binary and lambda-body losses all share the one root cause — a
   flat renderer dropping the body — and each is an R4/R5 violation, so one fix
   covers them.
2. **A single anonymous-class argument stays glued after `(`** (agreed with the
   user): IntelliJ keeps `stream.map(new Function<>() { … })` on the call line
   with the body multi-line; the wrapped-argument `(\n … \n)` layout is reserved
   for multi-argument lists. This mirrors the shipped lambda branch, whose
   `entries.len() == 1` guard already produces exactly this split.
3. **Verbatim fallback in the flat renderers (R4)** (agreed with the user):
   an anonymous body has no flat form, so `flat_new` / `flat_new_chain` echo the
   node's source rather than inventing a placeholder `{ ... }` or a flattened
   body. This is the same pattern as `flat_block` for unflattenable blocks and
   keeps flat contexts byte-preserving; the multi-line result is what trips the
   existing guards into the canonical layout.
4. **New `anonymous_class_bodies` golden suite** (agreed with the user): the
   regression is a never-corrupt bug with no governing XML option, so it gets
   its own suite mirroring `lambda_body_comments` (the closest precedent, and
   itself a bug-regression suite under `tests/options/`), rather than being
   pinned onto an unrelated option.
5. **Docs are extended, not corrected** (agreed with the user): no existing doc
   states that a flat rendering keeps an anonymous body, so nothing is wrong —
   the README behaviour notes gain the preservation rule and
   `docs/requirements.md` gains an R49 row, with the changelog entry appended on
   delivery.

# Acceptance criteria

- The reproduction above formats with the whole body kept, glued on the call
  line; the output is byte-stable and idempotent (formatting it again is a
  no-op).
- Every context in the reproduction table keeps its body: call argument (single
  and among other arguments), `new` argument, chain receiver, ternary side,
  binary operand and lambda body.
- A single anonymous-class argument glues after `(`; a multi-argument list that
  contains one renders through the wrapped-argument layout with the body intact.
- The contexts that already worked — field initialiser, local-variable
  initialiser, `return`, array-initialiser element, bare expression statement —
  are byte-identical to today (no existing fixture's golden changes;
  `cargo test --workspace` green).
- The preserved body consistently takes `SPACE_BEFORE_CLASS_LBRACE`,
  `BLANK_LINES_AFTER_ANONYMOUS_CLASS_HEADER` and the class-body member options,
  since it is rendered through the shared `new_expr` / `class_body` path.
- New goldens: `tests/java/anonymous_class_bodies/` (single argument, argument
  among others, `new` argument, chain receiver, ternary and binary operand,
  lambda body, plus a working initialiser / `return` control), wired as
  `tests/options/anonymous_class_bodies.rs` and registered in `tests/options.rs`.
- `cargo test --workspace` is green with `cargo clippy --workspace --lib
--bins --tests -- -D warnings` and `cargo fmt --all -- --check` clean.
- The README behaviour notes describe the preservation and the glued
  single-argument layout; `docs/requirements.md` gains the R49 row; a changelog
  entry is appended on delivery (`fawi-implement`).

# Implementation plan

## Approach

All engine changes are confined to `crates/core/src/formatter.rs`, plus the
golden fixtures, the option test file, and the doc updates.

**`anon_body`** (new helper, beside `new_expr` ~L9226): the anonymous
`class_body` child of an `object_creation_expression`, found positionally
(`self.all_ch(node).into_iter().find(|c| c.kind() == "class_body")`).
`new_expr` (~L9242) switches to it, so the lookup has one definition and the
flat renderers stop relying on the field name the grammar never sets.

**`flat_new`** (~L10482) and **`flat_new_chain`** (~L8677): when `anon_body` is
`Some`, return `self.txt(node).to_string()` — the R4 verbatim fallback — instead
of a body-free one-line form; the dead `self.fld(node, "class_body")`
placeholder is removed from both. The result carries line breaks, so the
existing `flat_ok` / `contains('\n')` guards in `method_inv_ac` (~L8494),
`args_wrapped` (~L8771, ~L8787, ~L8807), `flat_inv`, `new_expr` (~L9252) and
`chain_link_args` (~L9210, already guarded) route the construct to its
multi-line layout.

**`args_wrapped`** (~L8819): the "single unflattenable argument glued after
`(`" branch accepts an `object_creation_expression` with an anonymous body in
addition to `lambda_expression`, so a single anonymous-class argument renders
glued on the call line (`stream.map(new Function<>() { … })`); its existing
`entries.len() == 1` guard keeps a multi-argument list on the wrapped layout.

**`flat_or_expr`** (new helper, beside `flat` ~L10311): `self.flat(node)` unless
that text carries a line break, in which case `self.expr(node, indent, c)` — the
canonical rendering at the real column. Used to build the flat strings of
**`ternary_ac`** (~L9548) and **`binary_ac`** (~L9428), so a ternary side or a
binary operand holding an anonymous class keeps the body re-indented
canonically instead of pasted from the source. A lambda body in argument
position needs no change: its multi-line flat text already routes the argument
through the glued branch, which re-renders the lambda through `expr`.

**Tests**: a new `anonymous_class_bodies` suite — fixtures under
`crates/core/tests/java/anonymous_class_bodies/` with the option file
`crates/core/tests/options/anonymous_class_bodies.rs` registered in
`tests/options.rs` — pinning `argument` (the report), `argument_among_others`,
`new_argument`, `chain_receiver`, `ternary`, `binary`, `lambda_body`, and an
`initialiser_control` for the contexts that already worked.

**Docs**: the README behaviour notes gain the anonymous-body preservation rule
and the glued single-argument layout; `docs/requirements.md` gains the R49 row;
the backlog index gains the row; `docs/dev/changelog.md` is appended on
delivery.

## Steps

- [x] Add `anon_body` and switch `new_expr` to it.
- [x] `flat_new` / `flat_new_chain`: echo the source verbatim when an anonymous
      body is present; drop the dead field lookup.
- [x] `args_wrapped`: widen the glued single-unflattenable-argument branch to
      anonymous-class arguments.
- [x] Add `flat_or_expr`; use it in `ternary_ac` and `binary_ac`.
- [x] Add the `anonymous_class_bodies` fixtures and option test file, and
      register it in `tests/options.rs`.
- [x] README behaviour notes: document the preservation and the glued
      single-argument layout.
- [x] `docs/requirements.md`: add the R49 row.
- [x] `docs/dev/backlog/index.md`: add the new row.
- [x] Quality gates: `cargo test --workspace`, `cargo clippy --workspace --lib
  --bins --tests -- -D warnings`, `cargo fmt --all -- --check` — all green.

## Closing

Shipped on 2026-09-14. The flat renderers in `crates/core/src/formatter.rs` no
longer drop an anonymous class body: `anon_body` finds the `class_body` child of
an `object_creation_expression` positionally (the grammar gives it no field name,
which is why the old `self.fld(node, "class_body")` lookup never matched), and
`flat_new` / `flat_new_chain` echo the node's source verbatim (R4) when a body is
present, so the existing `flat_ok` / `contains('\n')` guards route the construct
to the multi-line layout. `args_wrapped` glues a single unflattenable argument —
an anonymous class or a block lambda — after the call's `(`, and the new
`flat_or_expr` helper renders a ternary side or a binary operand holding an
anonymous class through the canonical `expr` rendering. A call / `new` argument,
a chain receiver, a ternary side, a binary operand and a lambda body therefore
all keep the implementation block; field / local initialisers, `return`, array
elements and bare statements were already correct and stay byte-identical. The
new `tests/java/anonymous_class_bodies/` suite (eight golden pairs wired through
`tests/options/anonymous_class_bodies.rs`) pins each context and is idempotent;
the README behaviour notes and `docs/requirements.md` (R49) document it. Verified
with `cargo test --workspace` (901 core integration tests — eight new — plus 9
core unit, 18 CLI and 6 GUI tests), `cargo clippy --workspace --lib --bins
--tests -- -D warnings`, and `cargo fmt --all -- --check`. The changes are
uncommitted in the worktree (no commit hash); the changelog entry was added on
2026-09-14.
