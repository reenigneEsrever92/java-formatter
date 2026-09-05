---
type: ChangeRequest
kind: feature
title: Set up a GitHub Actions CI pipeline (fmt, clippy, tests on an OS matrix)
description: Add a .github/workflows/ci.yml workflow running fmt check, clippy with -D warnings and the workspace test suite on ubuntu/macos/windows × stable, with the cleanup needed to start green.
state: done
verified: { by: maintainer, at: 2026-09-05T15:20:00Z }
priority: medium
tags: [dev, ci]
owner: maintainer
---

# Problem

The repository has no CI at all: there is no `.github/` directory and no
workflow file, so nothing verifies changes automatically. The quality gates
that would normally run in CI have already drifted: `cargo fmt --check` fails
on 34 files under `crates/core/tests/options/` (test files rustfmt wants to
reflow), `cargo clippy` reports three unique warnings in `crates/core/src`
(six with duplicates), and the 455-golden integration suite is only ever run
locally — a regression could land without anyone noticing until a developer
happens to run `cargo test`. (Separately, `crates/core/benches/format.rs`
references a `tests/java/kitchen_sink.java` fixture that no longer exists, so
`cargo bench` / `--all-targets` builds are broken at HEAD; the owner is fixing
that independently.)

# Proposal

Add `.github/workflows/ci.yml` running three checks on an OS matrix
(`ubuntu-latest`, `macos-latest`, `windows-latest`) × stable Rust
(`dtolnay/rust-toolchain@stable`), with `Swatinem/rust-cache` per matrix cell:

- `cargo fmt --check` (formatting gate, ubuntu cell only — rustfmt output is
  platform-independent);
- `cargo clippy --workspace --lib --bins --tests -- -D warnings` (lint gate on
  every OS, since target-specific cfg can surface different warnings);
- `cargo test --workspace` (the 455-golden suite plus the cli/gui build, on
  every OS).

Benches are deliberately not compiled: the missing `kitchen_sink.java` fixture
is the owner's follow-up, and neither `cargo test` nor
`cargo clippy --lib --bins --tests` touches the bench target. Triggers: push
to `main`, pull requests, and `workflow_dispatch` for manual runs.

The pipeline must be green on its first run, so this request also includes the
one-time cleanup the gates would flag today: run `cargo fmt` over the 34
option test files and fix the three unique clippy warnings in `crates/core/src`
— formatting/lint-only, with every golden staying byte-identical.

Docs touched: README "Testing" section (document the CI checks and add a
workflow status badge), docs/requirements.md (a new dev/maintainability
requirement row tied to R9), docs/dev/backlog/index.md (this entry), and
docs/dev/changelog.md on delivery.

# Decisions

- **Full quality gates (owner choice).** CI covers `cargo fmt --check`,
  `cargo clippy` with `-D warnings` and `cargo test --workspace`, and the
  request includes the one-time cleanup that makes them green: rustfmt over
  the 34 unformatted option test files, and the three unique clippy warnings
  (six with duplicates) in `crates/core/src`. Without the cleanup the fmt gate
  would fail on the first run, so the cleanup is in scope rather than
  deferred.
- **Benches stay out of CI (owner choice).** The missing
  `tests/java/kitchen_sink.java` fixture (referenced from
  `crates/core/benches/format.rs`) is the owner's separate follow-up; this
  request leaves it alone and the workflow avoids compiling benches
  (`cargo test` without `--all-targets`, clippy with explicit
  `--lib --bins --tests`). Once the fixture is restored, the workflow should
  switch to `--all-targets` — recorded as a follow-up.
- **OS matrix × stable, not a toolchain matrix.** The project pins "Rust
  (stable)" (README "Building"; docs/requirements.md technology choices) and
  declares no MSRV, so the matrix axis is the OS (`ubuntu-latest`,
  `macos-latest`, `windows-latest`) at stable. All three build the workspace:
  the lockfile already resolves each platform backend (`windows-sys` on
  Windows, `objc2`/`block2` on macOS, wayland/xdg-portal on Linux), and the
  GUI (`eframe`/`rfd`) builds on Linux without extra system packages
  (verified locally).
- **Triggers.** `push` to `main`, `pull_request`, and `workflow_dispatch`
  (owner approved): PR runs cover incoming changes, the push run guards
  `main`, and manual dispatch allows ad-hoc runs.
- **Caching.** `Swatinem/rust-cache` per matrix cell keeps the three-OS,
  three-job pipeline fast; correctness never depends on it — a cold run must
  pass too.
- **No release build job.** The test job compiles every target CI cares about
  in debug; a release job would roughly double compile time for little signal
  and can be added with a release workflow later.

# Acceptance criteria

- `.github/workflows/ci.yml` exists and runs on `push` to `main`,
  `pull_request` and `workflow_dispatch`, on the `ubuntu-latest` /
  `macos-latest` / `windows-latest` × stable matrix with per-cell rust-cache.
- `cargo fmt --check` passes: the 34 option test files are rustfmt-formatted.
- `cargo clippy --workspace --lib --bins --tests -- -D warnings` passes on all
  three OSes: the three unique core warnings are fixed.
- `cargo test --workspace` passes on all three OSes — the same 455 tests and
  byte-identical goldens as before the cleanup (the cleanup is
  formatting/lint-only; no golden is regenerated).
- The pipeline is green on a fresh cold checkout of `main` (no reliance on
  locally cached artifacts; `cargo bench` is not part of CI).
- README "Testing" documents the CI checks and carries a workflow status
  badge; docs/requirements.md gains the new requirement row; the backlog
  index and changelog are updated.
