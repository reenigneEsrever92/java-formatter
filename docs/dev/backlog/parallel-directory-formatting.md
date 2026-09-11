---
type: ChangeRequest
kind: feature
title: Format a directory in parallel across CPU cores
description: Replace the sequential per-file loop in directory mode with a rayon parallel iteration so -d / --dir runs use the machine's cores by default, with unchanged per-file results, failure counting, exit codes, and progress bar.
state: done
verified: { by: maintainer, at: 2026-09-11T00:00:00Z }
priority: medium
tags: [dev, cli, performance]
owner: maintainer
---

# Problem

Directory mode (`-d` / `--dir`, with `-r` / `--recursive`) formats every
`*.java` file under a directory in place (`format_directory` in
`crates/cli/src/main.rs`). It processes the collected file list in a sequential
`for` loop: read → format → write-if-changed, one file at a time. Formatting is
CPU-bound — each file runs tree-sitter parsing and the tree pretty-printer — and
files are independent, so on a multi-core machine all but one core sit idle for
the whole run. Wall-clock time is the sum of every file's formatting time, which
makes a large source tree unnecessarily slow.

# Proposal

Replace the sequential loop with a parallel iteration over the already-collected
`files` vector using `rayon`, so directory runs use the machine's cores by
default. Each worker independently reads, formats, writes-if-changed, and reports
its file's diagnostics; the counter shown in the progress bar advances once per
file as before. No core-library change is needed:
`formatter::format_java_diagnosed` constructs a fresh tree-sitter `Parser` per
call and holds no shared mutable state, and `JavaStyle` is plain `Sync` data, so
it can be shared by reference across threads.

Parallelism changes nothing observable about the outcome: every file's formatted
bytes, the write-only-when-changed rule, the failure count, the summary line, the
exit code, and single-file / stdin mode are all exactly as today. The only
differences are that the run gets faster on multi-core machines, that the
progress bar's "current file" becomes best-effort (the most recently handled
file, since several are in flight), and that per-file stderr messages may appear
in any order.

`rayon` is already in `Cargo.lock` (transitive via `criterion`), so adding it as
a direct dependency of the cli crate needs no new crate download.

# Decisions

- **Parallelize with `rayon`, not hand-rolled threads — user choice.** rayon's
  parallel iterators (a `par_iter().for_each` over the collected files) express
  the work in a few lines with work-stealing load balancing, and the crate is
  already resolved in the workspace lockfile, so it adds no new download. The
  std-only alternative (`std::thread::scope` plus an `AtomicUsize` index) avoids
  a new direct dependency but is more code to maintain and balance.
- **Automatic thread count, no `-j` / `--jobs` flag — user choice.** rayon's
  global pool already defaults to the machine's available parallelism, and it
  honours the standard `RAYON_NUM_THREADS` environment variable, so an operator
  or CI job that needs to cap concurrency can set it without new CLI surface.
- **The progress bar keeps its shape, with a best-effort file name — user
  choice.** The determinate `[pos/len]` counter still advances once per completed
  file. The `{msg}` file name becomes the most recently handled file rather than
  a true single "current" file (several are in flight); this is deliberately
  best-effort and cosmetic. The bar stays hidden when stdout is not a terminal,
  and the non-terminal summary line is unchanged.
- **Per-file stderr ordering is irrelevant; messages are serialized only to
  avoid interleaving — user choice.** Warning/error lines keep naming their own
  file and their own text, but with several workers the overall order across
  files is nondeterministic. A mutex around the print (combined with the bar's
  suspend mechanism) keeps each line intact; the tests assert on content, not
  order, so they remain order-agnostic.
- **The `failed` count becomes an atomic counter.** Multiple workers increment
  it concurrently; an `AtomicU64` (relaxed ordering, since it is only summed at
  the end) preserves the existing `Formatted N files, M failed` summary exactly.
- **Collection stays sequential and deterministic.** `collect_java_files`
  performs cheap directory I/O and its sorted order feeds deterministic input to
  the parallel stage; parallelizing the walk would add complexity for negligible
  gain.
- **Scope: directory mode only.** Single-file and stdin behaviour, the exit-code
  contract, and the core library are untouched.

Docs touched: README "Usage" (the directory-mode paragraph gains a sentence that
the run formats files in parallel using the machine's cores, capped by
`RAYON_NUM_THREADS`, and that the bar's file name is best-effort),
docs/requirements.md R42 (add the parallel-execution clause),
docs/dev/backlog/index.md (this entry, on delivery), and
docs/dev/changelog.md (on delivery).

# Acceptance criteria

- `java-formatter -d DIR [-r] [--style …]` produces byte-identical files to a
  sequential run — same formatted output, same write-only-when-changed rule.
- The run uses more than one thread on a multi-core machine when more than one
  file is present; `RAYON_NUM_THREADS=1` forces a single-threaded run that is
  byte-identical to the default run.
- Failure semantics are unchanged: per-file read errors, invalid Java, and write
  failures are logged to stderr naming the file, other files are still formatted,
  and the process exits 1 if any file failed, 0 otherwise.
- The non-terminal summary (`Formatted N files` / `Formatted N files, M failed`)
  is unchanged, and the terminal bar still shows `[pos/len]` advancing once per
  file with a best-effort file name.
- Per-file stderr messages are never interleaved or garbled within a line.
- An empty directory, a directory with no `*.java` files, and a single-file run
  behave exactly as today; single-file / stdin mode is untouched.
- The existing 15 CLI integration tests pass unchanged.
- New CLI tests cover a large multi-file directory where every file is formatted
  and failure counting with several failing files, plus a `RAYON_NUM_THREADS=1`
  versus default comparison showing identical outputs.
- README "Usage" and docs/requirements.md R42 document the parallel behaviour;
  the backlog index and changelog are updated on delivery.

# Implementation plan

## Approach

### 1. Dependency

Add `rayon = "1.12"` to `[workspace.dependencies]` in the root `Cargo.toml` and
reference it from `crates/cli/Cargo.toml` `[dependencies]` (`rayon = { workspace =
true }`). The crate is already resolved in `Cargo.lock` (transitive via
`criterion`) and cached, so no new download is needed.

### 2. Parallel loop (`crates/cli/src/main.rs`)

Replace the sequential `for path in files` loop in `format_directory` with
`files.par_iter().for_each(|path| { … })` (`use rayon::prelude::*`):

- the `--style` `JavaStyle` is borrowed (`&style`) and shared; it is `Sync`.
- `failed` becomes an `AtomicU64` incremented with `Relaxed` ordering and read
  once after the loop.
- a `report(&logger, &bar, message)` helper prints each per-file line under a
  `Mutex<()>` and inside `bar.suspend(…)`, so parallel lines never interleave
  and the bar is cleared first on a terminal.
- `bar.set_message` is best-effort (the most recently started file) and
  `inc(1)` advances once per file; both are thread-safe.
- collection, the write-if-changed rule, the summary line and the exit code are
  unchanged.

### 3. Tests (`crates/cli/tests/directory_mode.rs`)

Add a `run_with_env` helper and three tests: 64 files all formatted in one
parallel run; a run with three invalid-Java files counting `Formatted 1 file,
3 failed` and still formatting the good one; and a 32-file tree formatted with
`RAYON_NUM_THREADS=1` versus the default pool compared byte-for-byte.

### 4. Docs

- README "Usage": the directory-mode paragraph notes the parallel run and the
  `RAYON_NUM_THREADS` cap.
- docs/requirements.md R42: adds the parallel-execution clause.
- backlog index and changelog on delivery.

### 5. Verification

`cargo fmt --all -- --check`,
`cargo clippy --workspace --lib --bins --tests -- -D warnings`, and
`cargo test --workspace` (820 core golden tests, 18 CLI, 6 GUI).

## Steps

- [x] Add `rayon` to the workspace and cli manifests.
- [x] Parallelise `format_directory` with a shared atomic failure counter, a
      print mutex and best-effort bar messages; leave collection, writes,
      summary and exit codes unchanged.
- [x] Add the three CLI tests.
- [x] Update README "Usage" and docs/requirements.md R42.
- [x] Run fmt, clippy and the workspace tests.

## Closing

The directory loop now runs on rayon's global pool, so a run formats the
collected files on every available core. All 18 CLI integration tests (15
existing plus 3 new), 820 core golden tests (plus 9 core unit tests) and 6 GUI
tests pass; `cargo fmt --check` and `cargo clippy --workspace --lib --bins
--tests -- -D warnings` are clean. The changes are uncommitted in the worktree
(no commit hash); the changelog entry was added on 2026-09-11.
