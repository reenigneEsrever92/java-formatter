---
type: ChangeRequest
kind: refactor
title: Split the crate into a core/cli/gui workspace
description: Restructure the single java-formatter crate into a Cargo workspace with java-formatter-core (lib), java-formatter-cli (binary) and a java-formatter-gui skeleton under crates/.
state: done
verified: { by: maintainer, at: 2026-09-03T11:34:40Z }
priority: medium
tags: [dev, workspace]
owner: maintainer
---

# Problem

The project is a single crate `java-formatter` (root `Cargo.toml`) that mixes
three concerns: the formatting library (`src/config.rs`, `src/formatter.rs`),
the CLI binary (`src/main.rs`), and — next — a desktop GUI to edit code
styles. A single package cannot be a clean dependency target for multiple
binaries, and the upcoming egui codestyle editor (see the separate feature
request) needs the library as a standalone crate it can depend on without
pulling in the CLI. The repository also lacks a place for the GUI, and the
root-level `tests/` / `benches/` / `examples/` cannot move to a virtual
workspace root. A Cargo workspace with dedicated crates gives each surface a
home and keeps the library dependency-free of the binaries.

# Proposal

Restructure the repository into a Cargo workspace under `crates/`:

- `crates/core` — package `java-formatter-core`, a lib crate holding the
  current `src/lib.rs` content (`config` + `formatter` modules), plus the
  integration tests, fixtures, benchmarks and the `tree_dump` example that
  belong with the library.
- `crates/cli` — package `java-formatter-cli`, a binary crate holding the
  current `src/main.rs` CLI; its binary is explicitly named `java-formatter`
  so every README usage example and the documented CLI surface stay valid.
- `crates/gui` — package `java-formatter-gui`, a binary crate created as a
  minimal stub in this change so the three-crate workspace structure exists;
  the egui editor itself is delivered by the separate feature request.

The root `Cargo.toml` becomes a virtual workspace manifest (`[workspace]`
members `crates/*`, shared dependency versions via `[workspace.dependencies]`).
The root `codestyle.xml` sample stays at the root (README examples reference
it; core tests/benches reach it via relative `include_str!` paths).

This is a pure move: no formatting behaviour, CLI surface, or test outcome
changes.

Docs touched: `README.md` (Architecture section, and any crate references in
the usage text), `docs/overview.md` (library/binary split wording),
`docs/requirements.md` (R9 maintainability wording and the milestones note),
`docs/dev/changelog.md` on completion.

# Decisions

- **Two change requests, refactor first.** Decided with the user on 2026-09-03:
  the workspace split is captured as this `refactor` request and the egui
  codestyle editor as a separate `feature` request, so each can be planned and
  reviewed independently; the feature depends on this split.
- **Package names differ from folder names.** Folders `crates/core`,
  `crates/cli`, `crates/gui` hold packages `java-formatter-core`,
  `java-formatter-cli`, `java-formatter-gui`. Namespaced names give distinct
  import namespaces (`java_formatter_core`, …) and avoid a lib crate literally
  named `core`, which can collide with Rust's built-in `core` crate.
- **CLI binary keeps its name.** The cli package sets an explicit `[[bin]]`
  name `java-formatter`; the GUI binary is named `java-formatter-gui`.
- **Tests, benches and examples follow the library into core.** Integration
  tests and `tests/java/` fixtures move to `crates/core/tests/`, the Criterion
  suite to `crates/core/benches/`, and `examples/tree_dump.rs` (raw
  tree-sitter exploration) to `crates/core/examples/` with the tree-sitter
  dev-dependencies it needs; crate imports in tests/benches are renamed from
  `java_formatter` to `java_formatter_core`.
- **Root becomes a virtual workspace.** The root `Cargo.toml` drops its
  `[package]` section and declares workspace members; `Cargo.lock` is
  regenerated for the multi-package workspace.
- **No behavioural change in this request.** Formatting output, CLI flags,
  exit codes and stderr warnings are byte-identical; the moved suite must pass
  unchanged.

# Acceptance criteria

- `cargo build` at the workspace root builds the `java-formatter-core` lib,
  the `java-formatter` binary and the `java-formatter-gui` stub binary.
- `cargo test` passes: the full integration suite (moved to
  `crates/core/tests/`, fixtures under `crates/core/tests/java/`) and any unit
  tests, with the same results as before the split.
- `cargo bench` compiles and runs the moved Criterion suite from
  `crates/core/benches/`.
- `java-formatter --style codestyle.xml <file>` behaves exactly as before —
  same flags, same output, same exit codes, same stderr warnings — and
  reading from stdin/`-` still works.
- The `java-formatter-gui` binary exists and runs as a stub.
- The README Architecture section and `docs/` references to the single-crate
  `src/` layout are updated; a changelog entry is appended when the change
  ships (fawi-implement).

# Implementation plan

## Approach

The single `java-formatter` package is split into a virtual Cargo workspace
with three members under `crates/`. The root `Cargo.toml` drops its
`[package]` / `[[bin]]` / dependency sections and becomes a `[workspace]`
manifest (members `crates/core`, `crates/cli`, `crates/gui`; `resolver = "2"`),
sharing versions through `[workspace.package]` (version, edition) and
`[workspace.dependencies]`. `codestyle.xml` stays at the root; the moved
`crates/core` tests and benches reach it via adjusted relative
`include_str!` paths (`../../../codestyle.xml`), and fixture paths inside
`crates/core/tests/` (`java/…`) keep working because the fixtures move with
the tests.

`crates/core` is the lib package `java-formatter-core` (lib name
`java_formatter_core`): `src/lib.rs`, `src/config.rs`, `src/formatter.rs` move
verbatim from the old `src/`; the integration suites (`tests/*.rs`,
`tests/common/`, fixtures under `tests/java/`) move to `crates/core/tests/`;
the Criterion suite moves to `crates/core/benches/`; the `tree_dump` example
is not present in this repository (the `examples/` folder is empty/untracked)
so there is nothing to move there. Its `Cargo.toml` carries the library
dependencies (quick-xml, serde, tree-sitter, tree-sitter-java), the criterion
dev-dependency, and the `[[bench]] format` declaration. `crates/cli` is the
binary package `java-formatter-cli` whose `[[bin]]` is explicitly named
`java-formatter` (so the documented CLI surface and README usage stay valid);
it depends on clap and on `java-formatter-core`. `crates/gui` is the binary
package `java-formatter-gui` with a stub `main.rs` printing a
not-implemented notice, so the three-crate workspace exists and builds; the
egui editor is delivered by the separate feature request.

No formatting, config, or CLI behaviour changes: `src/` code is a pure move,
test/bench `java_formatter::` imports become `java_formatter_core::`, and
`main.rs` keeps its clap surface, stderr warnings and exit codes.

Docs touched: `README.md` (crate-link, Architecture, Testing, Benchmarking),
`docs/overview.md` (workspace wording), `docs/requirements.md` (R9 wording,
milestones note), `docs/dev/backlog/index.md` (row state),
`docs/dev/changelog.md`.

## Steps

- [x] Replace the root `Cargo.toml` with a virtual workspace manifest
      (members, `[workspace.package]`, `[workspace.dependencies]`); regenerate
      `Cargo.lock` for the multi-package workspace.
- [x] Create `crates/core` (package `java-formatter-core`): move `src/lib.rs`,
      `src/config.rs`, `src/formatter.rs` to `crates/core/src/`, the
      integration suites and `tests/java/` fixtures to `crates/core/tests/`,
      and `benches/format.rs` to `crates/core/benches/`; write its `Cargo.toml`
      (lib deps, criterion dev-dep, `[[bench]] format` harness=false).
      (`examples/` is empty in this repo — no `tree_dump.rs` to move.)
- [x] Create `crates/cli` (package `java-formatter-cli`): move `src/main.rs`
      to `crates/cli/src/main.rs`, set `[[bin]] name = "java-formatter"`, and
      depend on clap + `java-formatter-core` (AC4: binary name, flags, output,
      exit codes unchanged).
- [x] Create `crates/gui` (package `java-formatter-gui`) with a stub
      `src/main.rs` binary named `java-formatter-gui` (AC5: exists and runs).
- [x] Rename crate imports in moved tests/benches/main (`java_formatter::` →
      `java_formatter_core::`) and fix the `include_str!` paths to the root
      `codestyle.xml` in `crates/core/tests/config.rs` and
      `crates/core/benches/format.rs` (AC2, AC3).
- [x] `cargo build` (workspace root), `cargo test`, and `cargo bench`;
      manually verify `java-formatter --style codestyle.xml <file>`, stdin/`-`,
      parse-warning stderr behaviour and `java-formatter-gui` (AC1–AC5).
- [x] Update the README and `docs/` (`overview.md`, `requirements.md` R9 and
      milestones, backlog index row) for the new layout, and append a changelog
      entry when the change ships (fawi-implement).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).
