---
type: ChangeRequest
kind: improvement
title: Detect parse errors and warn, still emitting best-effort output
description: Surface tree-sitter parse errors on stderr while keeping the safe best-effort formatting contract.
state: done
verified: { by: maintainer, at: 2026-09-02T18:54:03Z }
priority: high
tags: [dev, formatter]
owner: maintainer
---

# Problem

`formatter::format_java` never inspects the tree-sitter tree for parse
errors. A file that is not valid Java is formatted best-effort — the CST
contains `ERROR` nodes and the formatter echoes whatever it does not model
verbatim. That is safe (requirement R4), but silent: a user who pipes in a
file with a syntax error has no way to know that part of the output was
never really formatted. Given the project's first value is correctness
(requirement R5), undetected invalid input is a correctness blind spot.

# Proposal

Detect parse errors after parsing and report them on stderr as a warning —
naming the error construct and/or line where feasible and stating that the
output is best-effort — while still emitting the formatted output and
exiting 0. The library entry point should expose the detection (e.g. return
the formatted text plus a list of parse-error locations) so that both CLI
and library users get the signal, and the CLI prints the warning. The
never-corrupt contract for invalid input (R4, R15) is documented in the
README and in the requirements.

Docs touched: `README.md` (formatting-behaviour notes and limitations),
`docs/requirements.md` (R15 moves from deferred to delivered when shipped),
`docs/dev/changelog.md` on completion.

# Decisions

- **Warn, do not fail.** Invalid input still produces best-effort output and
  exits 0 — decided with the user on 2026-09-02 so editors and pipes keep
  working (a non-zero exit would break scripted use). The warning is the
  signal, not the exit code.
- **Detect at the library boundary.** Detection lives in the formatting entry
  point, not the CLI argument parsing, so both consumers benefit.
- **Keep R4 intact.** Detection never changes formatting behaviour; it only
  adds a stderr report.

# Acceptance criteria

- Given a `.java` file with a syntax error, when it is formatted, then a
  warning naming the error (construct and/or line) is written to stderr, the
  best-effort formatted source is still written to stdout, and the exit code
  is 0.
- Given valid Java input, when it is formatted, then no parse warning is
  printed.
- Given empty input, no parse warning is printed (output remains a single
  newline).
- The library API reports parse-error locations to callers (not only to the
  CLI's stderr).
- The never-corrupt behaviour for invalid and unmodeled input is documented
  in the README, and the fixture-based integration suite covers a syntax-error
  case under `tests/`.

# Implementation plan

## Approach

Detection belongs in the formatting entry point so both the library and the
CLI see it. Keep the existing `pub fn format_java(src, style) -> String`
signature unchanged — the whole integration suite and the benches call it —
and add a sibling `pub fn format_java_diagnosed(src, style) ->
(String, Vec<ParseDiagnostic>)`; `format_java` delegates and drops the
diagnostics. After `parser.parse(...)` in `format_java` (src/formatter.rs
L18-36), inspect the tree: if `tree.root_node().has_error()` is false there
are no diagnostics. Otherwise collect error and missing nodes
(`node.is_error() || node.is_missing()`), skipping any node whose ancestor is
also an error node (an `ERROR` node can span a large region and would flood
the report with descendants), capping at ~10 and deduplicating by
(kind, line). `ParseDiagnostic` carries the node kind, a human message, and a
1-based line/column computed from the node's start byte against `src`.

Formatting logic is untouched, so R4/R5 behaviour and every existing golden
stay identical; the library merely _reports_. The CLI (src/main.rs L63)
switches to the new function, prints each diagnostic as a `warning: …` line
on stderr before writing the formatted output, and still exits 0 (decision:
warn, do not fail).

## Steps

- [x] Add `ParseDiagnostic` and `format_java_diagnosed` to
      src/formatter.rs; keep `format_java` as a delegating wrapper. Existing
      suite and benches must compile unchanged (AC: no API break).
- [x] Implement the tree walk collecting top-most error/missing nodes with
      the cap and dedupe described above; compute line/column from byte
      offsets (count `\n` before the start byte).
- [x] Wire src/main.rs to `format_java_diagnosed`, emitting one
      `warning: …` per diagnostic on stderr, then the formatted output; exit
      code stays 0 (AC1, AC2).
- [x] Add an integration suite `tests/parse_errors.rs` (reusing
      tests/common/mod.rs) with a fixture `tests/java/errors/syntax_error.java`:
      assert diagnostics are non-empty for the fixture and empty for a valid
      input and for empty input (AC2, AC3, AC4).
- [x] Manual CLI check: pipe an invalid snippet through `cargo run`, confirm
      the warning appears on stderr, output is still written, and `echo $?`
      reports 0 (AC1).
- [x] Update the README: document the never-corrupt best-effort contract and
      the new stderr warning; adjust the formatting-behaviour notes.
- [x] Run `cargo test`; update docs/dev/backlog and
      docs/requirements.md (R15) and append a changelog entry when the change
      ships (fawi-implement).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
