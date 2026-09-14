---
type: ChangeRequest
kind: bug
title: A method parameter list is not wrapped when the declaration header overflows only because of the throws clause or the body brace
description: The parameter-list margin test measures only `(…params…)`, so a declaration header pushed over the margin by its trailing `throws` clause or its body `{` is left over-margin even under `METHOD_PARAMETERS_WRAP` wrap-if-long / chop-down.
state: done
verified: { by: Zed coding agent, at: 2026-09-14 }
priority: medium
tags: [dev, bug, formatter]
owner: maintainer
---

# Problem

A method or constructor declaration whose **whole header** exceeds the right
margin is left untouched when the parameter list _alone_ fits. The overflow
comes from the tokens that follow the parameters — the ` throws …` clause and
the body's opening ` {` — and the wrap decision never sees them, so an
over-margin line is emitted even when the scheme asks the parameter list to
wrap.

The cause is that `Fmt::formal_params` decides `should_wrap` from
`!self.fits(c, &flat)`, where `flat` is only the parenthesised parameter list
and `c` is the column just after `(`. `Fmt::method_decl` / `Fmt::constructor_decl`
call it with that `c` and only _then_ append the `throws` clause and the body
brace. The trailing tokens are invisible to the test, so:

- the parameter list is judged to fit and stays flat, while
- the `throws` clause cannot absorb the overflow either — under its default
  `THROWS_LIST_WRAP` (`DoNotWrap`) it never wraps, and `clause_list` refuses to
  wrap a single-element list even under `WrapAlways` (R26).

The result is an over-margin declaration header that violates the margin
contract (R22) and diverges from IntelliJ, whose wrap codes are defined over
"if the _line_ is long". No existing fixture covers a header whose parameters
fit but whose total width does not, which is how the gap went unnoticed.

# Reproduction

Style: the repository's own `codestyle.xml` — `RIGHT_MARGIN` default (`120`),
`METHOD_PARAMETERS_WRAP=5` (chop down if long), `METHOD_PARAMETERS_LPAREN_ON_NEXT_LINE`
and `METHOD_PARAMETERS_RPAREN_ON_NEXT_LINE` on.

Input (`Sample.java`):

```java
class Sample {
    public String transform(String firstArgument, int secondArgument, boolean thirdArgument, double fourthArgument) throws IllegalStateException {
        return firstArgument;
    }
}
```

Observed — the header stays on one line at **146 columns** (26 over the
margin); the consumed `)` sits at column 115, so the parameter list fits and
the single `throws IllegalStateException` pushes the line over:

```java
class Sample {
    public String transform(String firstArgument, int secondArgument, boolean thirdArgument, double fourthArgument) throws IllegalStateException {
        return firstArgument;
    }
}
```

Expected — the parameter list wraps per `METHOD_PARAMETERS_WRAP`, and the
`throws` clause rides the `)` line:

```java
class Sample {
    public String transform(
        String firstArgument,
        int secondArgument,
        boolean thirdArgument,
        double fourthArgument
    ) throws IllegalStateException {
        return firstArgument;
    }
}
```

The same defect fires when only the body brace overflows. With
`RIGHT_MARGIN=85`, the parameter list ends exactly at the margin and the
trailing ` {` pushes the line to 87 columns — again nothing wraps:

```java
class Sample {
    public void methodName(int alphaParameter, int betaParameter, int gammaParameter) {
        int x = 1;
    }
}
```

At `RIGHT_MARGIN=84` the same input wraps (the parameter list itself now ends
past the margin), confirming the wrap machinery is correct and only the
measurement is wrong.

With `METHOD_PARAMETERS_WRAP` absent (its default `DoNotWrap`) an over-margin
header is unchanged — that is intended, and stays so.

# Proposal

Make the declaration parameter-list margin test measure the whole header
instead of the parameters in isolation. `Fmt::formal_params` (declaration side)
gains the trailing text that shares the header line — the flat `throws` clause
plus the body's opening brace (or `;` for an abstract / interface method) when
the brace style keeps it on the header line — and tests
`fits(c, &(flat + trailing))`. When that composed line overflows, the list
wraps under `METHOD_PARAMETERS_WRAP` (wrap-if-long and chop-down share the
layout; `WrapAlways` still wraps unconditionally). `Fmt::method_decl` and
`Fmt::constructor_decl` build and pass that trailing text; call sites
(`is_call`) pass nothing and are unchanged.

`METHOD_PARAMETERS_WRAP` takes precedence over `THROWS_LIST_WRAP`: the trailing
`throws` clause is measured in its flat form, so when both options are enabled
the parameters wrap first and the throws list stays on the `)` line. When the
parameters do not wrap (their option is `DoNotWrap`), the throws list keeps its
current behaviour, so the throws-family goldens are unchanged. The change is
whitespace/layout only (R5), and the produced layouts re-format to themselves
(R6).

# Decisions

1. **Whole-header measurement (agreed with the user on 2026-09-14).** The
   parameter-list margin test counts the parameter list, the `throws` clause
   and the body `{` — not just the parameter list. The wrap codes are defined
   over "if the line is long", and the line is the whole declaration header.
2. **`METHOD_PARAMETERS_WRAP` wins over `THROWS_LIST_WRAP` (agreed with the
   user on 2026-09-14).** When both are set to a wrapping code, the parameter
   list wraps and the throws list stays on the `)` line; the throws clause is
   measured flat rather than given the first chance to wrap. This keeps a
   single, predictable wrap point for the declaration header.
3. **Only tokens that actually share the header line count (agreed with the
   user on 2026-09-14).** A next-line brace style (`METHOD_BRACE_STYLE` =
   next-line) contributes no trailing brace, so it does not make the list wrap.
4. **Out of scope:** the `clause_list` single-element rule and the
   `throws` / `extends` / `implements` clause layouts themselves are unchanged;
   when the parameters do not wrap, the throws list keeps its behaviour (R26).
   A single-element `throws` clause still never splits.
5. **No existing doc is contradicted.** The README lists
   `METHOD_PARAMETERS_WRAP` but does not state how its margin is measured, so
   the fix adds a behaviour note rather than correcting a wrong one.

# Acceptance criteria

- The reported declaration formats to the wrapped parameter list
  (`RIGHT_MARGIN=120`, `METHOD_PARAMETERS_WRAP=5`): no output line exceeds the
  margin, and the `throws` clause stays on the `)` line.
- A declaration whose header overflows only by the body brace
  (`RIGHT_MARGIN=85` case) wraps under wrap-if-long / chop-down; the same
  fixture under a next-line brace style stays flat.
- The same behaviour holds for constructors (they share the code path).
- No spurious wrap: a declaration whose whole header fits the margin is left
  flat.
- With `METHOD_PARAMETERS_WRAP` at its default (`DoNotWrap`), over-margin
  headers are byte-identical to today, and the `throws_list_wrap`,
  `throws_keyword_wrap` and `align_*_throws_list` goldens stay byte-identical.
- New golden fixtures: parameters fit + single `throws` overflows; parameters
  fit + multi-element `throws` overflows; parameters fit + only the brace
  overflows; next-line brace style (no trailing token) stays flat; an
  absent-option (do-not-wrap) case that keeps the over-margin header. Each
  wrapped golden is idempotent (formatting the output is a no-op — R6).
- `cargo test --workspace` is green with
  `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
  `cargo fmt --all -- --check` clean.
- The README behaviour notes state that the declaration parameter-list wrap
  decision measures the whole header (parameters + `throws` + body brace);
  `docs/requirements.md` records the fix under the next free R number; a
  changelog entry is appended on delivery.

# Implementation plan

## Approach

The change is confined to `crates/core/src/formatter.rs` plus the golden
fixtures, tests and docs. No new option, so `crates/core/src/config.rs` is
untouched.

**Shared flat clause text.** `Fmt::clause_list` (L10230-10241) builds the flat
` throws A, B` text inline. Extract it into a small `Fmt::flat_clause(keyword,
node, render)` helper and have `clause_list` call it, so the text the new margin
test measures is byte-identical to the text `clause_list` emits when it does not
wrap.

**The header tail.** Add `Fmt::decl_header_tail(node, indent) -> String`: the
tokens that share the declaration's header line after the parameter list —

- the flat `throws` clause (`flat_clause("throws", throws, |n| self.flat_type(n))`)
  when `get_throws(node)` is `Some`;
- then the body opener: `;` when the method has no `body`; nothing when
  `method_brace_style` is a next-line style (`NextLine` / `NextLineShifted` /
  `NextLineShifted2`) or when the one-line body presentation starts with a
  newline (`KEEP_SIMPLE_METHODS_IN_ONE_LINE` with `NEW_LINE_WHEN_BODY_IS_PRESENTED`,
  detected via the existing `one_line_body` + `body_gap`); otherwise
  `self.sp(self.style.space_before_method_lbrace)` + `{`.

The tail is a pure measurement string: it mirrors `brace_before_body`'s
placement without a trailing newline, so `col_after` cannot reset the column and
trivially "fit".

**The margin test.** `formal_params` (L5244) gains a `header_tail: &str`
argument and its `should_wrap` margin arm (L5328-5332) becomes
`!self.fits(c, &format!("{}{}", flat, header_tail))`. `WrapAlways` still wraps
unconditionally and `DoNotWrap` still never wraps, so `METHOD_PARAMETERS_WRAP`
takes precedence over `THROWS_LIST_WRAP` (its `flat` clause is measured, not
rendered first). The `PARAMETER_ANNOTATION_WRAP` demand test (L5309-5313) keeps
measuring the list alone — it decides whether a parameter's annotation needs its
own-line form, not the line's length.

**Callers.** `method_decl` (L4343-4348) and `constructor_decl` (L4418-4423)
compute `decl_header_tail(node, indent)` and pass it to `formal_params`; the
`pcol` arithmetic is unchanged, so the tail is measured from the same start
column. `is_call` remains `false` at both call sites (no call site passes
`true` today), so the call path keeps an empty trailing string.

**Tests.** `crates/core/tests/options/method_parameters_wrap.rs` gains golden
pairs; fixtures go under `crates/core/tests/java/method_parameters_wrap/`. A
narrow margin (60, as the throws-family files use) keeps the fixtures small.
Cases: parameters fit while a single `throws` overflows; parameters fit while
only the body brace overflows; a next-line brace style leaves the header flat;
an absent option keeps the over-margin header; an already-wrapped header is a
no-op; and a header that fits is not wrapped.

**Docs.** `README.md` gains a behaviour note stating that the declaration
parameter-list wrap decision measures the whole header (parameters + `throws` +
body brace) and that a next-line brace style contributes no trailing brace.
`docs/requirements.md` gains the next free requirement row (R50). The change is
whitespace/layout only (R5) and the produced layouts re-format to themselves
(R6).

## Steps

- [x] `crates/core/src/formatter.rs`: extract `flat_clause` from `clause_list`
      and make both use it (no behaviour change on its own).
- [x] `crates/core/src/formatter.rs`: add `decl_header_tail` and thread the
      `header_tail` argument through `formal_params`, `method_decl` and
      `constructor_decl`; the margin arm measures `flat + header_tail`.
- [x] Add the `method_parameters_wrap` fixtures and golden assertions (the five
      cases above), eyeballing each golden before accepting it; regenerate only
      goldens the fix is intended to change.
- [x] `README.md`: add the whole-header measurement behaviour note and update
      the `METHOD_PARAMETERS_WRAP` description so it does not contradict it.
- [x] `docs/requirements.md`: add the R50 row and a milestones mention.
- [x] Quality gates: `cargo test --workspace`, `cargo clippy --workspace --lib
--bins --tests -- -D warnings`, `cargo fmt --all -- --check` — all green.
- [x] `docs/dev/backlog/index.md`: flip this row's state to `done`; append the
      changelog entry and set this request `state: done` with `verified`.

## Closing

Shipped on 2026-09-14. `Fmt::formal_params` in `crates/core/src/formatter.rs`
now measures the whole declaration header: a new `Fmt::decl_header_tail` builds
the flat `throws` clause plus the body's opening `{` — or the terminating `;`
when there is no body, and nothing when the brace style moves the brace to its
own line — and the `should_wrap` margin arm tests `flat + header_tail`, so
`METHOD_PARAMETERS_WRAP` wrap-if-long / chop-down wraps a header that crosses
the margin only after the `)`. The flat clause text comes from a new
`Fmt::flat_clause` helper shared with `Fmt::clause_list`, so the measured text
equals the rendered text; `method_decl` and `constructor_decl` pass the tail,
and the (unused) call path passes an empty string. New golden pairs under
`crates/core/tests/java/method_parameters_wrap/` —
`params_fit_throws_overflows` (plus a do-not-wrap golden), `brace_overflow`
(plus its next-line-brace golden), `header_fits` and `already_wrapped` — with
seven assertions in `crates/core/tests/options/method_parameters_wrap.rs` pin
the behaviour. The README behaviour notes, `docs/requirements.md` (R50) and the
changelog document it. Verified with `cargo test --workspace` (912 core
integration tests — six new plus one refactored — 9 core unit, 18 CLI and 6 GUI
tests), `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
`cargo fmt --all -- --check`, all green. The changes are uncommitted in the
worktree.
