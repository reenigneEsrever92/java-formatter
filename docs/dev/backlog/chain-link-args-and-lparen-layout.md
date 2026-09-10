---
type: ChangeRequest
kind: bug
title: Wrap chain-link arguments and fix the lparen-stays / rparen-alone indentation
description: A chain link's argument list ignores CALL_PARAMETERS_WRAP and stays flat, and the lparen-stays / rparen-alone layout prefixes the first element with the continuation indent instead of gluing it after '('.
state: done
priority: high
tags: [dev, bug, wrapping]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-10T00:00:00Z
---

# Problem

Two defects in `crates/core/src/formatter.rs` produce incorrect argument-list
layout.

1. **`CALL_PARAMETERS_WRAP` is ignored for a method-call chain's link
   arguments.** `fmt_chain_ac` renders every link's argument list with
   `flat_args_chain`, which is flat and only breaks a _nested chain_; it never
   runs the wrapped layout (`args_wrapped`) where `CALL_PARAMETERS_WRAP` and
   `CALL_PARAMETER_INDENT` are applied. A plain call wraps correctly, but the
   same call as a chain link stays on one line even when it overflows the
   margin.
2. **The lparen-stays / rparen-alone layout injects indentation.**
   `CALL_PARAMETERS_LPAREN_ON_NEXT_LINE=false` with
   `CALL_PARAMETERS_RPAREN_ON_NEXT_LINE=true` is the "first element stays on the
   header line after `(`" arm. When the matching align option is off, that arm
   still prefixes the first element with the continuation indent, so the header
   line reads `alpha(` followed by the indent's worth of spaces and the first
   argument, instead of `alpha(` + the argument.

Impact: formatted output that is wrong (over-margin lines that should have
wrapped) and visibly malformed (`(` + stray indentation), in both cases only
reaching an existing option through a path that did not honour it. The
declaration (`formal_params`) and resource-list (`resource_list`) renderers carry
the same lparen-stays defect, but their align options (`ALIGN_MULTILINE_PARAMETERS`,
`ALIGN_MULTILINE_RESOURCES`) default **on**, which masks it; calls expose it
because `ALIGN_MULTILINE_PARAMETERS_IN_CALLS` defaults off. Worse, the current
behaviour is **pinned by six golden fixtures**, so the suite currently asserts
the bug.

# Reproduction

**Defect 1.** Style: `RIGHT_MARGIN=80`, `CALL_PARAMETERS_WRAP=1`,
`METHOD_CALL_CHAIN_WRAP=1`; source is a single long builder chain:

```java
ConfigJson.ProcessTypeConfig.builder().lastCallText(LocalizedText.builder().text("Last Call", Locale.GERMAN).build()).samplingTypes(List.of(ConfigJson.SamplingType.DEFAULT, ConfigJson.SamplingType.RESAMPLING, ConfigJson.SamplingType.LAST_CALL)).build();
```

Observed: the chain breaks one link per line, but the link arguments stay flat
and overrun the margin:

```java
        ConfigJson.ProcessTypeConfig.builder()
                .lastCallText(LocalizedText.builder()
                    .text("Last Call", Locale.GERMAN)
                    .build())
                .samplingTypes(List.of(ConfigJson.SamplingType.DEFAULT, ConfigJson.SamplingType.RESAMPLING, ConfigJson.SamplingType.LAST_CALL))
                .build();
```

Expected: the over-margin link argument list wraps one argument per line like
any other call.

**Defect 2.** Style: `RIGHT_MARGIN=40`, `CALL_PARAMETERS_WRAP=1`,
`CALL_PARAMETERS_LPAREN_ON_NEXT_LINE=false`,
`CALL_PARAMETERS_RPAREN_ON_NEXT_LINE=true`; source
`alpha(beta, gamma, delta, epsilon, zeta, eta, theta, iota);`.

Observed (the first argument is preceded by the continuation indent):

```java
        alpha(            beta,
            gamma,
            …
            iota
        );
```

Expected (the first argument sits on the `(` line, the rest at the continuation
indent, `)` on its own line at the statement indent):

```java
        alpha(beta,
            gamma,
            …
            iota
        );
```

The full `(lparen, rparen)` matrix shows `(false,false)` and `(true,false)` are
already correct (one argument per line), `(true,true)` is correct, and only
`(false,true)` with align off is wrong.

# Proposal

Fix the three renderers and the chain renderer in `crates/core/src/formatter.rs`:

1. `args_wrapped` — in the `(!lparen_nl && rparen_nl)` arm with
   `ALIGN_MULTILINE_PARAMETERS_IN_CALLS` off, glue the first argument directly
   after `(` (no prefix) and keep the remaining arguments at the continuation
   indent, with `)` on its own line.
2. `formal_params` — apply the same gluing in its `first_inline && !align_on`
   case, so the declaration layout matches.
3. `resource_list` — apply the same gluing in its `!lp && rp` case, so the
   resource layout matches.
4. `fmt_chain_ac` — render each link's arguments through `args_wrapped` (which
   honours `CALL_PARAMETERS_WRAP` / `CALL_PARAMETER_INDENT` / the paren options
   and still breaks a nested over-margin chain through its existing
   `flat_args_chain` recursion), at one continuation level below the link's own
   line and with `)` closing at the link line. Which path runs is unchanged when
   `CALL_PARAMETERS_WRAP` is off, so today's chain output is preserved.

The six goldens that pin the malformed `(` + indent layout are updated to the
glued form; the scenarios they cover keep their option combinations.

# Decisions

1. **Both defects, one request** (agreed with the user on 2026-09-10) — they are
   independent root causes but the same "argument-list layout" area of
   `formatter.rs`, reported together.
2. **Fix the identical lparen-stays cases too** (agreed with the user): the same
   defect in `formal_params` and `resource_list` is fixed, so turning their align
   options off cannot expose it. The fix is the same shape in all three.
3. **Chain-link arguments follow `CALL_PARAMETERS_WRAP`** (agreed with the user)
   — wrap-if-long / wrap-always / chop-down, one argument per line, with
   `CALL_PARAMETER_INDENT` honoured; arguments sit one level below the link line
   and `)` closes at the link line. `METHOD_PARAMETERS_WRAP` is unaffected (chain
   links are calls, not declarations).
4. **Preserve the existing nested-chain breaking** (agreed with the user): a
   chain nested inside a link argument keeps breaking as today, independent of
   `CALL_PARAMETERS_WRAP`; the fix reuses `args_wrapped`'s existing
   `flat_args_chain` path rather than replacing it. With `CALL_PARAMETERS_WRAP`
   off (the default) chain output is byte-identical to today's.
5. **The goldens were wrong, not the layout.** Six fixtures assert the malformed
   `(` + indent output for `(false,true,align-off)` — `call_parameters_lparen_on_next_line/long_call_lparen_off`,
   `call_parameters_rparen_on_next_line/long_call`,
   `align_multiline_parameters/sample_cont`,
   `align_multiline_parameters_in_calls/sample_cont`,
   `align_multiline_parameters_in_calls/sample_default` and
   `align_multiline_resources/sample_cont`. Their test names already state the
   intended behaviour ("keeps the first argument on the lparen line"), so the
   fixtures are corrected to the glued form rather than the options being
   redefined.
6. **The `README` note stays accurate.** Its statement that a chain _used as an
   argument_ breaks independent of `CALL_PARAMETERS_WRAP` is about the nested
   chain case (decision 4) and is unchanged; the new behaviour — a chain link's
   own argument list wrapping per `CALL_PARAMETERS_WRAP` — is added to that note.

# Acceptance criteria

- With `CALL_PARAMETERS_WRAP` set and a chain whose link argument list overflows
  the margin, the link's arguments wrap one per line at one continuation level
  below the link, with `)` on the link's line; without the option (default) the
  chain output is byte-identical to before.
- `CALL_PARAMETER_INDENT` applies to a wrapped chain-link argument list.
- `CALL_PARAMETERS_LPAREN_ON_NEXT_LINE=false` with
  `CALL_PARAMETERS_RPAREN_ON_NEXT_LINE=true` and
  `ALIGN_MULTILINE_PARAMETERS_IN_CALLS` off emits the first argument directly
  after `(` (no continuation indent), the rest at the continuation indent, and
  `)` on its own line; the `(true,true)` and `(false,false)` layouts are
  unchanged.
- The declaration and resource-list renderers behave the same when
  `ALIGN_MULTILINE_PARAMETERS` / `ALIGN_MULTILINE_RESOURCES` are off.
- The six corrected fixtures assert the glued layout, new golden cases cover
  chain-link argument wrapping (wrap-if-long and the default do-not-wrap), and
  `cargo test --workspace` is green with
  `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
  `cargo fmt --all -- --check` clean.
- The README's chain-wrapping note documents that a chain link's argument list
  wraps per `CALL_PARAMETERS_WRAP`, and a changelog entry is appended on delivery
  (`fawi-implement`).

# Implementation plan

## Approach

All changes are in `crates/core/src/formatter.rs` plus the fixtures that pin the
old layout; no config, GUI, or new-dependency work.

**Defect 2 — glue the first element (three renderers).**

- `args_wrapped` (`crates/core/src/formatter.rs`): the `(false, true)` arm of the
  `match (lp, rp)` is the lparen-stays layout when alignment is off. Replace the
  `arg_strs.join(",\n")` body so the first argument is rendered without the
  `ind` prefix, directly after `(` (its column is `c + 1 + usize::from(pad)`),
  the remaining arguments keep their `ind` prefix, and the `ind(indent)` tail
  still puts `)` on its own line. The `(true, *)` arms and the align-on branch
  are untouched.
- `formal_params`: add an `else if first_inline` arm (between the existing
  `first_inline && align_on` arm and the general one) that renders the first
  parameter with `wrapped_param(params[0], &ind, inner)` and the rest as
  `ind + wrapped_param(...)` joined by `",\n"`. `lead_nl` is already `false` for
  `first_inline`, so the body glues after `(`.
- `resource_list`: add an `else if !lp && rp` arm that emits
  `line_of(resources[0], last)` with no prefix and the remaining resources as
  `ind + line_of(...)`; `lead_nl` is already `false` there.

**Defect 1 — wrap chain-link arguments.** In `fmt_chain_ac`, replace the eight
`self.flat_args_chain(link.args, indent + 1, col)` calls with a small helper
`chain_link_args(args, own_line, stmt_indent, col)` that calls
`self.args_wrapped(args, if own_line { stmt_indent + 1 } else { stmt_indent }, col)`.
`own_line` is `i > 0`, or `i == 0` when the first link starts its own line
(`first_next`, or the builder layout with a non-empty base); the first link glued
to the header passes `stmt_indent` so its `)` closes at the header indent. The
`col` argument keeps each existing call site's value (`col_after(0, &prefix)` for
own-line links, `col_after(c, &prefix)` for the header link). Because
`args_wrapped` falls back to `flat_args` / `flat_args_chain` when the list fits
and `CALL_PARAMETERS_WRAP` is off, the default chain output is unchanged.

**Fixtures.** Correct the six goldens that pin the malformed layout to the glued
form (same option combinations, only the first element's line changes):
`call_parameters_lparen_on_next_line/long_call_lparen_off.out.java`,
`call_parameters_rparen_on_next_line/long_call.out.java`,
`align_multiline_parameters/sample_cont.out.java`,
`align_multiline_parameters_in_calls/sample_cont.out.java`,
`align_multiline_parameters_in_calls/sample_default.out.java`,
`align_multiline_resources/sample_cont.out.java`. Add chain cases to
`tests/java/call_parameters_wrap/` (`chain_if_long`, and a `chain_do_not_wrap`
companion pinning the unchanged default) with a matching test in
`tests/options/call_parameters_wrap.rs`. Per `.agents/AGENTS.md` every test is a
golden pair; no inline Java strings, no new topic suite.

**Docs.** `README.md`'s chain-wrapping note gains the link-argument behaviour;
`docs/dev/changelog.md` gets an entry on delivery.

## Steps

- [x] Fix the `(false, true)` arm of `args_wrapped` to glue the first argument.
- [x] Fix the `first_inline` case of `formal_params` and the `!lp && rp` case of
      `resource_list` identically.
- [x] Replace the `flat_args_chain` calls in `fmt_chain_ac` with a
      `chain_link_args` helper that wraps through `args_wrapped`.
- [x] Update the six goldens that pin the malformed layout to the glued form.
- [x] Add `call_parameters_wrap` chain fixtures (wrap-if-long + do-not-wrap) and
      their tests.
- [x] Run `cargo test --workspace` — all goldens pass, no unrelated change.
- [x] Run `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
      `cargo fmt --all -- --check`.
- [x] Update the README chain-wrapping note and append a
      `docs/dev/changelog.md` entry.
- [x] Mark the request `done` with `verified`, and set its backlog index row to
      `done`.

## Closing

Shipped on 2026-09-10. The chain fix routes a link's argument list through
`args_wrapped` only when `CALL_PARAMETERS_WRAP` is set and otherwise keeps the
previous `flat_args_chain` path, so all pre-existing chain goldens are
byte-identical. Verified with `cargo test --workspace` (812 core integration
tests, including the two new chain cases in `call_parameters_wrap`; nine core
unit tests and six GUI tests unchanged), `cargo clippy --workspace --lib --bins
--tests -- -D warnings` (which caught and prompted a `needless_range_loop` fix)
and `cargo fmt --all -- --check`. The six corrected goldens were regenerated with
the CLI so they are byte-exact. No commit was made as part of this work.
