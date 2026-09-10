---
type: ChangeRequest
kind: bug
title: Prefer breaking a call chain over wrapping its parameters when PREFER_PARAMETERS_WRAP is off
description: With PREFER_PARAMETERS_WRAP off an overflowing chain keeps the receiver and first call on an over-margin header line and wraps that call's arguments, while PREFER_PARAMETERS_WRAP is never consulted.
state: done
priority: high
tags: [dev, bug, wrapping]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-10T00:00:00Z
---

# Problem

With `PREFER_PARAMETERS_WRAP` off (the built-in default — "prefer breaking the
chain over wrapping parameters") an overflowing method-call chain produces the
worst of both layouts: the receiver and the first call stay on a header line that
already exceeds the margin, and that call's argument list is then wrapped anyway.

In `crates/core/src/formatter.rs` the chain renderer `fmt_chain_ac` keeps the
receiver and the first call on the header line unless
`WRAP_FIRST_METHOD_IN_CALL_CHAIN` is set, and the chain-link argument wrapping
added by `chain-link-args-and-lparen-layout` measures the argument list's fit at
that (over-margin) column, so it wraps the arguments. `PREFER_PARAMETERS_WRAP` is
not consulted at all: its branch in `method_inv_ac` inspects only the _outermost_
invocation's arguments, which in a chain is the last call's — so `true` and
`false` produce byte-identical output for the same source.

# Reproduction

Style: `RIGHT_MARGIN=80`, `METHOD_CALL_CHAIN_WRAP=5` (chop down if long),
`CALL_PARAMETERS_WRAP=5`, `CALL_PARAMETERS_LPAREN_ON_NEXT_LINE=true`,
`CALL_PARAMETERS_RPAREN_ON_NEXT_LINE=true`, `CONTINUATION_INDENT_SIZE=4`,
`PREFER_PARAMETERS_WRAP` absent (off).

```java
class A {
    private static final ObjectMapper NODE_TYPE_MAPPER = new ObjectMapper().registerModule(new Jdk8Module()).disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS);
}
```

Observed:

```java
class A {
    private static final ObjectMapper NODE_TYPE_MAPPER = new ObjectMapper().registerModule(
        new Jdk8Module()
    )
        .disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS);
}
```

The header line is 91 columns (over the 80 margin) _before_ any argument, and the
arguments are wrapped under it. Setting `PREFER_PARAMETERS_WRAP=true` gives the
same bytes. Expected (and what `WRAP_FIRST_METHOD_IN_CALL_CHAIN=true` already
produces):

```java
class A {
    private static final ObjectMapper NODE_TYPE_MAPPER = new ObjectMapper()
        .registerModule(new Jdk8Module())
        .disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS);
}
```

# Proposal

Make the chain renderer prefer breaking the chain when `PREFER_PARAMETERS_WRAP`
is off: in that mode, when the receiver plus the first call (with its flat
argument list) does not fit on the header line, the first call moves to its own
continuation line instead of its arguments wrapping under an over-margin header.
A link's arguments still wrap when they do not fit on the link's own line, so the
earlier "`CALL_PARAMETERS_WRAP` in a chain" fix is preserved.

With `PREFER_PARAMETERS_WRAP` on the current behaviour is kept: the receiver and
first call stay on the header line and that call's arguments wrap, so the option
now actually distinguishes the two layouts.

# Decisions

1. **Bug, not an option question** (agreed with the user on 2026-09-10):
   `PREFER_PARAMETERS_WRAP` is documented as the precedence switch and currently
   has no effect on a chain's overflowing call.
2. **Chain preferred when the option is off** (agreed with the user): the
   receiver + first call moves to its own line when they cannot fit, even with
   `WRAP_FIRST_METHOD_IN_CALL_CHAIN` off — an over-margin header line is never
   acceptable. It only moves when doing so actually resolves the overflow (the
   first call fits on its own line); otherwise the arguments wrap as before.
3. **The option switches the two layouts** (agreed with the user): off → break
   the chain (moving the first call when needed); on → keep the first call on the
   header line and wrap its arguments.
4. **`WRAP_FIRST_METHOD_IN_CALL_CHAIN` keeps its meaning** (agreed with the
   user): when on, the first call always starts its own line; when off, it only
   does so when the chain-wrap layout needs it (decision 2).
5. **The chain-link argument wrapping stays** — a link whose arguments do not fit
   on its own line still wraps (`chain-link-args-and-lparen-layout`), so a chain
   whose arguments are genuinely longer than a line is still wrapped rather than
   left over margin.
6. **Known residual**: with `PREFER_PARAMETERS_WRAP` on, when the receiver plus
   the first call's _prefix_ alone already exceeds the margin (as in the
   reproduction), wrapping the arguments cannot make the header line fit, so the
   header remains over margin. Moving the first call in that case regardless of
   the option is a possible future refinement, deliberately not taken here to
   keep the option's two layouts distinct.

# Acceptance criteria

- With `PREFER_PARAMETERS_WRAP` off, the reproduction emits the receiver on the
  header line and the first call on its own continuation line, with flat
  arguments (byte-equal to the `WRAP_FIRST_METHOD_IN_CALL_CHAIN=true` layout),
  and no line exceeds the margin.
- With `PREFER_PARAMETERS_WRAP` on, the layout keeps the first call on the header
  line and wraps its arguments, so the two option states now differ.
- A chain link whose argument list does not fit on its own line still wraps
  (the earlier chain-argument fix is intact).
- `WRAP_FIRST_METHOD_IN_CALL_CHAIN=true` continues to put the first call on its
  own line unconditionally.
- Existing goldens that pinned the old layout are corrected; new golden cases
  cover `PREFER_PARAMETERS_WRAP` off (chain break) and on (argument wrap) for an
  overflowing chain.
- `cargo test --workspace` is green with
  `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
  `cargo fmt --all -- --check` clean.
- The README chain-wrapping note documents the precedence; a changelog entry is
  appended on delivery (`fawi-implement`).

# Implementation plan

## Approach

The change is confined to `crates/core/src/formatter.rs` (`fmt_chain_ac` and a
small helper) plus the goldens and docs.

- Add `first_call_should_wrap(base, cont, links, c, gap) -> bool`: returns
  `true` when the header line (`base . name(args)`) overflows the margin at `c`
  _and_ the call fits on its own `cont`-prefixed line — i.e. breaking the chain
  resolves the overflow. It uses the same `fits` test the flat rendering uses.
- In `fmt_chain_ac`'s generic branch, extend `first_next` to
  `!base.is_empty() && (wrap_first_method_in_call_chain || (!prefer_parameters_wrap && first_call_should_wrap(base, &cont, links, c, gap)))`.
  When it is set, the existing `first_next` arm already renders the first call on
  a `cont`-prefixed line and sets `link_pref` to `cont`, so no other layout code
  changes.
- The builder branch already starts every call on its own line and is unchanged.
- **Tests.** New golden pair(s) under `tests/java/prefer_parameters_wrap/` (or
  `method_call_chain_wrap/`) for an overflowing chain with prefer off (chain
  break) and on (argument wrap); correct any existing golden that pinned the old
  layout. Per `.agents/AGENTS.md` every test is a golden pair, no inline Java.
- **Docs.** `README.md`'s chain-wrapping note gains the precedence rule;
  `docs/dev/changelog.md` gets an entry on delivery.

## Steps

- [x] Add `first_call_should_wrap` and extend `first_next` in `fmt_chain_ac`.
- [x] Correct existing goldens that pinned the old layout and add the new
      prefer-off / prefer-on chain cases.
- [x] Run `cargo test --workspace` — green, with only the intended goldens moved.
- [x] Run `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
      `cargo fmt --all -- --check`.
- [x] Update the README chain-wrapping note and append a `docs/dev/changelog.md`
      entry.
- [x] Mark the request `done` with `verified`, and set its backlog index row to
      `done`.

## Closing

Shipped on 2026-09-10. The helper ended up as `first_call_should_wrap`, which
requires both that the header line overflow _and_ that the call fit on its own
line, so the chain is only broken when that resolves the overflow (the
reproduction and the tight-margin chain fixtures do; a short receiver with an
unbreakable argument list does not, and keeps the arguments wrapping instead of
breaking the chain pointlessly). Only two goldens pinned the old over-margin
header (`method_call_chain_wrap/chain`, `builder_methods/chain_plain`); they were
regenerated with the CLI, and a `first_call_long` prefer-off / prefer-on golden
pair was added to `PREFER_PARAMETERS_WRAP` showing the two states now differ.
Verified with `cargo test --workspace` (817 core integration tests — two new —
plus nine core unit tests and six GUI tests), `cargo clippy --workspace --lib
--bins --tests -- -D warnings` (which caught a `needless_borrow` on `gap`), and
`cargo fmt --all -- --check`. No commit was made as part of this work.
