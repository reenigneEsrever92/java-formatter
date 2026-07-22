---
type: ChangeRequest
kind: feature
title: Emit tab indentation per USE_TAB_CHARACTER / TAB_SIZE
description: Implement tab-based indentation output so tab-configured IntelliJ schemes are honoured.
state: done
verified: { by: maintainer, at: 2026-09-03T09:48:53Z }
priority: low
tags: [dev, formatter]
owner: maintainer
---

# Problem

`USE_TAB_CHARACTER` and `TAB_SIZE` are parsed into `config::JavaStyle` but
indentation output is space-based regardless (README limitation). A team
whose scheme uses tabs gets spaces instead, which diverges from IntelliJ and
can cause noisy diffs against tab-indented code.

# Proposal

When `USE_TAB_CHARACTER` is set, emit indentation as tab characters: each
indent level is one tab per `INDENT_SIZE` worth of width, with `TAB_SIZE`
defining the tab width used for column arithmetic (how many columns a tab
advances, and therefore how continuation indentation and margin checks are
computed). Mixed tab+space output is allowed where alignment requires exact
columns beyond a tab stop, mirroring IntelliJ's continuation-indent
behaviour. When `USE_TAB_CHARACTER` is unset (the default), output is
unchanged space-based indentation.

Docs touched: `README.md` (honoured-options table and limitations),
`docs/requirements.md` (R13), `docs/dev/changelog.md` on completion.

# Decisions

- **Tabs for indentation, spaces where alignment needs exact columns.** The
  column model (`col`, margin checks, `fits`) stays column-based so wrapping
  decisions do not change; only the emitted indent changes.
- **Defaults unchanged.** Schemes without the option, and the built-in
  default style, keep producing spaces.
- **R5/R6 hold.** Semantic equivalence and idempotency apply to tab output as
  to space output.

# Acceptance criteria

- With a scheme setting `USE_TAB_CHARACTER=true` and a given `TAB_SIZE`,
  nested code is indented with tab characters — one tab per indent level —
  in the golden fixture output.
- `TAB_SIZE` participates in margin/column decisions: a wrapped construct
  under a tab scheme wraps at the same logical column as under the
  equivalent space scheme.
- Without `USE_TAB_CHARACTER`, all existing fixtures are byte-identical to
  today (`cargo test` stays green, including idempotency).
- Tab output is idempotent: formatting a tab-formatted file again is a no-op.
- The README's "tab output is not implemented" limitation is removed and the
  option table entry reflects the implemented behaviour.

# Implementation plan

## Approach

Today indentation is space-only: `Fmt::ind` (src/formatter.rs L71-73) emits
`level * indent_size` spaces and `Fmt::cont` (L76-80) adds
`continuation_indent_size` on top; column arithmetic everywhere treats one
character as one column (`fits` L82-84, and the `c + <text>.len()` chains in
`block`, `args_wrapped`, `local_var`, `assign_expr`, `ternary`, …).
`JavaStyle` already carries `use_tab_character` and `tab_size`, parsed in
src/config.rs L339-340. The change must make emitted indentation tab-aware
_and_ keep column arithmetic tab-aware, or every wrap/margin decision would
be off by the byte-vs-column difference.

Emit side: replace the space repetition in `ind`/`cont` with an indent
builder that, when `use_tab_character` is set, emits as many `\t` characters
as cover full `tab_size` columns and then spaces for the remainder
(e.g. width 10, tab 4 → `\t\t  `). This matches IntelliJ's tab-stop model
generically; when `indent_size == tab_size` (the fixture case) each level is
exactly one tab. When `use_tab_character` is false the builder reproduces
today's space output byte-for-byte, so every existing golden is untouched
outside the new feature.

Measure side: introduce `fn col_after(&self, c: usize, s: &str) -> usize`
that advances a column by the width of `s`, counting a `\t` as advancing to
the next multiple of `tab_size`, and route the measurement sites through it:
`fits` first, then each `c + …len()` that measures text possibly containing
indentation (results of `ind`/`cont`, and strings assembled from them such
as block inner statements and wrapped-argument lines). Pure-keyword prefixes
(`"return "`, `" op "`) contain no tabs and may keep `.len()` — but when in
doubt a site should use `col_after`, since correctness of margins (R3/R5) is
cheaper to guarantee uniformly. Idempotency (R6) follows automatically once
column arithmetic and emission agree: reformatting tab output re-parses
whitespace tokens that are discarded and re-emits identical tabs.

The built-in default keeps `use_tab_character = false`, so default-style
output, all existing tests and the benches are unaffected unless a scheme
opts in.

## Steps

- [x] Add `col_after` and switch `fits` to it (src/formatter.rs L82-84);
      behaviour must be unchanged when no tabs are present (regression:
      `cargo test`).
- [x] Rework `ind`/`cont` to delegate to a tab-aware indent builder gated on
      `use_tab_character`; default path returns today's exact space strings.
- [x] Audit `.len()` column arithmetic: grep `c + .*len\(\)` and
      `\.len\(\)` on `ind(`/`cont(` results and assembled fragments
      (`args_wrapped` `ac = ind.len()`, `block` inner column, `local_var`
      `val_col`, `field_decl` `val_col`, `assign_expr` `rhs_col`,
      `ternary`/chain columns); route through `col_after` where a fragment can
      contain tabs.
- [x] Fixture `tests/java/indent/tab_indent.java` with a scheme setting
      `use_tab_character = true`, `tab_size = 4`, `indent_size = 4`: nested
      code indented one tab per level (AC1); set a small `right_margin` and
      assert a wrapped construct breaks at the same logical column as under
      the equivalent space scheme (AC2).
- [x] Tests: new tests/indent.rs (or extend tests/config.rs-style suite)
      asserting tab characters appear, `assert_idempotent` on the tab fixture
      (AC4), and that default-space output is byte-identical to today across
      the existing suite (AC3).
- [x] Run `cargo test` (full suite incl. idempotency) and `cargo bench --
--no-run` to confirm the bench harness still compiles.
- [x] Update the README (options table + remove limitation; document the
      tab-stop model and the mixed tab/space continuation behaviour) and
      docs/requirements.md (R13); changelog on ship.

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
