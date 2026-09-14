---
type: ChangeRequest
kind: improvement
title: Every formatted file ends with exactly one trailing empty line
description: The end-of-file normaliser trims all trailing newlines and appends a single line terminator, so a file never ends with an empty line and a source's own trailing blank lines are dropped; make the policy unconditional — every output ends with one trailing empty line, with no option to change it.
state: done
priority: medium
tags: [dev, improvement, line-endings]
owner: maintainer
verified: { by: Zed coding agent, at: 2026-09-14T14:27:09Z }
---

# Problem

Every formatted file currently ends with exactly one line terminator and never
with an empty line. `finalise_line_endings` in `crates/core/src/formatter.rs`
(~L10752), the sole end-of-file normaliser, is called from
`format_java_diagnosed` (~L104–107) and does exactly this:

```rust
fn finalise_line_endings(out: &str, sep: &'static str) -> String {
    let collapsed = out.replace("\r\n", "\n");
    let trimmed = collapsed.trim_end_matches('\n');
    if sep == "\n" { format!("{}\n", trimmed) }
    else { format!("{}{}", trimmed.replace('\n', sep), sep) }
}
```

Whatever the source carried at its end — no terminator, a single one, or a run of
blank lines — the output is `<last content line><sep>`. The whole golden suite
under `crates/core/tests/java/**` is baseline-pinned to this shape (e.g.
`line_separator/lines_default.out.java` ends `}\n`), and R22 documents it in
prose ("the configured separator … ends every line including the final
newline").

The gap is that the tool cannot express an "every file ends with one blank line"
convention. A source that ends with a trailing empty line silently loses it, and
a source that does not end with one never gains it, so teams that want that
convention must post-process the formatter's output. There is
no option that controls this today — `LINE_SEPARATOR` picks _which_ terminator
is emitted, never how many — and the intent here is that none should be added:
this is a fixed output policy, not a scheme setting.

# Proposal

Make the end-of-file policy unconditional: after collapsing CRLF and trimming
trailing newlines, emit **two** separators — the last content line's terminator
plus the terminator of one empty line — so every output ends `<sep><sep>` (under
LF, `…}\n\n`; under CRLF, `…}\r\n\r\n`; under CR, `…}\r\r`). A source with no
trailing blank line gains one, and a run of source trailing blank lines collapses
to exactly one, so the result is uniform and deterministic.

The change is confined to `finalise_line_endings`; the configured separator still
comes from `LINE_SEPARATOR`, and no new `OptionDef` or `codestyle.xml` key is
introduced. Because trimming still precedes the append, formatting an output is a
fixed point, so idempotency (R6) holds as before, and the change is
whitespace-only, so semantic equivalence (R5) holds. This is a deliberate
divergence from IntelliJ, whose formatter ends a file with a single newline, and
it is recorded as such (like the shipped `ENABLE_JAVADOC_FORMATTING` divergence).

# Decisions

1. **Exactly one trailing empty line, unconditional** (agreed with the user on
   2026-09-14): every output ends `<sep><sep>`. A source with no trailing blank
   line gains one and a source with several collapses to exactly one, straight
   from the "always … one" intent — the alternative ("preserve one if the source
   had one") was explicitly rejected.
2. **No option** (agreed with the user): the behaviour is fixed, so no
   `OptionDef` is added and no scheme key can turn it off. `LINE_SEPARATOR` is
   unchanged and still selects only the terminator.
3. **Empty and whitespace-only input also end with the trailing empty line**
   (agreed with the user): today empty input is documented as "output is a
   single newline"; it becomes a single empty line (`\n\n` under LF).
4. **Protected regions are unaffected** (accepted default): a `// @formatter:off`
   region that runs to end of file is still echoed byte-for-byte; the trailing
   empty line is appended by the universal finalisation step afterwards.
5. **Recorded divergence from IntelliJ** (accepted default): the built-in
   formatter ends with one newline, so the trailing empty line is a deliberate,
   documented divergence rather than a matching behaviour.
6. **Idempotency (R6) is preserved** (accepted default): trimming all trailing
   newlines before appending the two separators keeps formatting output a no-op.
7. **Full golden re-baseline** (accepted default): every `.out.java` under
   `crates/core/tests/java/**` gains the trailing empty line, including the
   CRLF/CR `line_separator` variants; the change is mechanical and the fixtures
   are the contract.

# Acceptance criteria

- For any input, the formatter's output ends with the last content line's
  terminator followed by exactly one empty line, in the configured separator:
  `…}\n\n` (LF), `…}\r\n\r\n` (CRLF), `…}\r\r` (CR).
- A source with no trailing blank line gains one; a source ending with several
  blank lines collapses to exactly one.
- Empty and whitespace-only input produce a single empty line.
- Formatting the output again is a no-op (R6), and the change is whitespace-only
  (R5).
- A file whose formatting is protected by an unmatched `// @formatter:off` to
  end of file keeps its verbatim content and still ends with the trailing empty
  line.
- `cargo test --workspace` is green after every `.out.java` golden under
  `crates/core/tests/java/**` is re-baselined, with
  `cargo clippy --workspace --lib --bins --tests -- -D warnings` and
  `cargo fmt --all -- --check` clean.
- The docs that pin the current rule are updated: `docs/requirements.md` R22
  (the line-endings row), the README "Line endings" behaviour note (L815–821),
  and a `docs/dev/changelog.md` entry on delivery. No option is documented,
  because none is added.

# Implementation plan

## Approach

The change is a one-helper edit plus a mechanical golden re-baseline and the doc
updates. No config, API, or CLI surface changes.

**Engine (`crates/core/src/formatter.rs`).** `finalise_line_endings` (~L10752) is
the single end-of-file normaliser; it already collapses CRLF and trims every
trailing newline, so only the append changes — emit the separator **twice**
(once to terminate the last content line, once for the trailing empty line)
instead of once:

```rust
fn finalise_line_endings(out: &str, sep: &'static str) -> String {
    let collapsed = out.replace("\r\n", "\n");
    let trimmed = collapsed.trim_end_matches('\n');
    if sep == "\n" {
        format!("{}\n\n", trimmed)                              // was "{}\n"
    } else {
        format!("{}{}{}", trimmed.replace('\n', sep), sep, sep) // was "{}{}"
    }
}
```

The trim-before-append order is what keeps R6: a file already ending `<sep><sep>`
has both terminators trimmed away and two re-appended, so the output is a fixed
point. The existing `\r\n` collapse still runs first, so a verbatim CRLF echo
cannot double a terminator. The helper's own doc comment (~L10745) and the
finalisation comment in `format_java_diagnosed` (~L99–107) stop saying "exactly
one trailing line end" and describe the two-separator output.

**Goldens (871 files).** Every expected output under
`crates/core/tests/java/**/*.out.java` gains one terminator: `\n` for 870 of
them and `\r` for `line_separator/lines_cr.out.java`, whose CR-scheme output ends
`}\r`. The suite reads goldens with `include_str!`, so one append per file is
enough and no `.java` _input_ fixture changes. Two facts keep the blast radius
contained: the 29 self-golden idempotency tests (`assert_eq!(format(X_OUT),
X_OUT)` — e.g. `options/array_creation.rs`, `options/switch_expressions_wrap.rs`)
stay green under the fixed-point behaviour, and no test asserts `format(input) ==
input` against a bare input fixture, so nothing but the goldens shifts.

**Non-golden surface.** The CLI tests (`crates/cli/tests/directory_mode.rs`)
compare the tool's own outputs across modes, the GUI only previews
(`crates/gui/src/main.rs`), and the benches assert no bytes — none of them pin
an exact expected string, so none changes.

**Tests.** A new `trailing_empty_line` option module
(`crates/core/tests/options/trailing_empty_line.rs`, registered in
`crates/core/tests/options.rs`) with fixtures under
`crates/core/tests/java/trailing_empty_line/` pins the rule directly: a fixture
ending in a run of blank lines collapses to exactly one, a fixture with none
gains one, and a zero-byte fixture plus a whitespace-only fixture pin the
empty-input decision; each case also asserts idempotency. The existing
`line_separator` goldens pin the three separator forms (`}\n\n`, `}\r\n\r\n`,
`}\r\r`).

**Docs.** R22 is the row that governs line endings, so its wording is amended
rather than a new row added; the same rule appears in the README's "Line
endings" behaviour note and in the `LINE_SEPARATOR` row of the settings
reference, and all three are updated together. The divergence from IntelliJ is
stated in the request (decision 5) and carried into the README note and the
changelog entry.

## Steps

- [x] `crates/core/src/formatter.rs`: make `finalise_line_endings` append two
      separators, and update both the helper's doc comment and the
      `format_java_diagnosed` finalisation comment (the two-separator rule, the
      R6 fixed point, and the deliberate IntelliJ divergence).
- [x] Re-baseline the goldens: append `\n` to every `.out.java` under
      `crates/core/tests/java/**`, except the two non-LF separator goldens,
      which gain their own separator — `line_separator/lines_crlf.out.java`
      gains `\r\n` and `line_separator/lines_cr.out.java` gains `\r`; check
      each file now ends with exactly two terminators.
- [x] Add the `trailing_empty_line` fixtures and the option test module, and
      register it in `crates/core/tests/options.rs` — blank-run collapse,
      gains-one, empty and whitespace-only input, each with an idempotency
      assert.
- [x] `docs/requirements.md`: amend the R22 row so the line-ending clause ends
      with the trailing empty line (and drop the now-wrong "including the final
      newline" phrasing), and update the edge-case sentence at L49 so empty
      input is "a single empty line" instead of "a single newline".
- [x] `README.md`: update the "Line endings" behaviour note (L815–821) to state
      that the configured separator ends every line and that every file ends
      with exactly one trailing empty line, noting the divergence from
      IntelliJ.
- [x] `docs/settings/common.md`: update the `LINE_SEPARATOR` row's effect text
      (currently "Line separator emitted at every line end, including the final
      newline") to match the amended R22 wording.
- [x] `docs/dev/backlog/index.md`: flip this request's row to `planned` (and to
      `done` on delivery).
- [x] Run `cargo test --workspace` (the re-baselined goldens plus the new
      module), then `cargo clippy --workspace --lib --bins --tests -- -D
warnings` and `cargo fmt --all -- --check` — all green.
- [x] `docs/dev/changelog.md`: append the delivery entry describing the
      two-separator finalisation, the golden re-baseline and the doc updates;
      mark this request `done` with the closing note.

## Closing

Shipped on 2026-09-14. `finalise_line_endings` in
`crates/core/src/formatter.rs` now appends the configured separator twice — once
to terminate the last content line and once for the trailing empty line — so
every output ends `<sep><sep>` (LF `…}\n\n`, CRLF `…}\r\n\r\n`, CR `…}\r\r`),
a deliberate divergence from IntelliJ's single final newline. All 871
`.out.java` goldens under `crates/core/tests/java/**` were re-baselined (the
CRLF variant gained `\r\n`, the CR variant `\r`), and a new
`trailing_empty_line` fixture suite pins the gains-one, collapse-to-one,
empty-input and whitespace-only cases plus idempotency. The README "Line
endings" note, `docs/requirements.md` (R22 and the empty-input edge-case
sentence) and the `LINE_SEPARATOR` row of `docs/settings/common.md` now
describe the rule. Verified with `cargo test --workspace` (906 core option
tests — four new — plus 9 core unit, 18 CLI and 6 GUI tests),
`cargo clippy --workspace --lib --bins --tests -- -D warnings` and
`cargo fmt --all -- --check`. The changes are uncommitted in the worktree (no
commit hash); the changelog entry was added on 2026-09-14.
