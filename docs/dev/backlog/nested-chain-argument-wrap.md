---
type: ChangeRequest
kind: bug
title: METHOD_CALL_CHAIN_WRAP does not wrap a chain nested in an argument
description: A method-call chain used as an argument (or inside a chain link's argument) is emitted flat and overflows the right margin even with METHOD_CALL_CHAIN_WRAP enabled.
state: done
priority: high
tags: [dev, bug, formatter]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-10T00:00:00Z
---

# Problem

`METHOD_CALL_CHAIN_WRAP` is documented as supported (✅ in
`docs/settings/common.md` "Expressions and statements"; README honoured-options
table) and does wrap the outermost chain into one link per line. But a
method-call chain nested **inside an argument** is emitted flat, so it blows far
past the right margin. Reporter's real-world case (IntelliJ scheme with
`METHOD_CALL_CHAIN_WRAP` active): a builder chain whose links take
`LocalizedText.builder().…build()` arguments, and a `.processTypes(List.of(…))`
link whose elements are themselves long builder chains. The outer chain breaks
per link; every nested chain stays on one line, producing lines hundreds of
columns over the margin.

The formatter appears to ignore the option on exactly the code where chains are
written as arguments — the common builder/DSL shape — which is why this reads as
"METHOD_CALL_CHAIN_WRAP is not working".

# Reproduction

Reduced, with `RIGHT_MARGIN` `60` and `METHOD_CALL_CHAIN_WRAP` `1` (`WrapIfLong`):

```java
class F {
    void m() {
        Config c = Config.builder().name(LocalizedText.builder().text("Hello", Locale.GERMAN).text("World", Locale.ENGLISH).build()).build();
    }
}
```

Observed — the outer chain breaks, the nested `LocalizedText.builder()…build()`
does not, and the link line is far over the 60-column margin:

```java
class F {
    void m() {
        Config c = Config.builder()
                .name(LocalizedText.builder().text("Hello", Locale.GERMAN).text("World", Locale.ENGLISH).build())
                .build();
    }
}
```

Expected — the nested chain also wraps per `METHOD_CALL_CHAIN_WRAP`, e.g.
`LocalizedText.builder()` on the link line with `.text(…)`, `.text(…)`, `.build()`
each on their own continuation line.

Variants confirmed against the current build:

- `use(LocalizedText.builder()…build())` — the outer call is **not** a chain and
  the chain is the **only** argument → the nested chain **does** wrap today
  (the special case in `args_wrapped`).
- `Config.builder().name(LocalizedText.builder()…build()).build()` — outer is a
  chain → nested chain stays flat (the reported case).
- `use(a, LocalizedText.builder()…build())` with `CALL_PARAMETERS_WRAP` `0` —
  multi-argument call → nested chain stays flat (wraps only with
  `CALL_PARAMETERS_WRAP` on).
- `WrapAlways` (code `2`) does not change any of the above: link arguments are
  flat regardless of the wrap code.

# Proposal

Render each chain link's argument list through the same wrapped-argument path the
rest of the formatter uses instead of forcing it flat, and generalise the
argument-chain handling so a chain argument wraps wherever it appears.

Concretely, in `crates/core/src/formatter.rs`: `fmt_chain_ac` builds every link
with `flat_args(link.args)` (both the builder branch ~L7021 and the generic
branch ~L7086), and `flat_args` maps each argument through `flat()`, so nested
constructs can never wrap. Replace that flat rendering with the arguments'
wrapped rendering (`args_wrapped`, already used by `inv_wrapped`) so a link's
arguments follow the normal fit/wrap decision, and generalise the
`args.len() == 1` chain special case in `args_wrapped` (~L6815) so **any**
argument that is itself an over-margin chain is wrapped, not just a lone one.
The change is whitespace/layout only (R5): arguments that fit stay byte-flat and
existing goldens are unchanged (R6).

Docs touched: README formatting-behaviour notes and `docs/dev/changelog.md` on
delivery.

# Decisions

1. **This is a bug, not a feature.** `METHOD_CALL_CHAIN_WRAP` is marked ✅ with
   no recorded nested-chain limitation, so the flat rendering is a defect.
2. **Scope: every argument position, recursively.** Fix the chain-link case the
   reporter hit (`.lastCallText(<chain>)`), and the same gap where it also shows
   up — a chain inside a link's multi-element argument
   (`.processTypes(List.of(<chain>, <chain>))`), a chain in a multi-argument call
   (`foo(a, <chain>)`), and chains nested deeper inside those. Reason: the option
   should behave the same wherever a chain sits; fixing only the lone-argument
   case would leave the reporter's `List.of(…)` shape broken.
3. **Independent of `CALL_PARAMETERS_WRAP`.** An over-margin chain argument
   wraps on its own even when `CALL_PARAMETERS_WRAP` is `DoNotWrap`, rather than
   requiring a second option to be enabled. Reason: IntelliJ wraps nested chains
   independently of the parameter-list wrap option, and the reporter's scheme
   should not have to turn on call-parameter wrapping to get the chain option to
   take effect.
4. **The nested chain composes with the other chain options.** The wrapped
   nested chain goes through the same `fmt_chain` path, so
   `ALIGN_MULTILINE_CHAINED_METHODS`, `WRAP_FIRST_METHOD_IN_CALL_CHAIN`,
   `BUILDER_METHODS` / `KEEP_BUILDER_METHODS_INDENTS` and `CHAINED_CALL_INDENT`
   apply to it as they do to a top-level chain.
5. **Fitting chains stay flat.** The fit check is unchanged, so only chains that
   overflow the margin change; `DoNotWrap` and absent schemes, and every
   pre-existing golden, stay byte-identical (R5, R6).
6. **Priority high.** The reporter's normal source (builder config objects) is
   badly over-margin, which defeats the option for common real code.

# Acceptance criteria

- The reproduction above (a chain link whose single argument is an over-margin
  chain) now wraps the nested chain per `METHOD_CALL_CHAIN_WRAP`, and the result
  re-formats to itself (R6).
- A multi-argument call with an over-margin chain argument wraps that chain even
  when `CALL_PARAMETERS_WRAP` is `DoNotWrap`.
- New tests in `crates/core/tests/options/method_call_chain_wrap.rs` (fixtures
  under `crates/core/tests/java/method_call_chain_wrap/`) cover: a nested
  single-argument chain in a link; a nested chain inside a multi-element
  argument (`List.of(<chain>, <chain>)`); a plain multi-argument call with a
  chain argument; a `DoNotWrap` case keeping everything flat; and an idempotency
  self-golden.
- Absent/default schemes and every pre-existing golden are unchanged;
  `cargo test --workspace` stays green; `cargo clippy --workspace --lib --bins
  --tests -- -D warnings` and `cargo fmt --all -- --check` stay clean.
- The README formatting-behaviour notes (and, if the wrapping is described
  there, `docs/settings/common.md`) describe the nested-chain behaviour, and a
  changelog entry is appended on delivery (fawi-implement).

# Implementation plan

## Approach

All rendering changes are in `crates/core/src/formatter.rs`; tests and fixtures
under `crates/core/tests/`. The flat path flattens recursively (`flat` →
`flat_inv` → `flat_args`), so anything swallowed by an argument can never wrap;
the fix adds a parallel, chain-aware renderer used where arguments are emitted
in an otherwise-flat context.

**Chain-aware argument rendering.** Add next to `flat_inv` / `flat_args`
(~L6711-6758):

- `flat_arg_chain(arg, indent, c)` — for a `method_invocation` argument, run
  `collect_chain(arg)`; when `links.len() >= 2`, `method_call_chain_wrap` is not
  `DoNotWrap` and `!fits(c, flat(arg))`, return
  `fmt_chain(&base, &links, indent, c, is_builder_chain(&links))`; when it is an
  invocation or an `object_creation_expression`, recurse through
  `flat_inv_chain` / `flat_new_chain`; otherwise return `flat(arg)`.
- `flat_args_chain(args_node, indent, c)` — mirrors `flat_args` (same
  `within_opt` paren/space handling and comma separator) but joins
  `flat_arg_chain` per argument and tracks the running column, so each
  argument's fit check uses its real start column and a multi-line argument
  continues from its last line.
- `flat_inv_chain(node, indent, c)` / `flat_new_chain(node, indent, c)` — the
  `flat_inv` / `flat_new` shapes with `flat_args_chain` supplying the argument
  list.

**Wire-up.**

- `fmt_chain_ac` (builder branch ~L7025, generic branch ~L7086): replace
  `flat_args(link.args)` with `flat_args_chain(link.args, indent,
  link_args_col)`, where `link_args_col` is the emitted link line's `(` column
  (link-line prefix + `.` + type args + name + gap).
- `args_wrapped` (~L6808-6831): in the `wrap == DoNotWrap` branch return
  `flat_args_chain(node, indent, c)` instead of `flat`, so an over-margin chain
  argument wraps with `CALL_PARAMETERS_WRAP` off (Decision 3); the existing
  single-argument-chain special case above it stays (its layout is the
  one-argument case of the same rule), which keeps the existing non-chain-outer
  golden unchanged.
- `new_expr` (~L7159-7169) already routes to `args_wrapped` once the flat
  creation does not fit, so `new Foo(<chain>)` is covered; its flat form stays
  the true flat form for the fit check.

**Fixed point under `KEEP_LINE_BREAKS`.** Wrapping a nested chain puts a
depth-0 line break inside the argument list, so on re-format
`has_join_break(arguments)` is true and `keep_wrapped` flips, which today sends
`args_wrapped` down its parameter-list-wrap path even under `DoNotWrap` and
would re-flow the chain (R6). The chain-aware rendering must therefore be the
layout produced on the second pass too: `fmt_chain_ac` renders link arguments
with the pure `flat_args_chain` (independent of `keep`), and `args_wrapped` must
keep the chain-aware rendering ahead of the `keep` gate (or otherwise make the
parameter-list path a fixed point for the chain-wrapped shape). This is pinned
by the idempotency self-goldens below.

**Scope guard.** Only over-margin chains change: `flat_args_chain` is
byte-identical to `flat_args` when no argument is an over-margin chain,
`DoNotWrap` and absent schemes keep everything flat, and the fit checks are
untouched, so every pre-existing golden must stay green (R5, R6).

Docs: README formatting-behaviour notes; changelog on delivery.

## Steps

- [x] Add `flat_arg_chain` / `flat_args_chain` / `flat_inv_chain` /
      `flat_new_chain` to `crates/core/src/formatter.rs` next to
      `flat_inv`/`flat_args`, recursing through invocation and creation
      arguments. (AC1, AC2)
- [x] Route `fmt_chain_ac`'s two `flat_args(link.args)` sites through
      `flat_args_chain` with the link's argument column. (AC1)
- [x] Make `args_wrapped`'s `DoNotWrap` branch return `flat_args_chain`, and
      confirm `new_expr` inherits it. (AC2)
- [x] Reconcile the `keep_wrapped` / fixed-point behaviour so a chain-wrapped
      argument list re-formats to itself. (AC1, AC3)
- [x] Add fixtures and tests in
      `crates/core/tests/options/method_call_chain_wrap.rs` plus
      `crates/core/tests/java/method_call_chain_wrap/`: a link single-argument
      chain, `List.of(<chain>, <chain>)` nested in a link, a plain
      multi-argument call with a chain argument under `CALL_PARAMETERS_WRAP`
      `DoNotWrap`, an all-flat `DoNotWrap` case, and an idempotency self-golden.
      (AC1-AC4)
- [x] Run `cargo test --workspace` and confirm no pre-existing golden changed;
      run `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
      `cargo fmt --all -- --check`. (AC4)
- [x] Update the README formatting-behaviour notes and append a
      `docs/dev/changelog.md` entry. (AC5)

## Closing

Shipped on 2026-09-10; the changelog entry was added the same day. Verified
with `cargo test --workspace` (810 option tests plus the four `highlight` and
two GUI tests), `cargo clippy --workspace --lib --bins --tests -- -D warnings`
and `cargo fmt --all -- --check`; no pre-existing golden changed. The
reproduction from the request now wraps both the outer and the nested chains
and re-formats to itself. No commit was made as part of this work.
