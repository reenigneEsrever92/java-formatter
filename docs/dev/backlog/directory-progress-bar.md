---
type: ChangeRequest
kind: improvement
title: Show a progress bar when formatting a directory
description: Draw a determinate progress bar on stdout (via indicatif) while -d / --dir formatting runs, showing the file count and current file, suspending it for warnings/errors, and print a one-line summary on stderr when stdout is not a terminal.
state: done
verified: { by: maintainer, at: 2026-09-11T00:00:00Z }
priority: low
tags: [dev, improvement, cli]
owner: maintainer
---

# Problem

Directory mode (`-d` / `--dir`, with `-r` / `--recursive`) formats every
`*.java` file under a directory in place (`crates/cli/src/main.rs`,
`format_directory`). On a large source tree a run can take a while, and the
operator gets zero feedback until it finishes: directory mode prints nothing to
stdout and only reports problems to stderr. There is no sense of how many files
have been processed or how many remain, so a long run feels unresponsive and a
scripted/CI run on success is completely silent.

# Proposal

While a directory-mode run is in progress, draw a determinate progress bar on
stdout using the `indicatif` crate (a new runtime dependency of the cli crate).
The bar shows the file counter and the current file name (e.g.
`Formatting [3/12] src/Foo.java`), incrementing once per file; `collect_java_files`
already gathers every file before the loop, so the total is known up front. The
bar is drawn only when stdout is a terminal — `indicatif`'s stdout draw target
hides itself for non-terminals, so redirected/piped output stays clean. While a
warning or error for a file is printed to stderr, the bar is suspended first so
the two never interleave on screen.

When stdout is not a terminal (so no bar can render), the run ends with a
one-line summary on stderr — `Formatted 12 files` or `Formatted 11 files, 1
failed` — so piped/CI runs still get feedback instead of silence.

The change is cosmetic: the per-file read → format → write-if-changed loop, the
failure semantics (continue past failures, exit 1 if any file failed), the
exit codes, and single-file / stdin mode are all unchanged. The documented
"stdout stays empty in directory mode" contract is amended: stdout carries
either the progress bar (terminal) or nothing (piped), never file contents.

Docs touched: README "Usage" paragraph on directory-mode stdout, docs/requirements.md
R42 row (the "stdout stays empty" wording), docs/dev/backlog/index.md (this
entry, on delivery), and docs/dev/changelog.md on delivery.

# Decisions

- **Bar on stdout via `indicatif` (user choice).** The progress bar renders on
  stdout when it is a terminal, using the `indicatif` crate — a new runtime
  dependency added to `workspace.dependencies` and `crates/cli/Cargo.toml`
  (its first fetch needs crates.io network access). `indicatif`'s stdout draw
  target hides the bar automatically when stdout is not a terminal, so no
  separate TTY detection is needed in our code. This amends today's
  "stdout stays empty in directory mode" wording (README, R42): the bar is
  terminal UI, not file output — file contents are still never written to
  stdout.
- **Counter + current file name (user choice).** The bar template shows
  `pos/len` and the current file's path as the message, incremented once per
  file. The total comes from the already-collected `files` vector, so the bar
  is determinate.
- **Warnings/errors suspend the bar (user choice).** Each per-file warning or
  error is printed through the bar's suspend mechanism (clear bar, print,
  redraw), so stderr messages and the bar never interleave on a terminal.
- **One-line summary when stdout is not a terminal (user choice).** Piped /
  redirected / CI runs get `Formatted N files` (and `, M failed` when any file
  failed) on stderr at the end of the run; a fully successful non-TTY run gains
  exactly this one line of output. On a terminal the bar itself is the
  feedback, so no trailing summary is printed.
- **No `--no-progress` flag (default).** TTY detection plus `indicatif`'s
  auto-hide already covers piped/CI runs, and every other mode has no bar, so a
  flag adds surface with no distinct use case.
- **Semantics unchanged.** Exit codes (0 on success, 1 if any file failed),
  continue-past-failures, write-if-changed, and the single-file / stdin path
  are untouched; the improvement is measured by the run being observable
  without any of those behaviours changing. The existing 12 CLI tests run with
  piped stdout (not a terminal), so they keep passing unchanged and pin the
  bar-free, summary-line behaviour.

# Acceptance criteria

- With stdout a terminal, `java-formatter -d DIR` / `-d DIR -r` draws a
  determinate bar on stdout showing the file counter and the current file
  name, incrementing once per file, and clears it on success.
- Per-file warnings/errors print to stderr without garbling the bar (the bar
  is suspended while each message prints).
- With stdout not a terminal (piped/redirected), no bar is drawn and stdout
  stays clean; the run ends with `Formatted N files` (plus `, M failed` when
  any file failed) on stderr.
- Exit-code semantics are unchanged: 0 when every file succeeded, 1 when any
  file failed; the run still continues past failures.
- Single-file / stdin mode is unchanged: formatted source to stdout, no bar,
  exit 0.
- The existing 12 CLI tests still pass (they use piped stdout, so they pin the
  no-bar + summary-line behaviour); new tests cover the summary line and the
  failure count.
- README "Usage" documents the bar and the amended stdout behaviour;
  docs/requirements.md R42 wording is updated; backlog index and changelog are
  updated on delivery.

# Implementation plan

## Approach

### 1. Dependency

Add `indicatif` (0.18, the current stable line) to `workspace.dependencies` in
`Cargo.toml` and reference it from `crates/cli/Cargo.toml` `[dependencies]`
(`indicatif = { workspace = true }`). It is not in `Cargo.lock` today, so the
first build fetches it and its transitive deps (`console`, `portable-atomic`,
`unicode-width`, …) from crates.io — requires a one-time network grant.

### 2. Progress bar in `format_directory` (`crates/cli/src/main.rs`)

- Detect the terminal once: `let tty = std::io::stdout().is_terminal()`
  (`std::io::IsTerminal`, std since 1.70 — no extra dependency).
- Create the bar with `ProgressBar::with_draw_target(files.len() as u64,
if tty { ProgressDrawTarget::stdout() } else { ProgressDrawTarget::hidden() })`
  so a piped stdout never draws (explicit rather than relying on
  `indicatif`'s own detection). Template `{prefix} [{bar:40.cyan/blue}]
{pos}/{len} {msg}`, prefix `Formatting`; the message is each file's path
  relative to the base `dir` (`path.strip_prefix(dir)`), matching the
  `Formatting [3/12] src/Foo.java` shape.
- In the loop: `bar.set_message(...)` before each file, `bar.inc(1)` after
  each file (including the read-error `continue` branch).
- Every per-file `eprintln!` (read error, parse warnings, write error) is
  wrapped in `bar.suspend(|| ...)` so stderr output never interleaves with the
  bar on a terminal; on a hidden bar `suspend` is a passthrough.
- After the loop: `bar.finish_and_clear()` (no-op on a hidden bar).
- Summary: when `!tty`, print to stderr `Formatted {n} file(s)` (plus
  `, {m} failed` when `m > 0`), where `n = files.len() - failed`
  (successfully formatted) and `m = failed` — matching
  `Formatted 11 files, 1 failed`. Singular/plural handled (`1 file`).
  On a terminal no summary is printed (the bar was the feedback).
- Exit code logic unchanged: `process::exit(if failed > 0 { 1 } else { 0 })`.

Simplify `failed` to a `u64` counter (incremented on read error, parse-error
warning, and write error) instead of the current `bool`.

### 3. Tests (`crates/cli/tests/directory_mode.rs`)

The existing 12 tests run with piped stdout, so they pin the hidden-bar
behaviour and keep passing; the new summary line adds stderr output only.
Add:

- `summary_line_reports_the_run` — a successful run on two files: stderr ends
  with `Formatted 2 files` and does not contain `failed`.
- `summary_line_counts_failures` — one invalid-Java file among good ones:
  stderr contains `Formatted 1 files, 1 failed` (2 files total, 1 failed).
- `summary_line_for_empty_directory` — a directory with no `.java` files:
  stderr contains `Formatted 0 files`, exit 0.

TTY rendering cannot be integration-tested without a pty; the draw target is
chosen explicitly from `IsTerminal`, so the piped tests cover the bar-hidden
path and the summary path deterministically.

### 4. Docs

- README "Usage": rewrite the directory-mode paragraph — file contents are
  never written to stdout; a progress bar (`Formatting [3/12] src/Foo.java`)
  is drawn on stdout when it is a terminal; piped/redirected runs end with a
  one-line stderr summary; exit-code sentence unchanged.
- docs/requirements.md R42: replace the "stdout stays empty" wording with the
  bar + summary behaviour (the improvement is a follow-up to R42, not a new
  requirement row).
- docs/dev/backlog/index.md: add this entry (state done) at the top.
- docs/dev/changelog.md: append a bullet under 2026-09-11.

## Steps

- [x] Add `indicatif` to `workspace.dependencies` and `crates/cli/Cargo.toml`.
- [x] Add the progress bar to `format_directory` (TTY-gated draw target,
      counter + relative file message, `suspend` for messages, `failed` as a
      counter, non-TTY summary on stderr, exit codes unchanged).
- [x] Add the three summary tests to `crates/cli/tests/directory_mode.rs`.
- [x] Update README "Usage" and docs/requirements.md R42 wording; add the
      backlog index row.
- [x] Run `cargo test --workspace`, `cargo clippy --workspace --lib --bins
    --tests -- -D warnings`, `cargo fmt --all -- --check`; smoke-test `-d`
      with piped and terminal stdout; append the changelog entry and close
      the request.

## Closing

All 15 CLI tests (12 directory-mode + 3 new summary tests), the 817 core
golden tests, and the 6 GUI tests pass; `cargo fmt --check` and
`cargo clippy --workspace --lib --bins --tests -- -D warnings` are clean. The
piped path was smoke-tested (empty stdout, `Formatted 2 files` on stderr,
exit 0) and the terminal path through a pty (the bar renders with ANSI
colors). The changes and the `indicatif` 0.18 dependency are uncommitted in
the worktree (no commit hash); the changelog entry was added on 2026-09-11.
