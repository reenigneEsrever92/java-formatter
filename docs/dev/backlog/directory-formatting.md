---
type: ChangeRequest
kind: feature
title: Format all Java files in a directory in place (-d/--dir, -r/--recursive)
description: Add -d/--dir DIR and -r/--recursive flags so the CLI formats every *.java file in a directory in place, logging per-file failures to stderr, continuing past them, and exiting 1 when any file failed.
state: done
verified: { by: maintainer, at: 2026-09-11T00:00:00Z }
priority: medium
tags: [dev, cli]
owner: maintainer
---

# Problem

The CLI formats exactly one input per invocation: the positional `FILE`
argument or stdin, both writing the formatted source to stdout. Formatting a
whole project therefore means scripting a loop of
`java-formatter Foo.java > Foo.formatted.java && mv Foo.formatted.java Foo.java`
— the README "Update a file" example documents exactly this dance — and there
is no way to say "apply this team scheme to all the Java in this directory".
Applying a scheme to a tree is manual, error-prone, and hard to script.

# Proposal

Add two flags to `crates/cli/src/main.rs`, leaving the core library untouched:

- `-d, --dir <DIR>` — format every `*.java` file in `DIR` **in place**,
  rewriting each file with its formatted source instead of printing to stdout.
- `-r, --recursive` — with `--dir`, extend the walk into subdirectories;
  without `--dir` it is a usage error. Directory names starting with `.`
  (e.g. `.git`, `.idea`) are skipped during the recursive walk. Non-`.java`
  files are silently ignored.

The style file (`--style`) is parsed once and shared across all files. In
directory mode nothing is written to stdout; per-file problems are logged to
stderr. A failing file — unreadable, invalid Java (parse-error warning), or an
in-place write failure — does not stop the run: the remaining files are still
formatted, and the process exits `1` if any file failed, `0` otherwise. The
invalid-Java file is still rewritten with best-effort output and warned about
per file, mirroring single-file R15 behaviour except for the exit code (see
Decisions). The existing positional `FILE` / stdin contract is unchanged.

Directory traversal uses `std::fs::read_dir` only — no new dependency —
with entries sorted by file name for deterministic order, and directory
symlinks not followed (avoids cycles).

Docs touched: README "Usage" (document `-d` / `--dir` and `-r` / `--recursive`
with examples), docs/requirements.md (a new use case and requirement row,
including the recorded divergence from R15's exit codes), docs/dev/backlog/index.md
(this entry, on delivery), and docs/dev/changelog.md on delivery.

# Decisions

- **In-place rewriting (`-d` formats files, not stdout) — user choice.**
  Directory mode rewrites each `*.java` file with its formatted source; stdout
  stays empty so scripts can rely on it. This is what "format all java files
  in a given directory" means operationally, and it matches how IntelliJ
  applies a scheme to a project.
- **Dedicated `-d` / `--dir` flag, positional `FILE` unchanged — user
  choice.** A new flag keeps the single-file / stdin contract byte-for-byte
  intact, keeps `--help` self-explanatory, and lets clap express the mutual
  exclusion (`--dir` conflicts with `FILE`).
- **Recursion is opt-in via `-r` / `--recursive` — user choice.** The default
  is a flat walk of the given directory only. Recursion includes
  subdirectories but skips hidden dot-directories (`.git`, `.idea`, …) so
  `-d . -r` does not wander into VCS metadata. Non-`.java` files are skipped
  silently in both modes.
- **Continue past failures, exit 1 if any file failed — user choice.**
  Per-file errors are logged to stderr with the file's path and formatting
  continues; the final exit status is 1 if any file failed, 0 otherwise.
  An empty directory or a directory with no `.java` files is success (nothing
  to do, exit 0).
- **Invalid Java counts as a failure in directory mode — user choice; a
  recorded divergence from R15.** Single-file mode keeps R15's contract
  (parse errors warn on stderr and exit 0, best-effort output to stdout). In
  directory mode the same file is rewritten in place with best-effort output
  and warned about, but it also sets the exit status to 1 — a batch run needs
  a non-zero status so an operator notices files that were not cleanly
  formatted. The divergence is documented in docs/requirements.md.
- **No new dependency for the walk — `std::fs::read_dir` with sorted entries.**
  The traversal is simple (flat walk, or recursion skipping dot-directories)
  and sorting makes the per-file order deterministic and testable. `walkdir`
  was considered but rejected to keep the dependency surface unchanged,
  consistent with the project's minimal-dependency stance.
- **`-r` without `-d` is a usage error** (clap `requires`): recursion has no
  meaning for a single file or stdin.

# Acceptance criteria

- `java-formatter -d DIR [--style codestyle.xml]` rewrites every `*.java`
  file directly inside `DIR` in place; files in subdirectories and
  non-`.java` files are untouched; stdout is empty; exit 0.
- `java-formatter -d DIR -r` also rewrites `*.java` files in subdirectories
  and skips hidden dot-directories (e.g. `.git`, `.idea`).
- `--style` is applied once across the run, not reparsed per file.
- A run containing an unreadable file still formats every other file, logs an
  error naming that file to stderr, and exits 1.
- A run containing an invalid-Java file rewrites all files (the invalid one
  best-effort, like single-file R15), warns per file on stderr, and exits 1.
- A run where an in-place write fails (e.g. read-only file) logs to stderr,
  formats the remaining files, and exits 1.
- `-r` / `--recursive` without `-d` / `--dir` is rejected with a clap usage
  error (exit 2); `-d` together with a positional `FILE` is rejected.
- Single-file and stdin behaviour is unchanged: same flags, formatted output
  to stdout, parse warnings to stderr, exit 0.
- Lightweight CLI-level integration tests cover the in-place rewrite, the
  recursive walk with dot-directory skipping, and the failure exit codes.
- README "Usage" documents the new flags with examples; docs/requirements.md
  gains the new use case and requirement row (with the R15 exit-code
  divergence recorded); the backlog index gains this entry and the changelog
  records the delivery.

# Implementation plan

## Approach

### 1. CLI flags (`crates/cli/src/main.rs`)

Extend the existing clap `Args` derive — the project already uses clap 4.6
derive (`workspace.dependencies`), so no new runtime dependency:

```rust
#[arg(conflicts_with = "dir")]
file: Option<PathBuf>,

/// Format every *.java file in this directory in place.
#[arg(short, long, value_name = "DIR")]
dir: Option<PathBuf>,

/// Recurse into subdirectories (requires --dir).
#[arg(short, long, requires = "dir")]
recursive: bool,
```

- `-d` / `--dir` and `-r` / `--recursive` do not collide with clap's generated
  `-h` / `--help`.
- `conflicts_with = "dir"` (arg id = field name) rejects `-d` together with a
  positional `FILE`; `requires = "dir"` makes `-r` without `-d` a usage error
  (clap exits 2 with a message — acceptance criterion "rejected with a clap
  usage error").
- Extract the style-loading block (the `args.style` match) into a small
  `fn load_style(style: &Option<PathBuf>) -> config::JavaStyle` helper used by
  both branches; the single-file branch keeps today's error messages and
  order.

### 2. Directory traversal (`crates/cli/src/main.rs`)

A `fn collect_java_files(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>)`
built on `std::fs::read_dir` only:

- `read_dir` the given directory; on error (missing path, not a directory)
  return the error for `main` to print to stderr and exit 1.
- Collect entries into a `Vec`, sort by file name (**deterministic order**),
  then recurse depth-first.
- Per entry (via `entry.file_type()`, which does not follow symlinks):
  - directory → recurse when `recursive` and the name does not start with `.`
    (hidden dot-directories skipped, e.g. `.git`, `.idea`);
  - symlink → follow with `fs::metadata`; if it points to a directory, skip
    (directory symlinks are never followed — avoids cycles); if it points to a
    file with a `.java` extension, include;
  - regular file → include when the extension is `java` (`path.extension() ==
Some("java")`); everything else is silently ignored.

No `walkdir` (already decided: minimal dependencies); the recursion is a dozen
lines and the dot-directory rule is trivial to honour.

### 3. Directory-mode main flow (`crates/cli/src/main.rs`)

Branch on `args.dir` before the existing single-file / stdin path, which stays
byte-for-byte unchanged:

```text
if let Some(dir) = &args.dir {
    let style = load_style(&args.style);            // parsed once, shared
    let mut files = vec![];
    match collect_java_files(dir, args.recursive, &mut files) {
        Err(msg)  => { eprintln!("error: {msg}"); process::exit(1); }
        Ok(())    => {}
    }
    let mut failed = false;
    for path in files {
        // read
        let source = match fs::read_to_string(&path) {
            Err(e) => { eprintln!("error: could not read '{}': {}", path.display(), e);
                        failed = true; continue; }
            Ok(s)  => s,
        };
        // format + diagnose
        let (formatted, issues) = formatter::format_java_diagnosed(&source, &style);
        for issue in &issues { eprintln!("warning: {}: {}", path.display(), issue); }
        if !issues.is_empty() {
            eprintln!("warning: {}: input is not valid Java; output is best-effort",
                      path.display());
            failed = true;                          // R15 divergence, per decisions
        }
        // write back only when changed (preserves mtime of already-formatted
        // files, matching clang-format -i; also makes the R6 no-op case a no-write)
        if formatted != source {
            if let Err(e) = fs::write(&path, &formatted) {
                eprintln!("error: could not write '{}': {}", path.display(), e);
                failed = true;
            }
        }
    }
    process::exit(if failed { 1 } else { 0 });
}
```

- No stdout output in directory mode (contract: stdout stays empty).
- `--style` is loaded once before the loop and shared (criterion: applied
  once across the run).
- A directory with no `.java` files → empty `files`, `failed == false`, exit 0
  (decision: success when nothing to do).

### 4. CLI integration tests (new `crates/cli/tests/directory_mode.rs`)

The cli crate has no tests today. Add integration tests that drive the built
binary through `std::process::Command`:

- The binary path comes from `env!("CARGO_BIN_EXE_java-formatter")` — Cargo
  exposes it to integration tests for the `[[bin]]` target with no extra
  machinery.
- Add `tempfile = "3"` to `workspace.dependencies` and reference it from
  `crates/cli/Cargo.toml` `[dev-dependencies]` (`tempfile = { workspace = true }`)
  for panic-safe, cross-platform temp directories. A dev-dependency does not
  affect the shipped binary and matches the existing pattern of criterion in
  core's dev-dependencies.
- Use input fixtures the **default** style demonstrably changes (verified once
  against the single-file mode) so the in-place rewrite is observable; for the
  `--style` test use a scheme with a visible option (e.g. tab indentation).

Test cases (each maps to an acceptance criterion):

1. **Flat in-place rewrite** — dir with two `.java` files, a `.txt`, and a
   subdirectory containing a `.java`; run `-d dir`; assert: both top-level
   `.java` files are reformatted, the `.txt` and the subdirectory file are
   untouched, stdout is empty, exit 0.
2. **Write-only-if-changed** — run `-d dir` twice; second run exit 0 and does
   not error (idempotency, R6); asserted via content equality.
3. **Recursive walk + dot-directory skip** — `-d dir -r` formats nested
   `.java` files and skips a `.git`-style dot-directory (and its files).
4. **`--style` applied across the run** — a scheme with a visible option
   changes every file in the directory.
5. **Unreadable file (cfg(unix))** — chmod 000 one file; the others are still
   formatted, stderr names the failing file, exit 1.
6. **Invalid Java** — one file with broken syntax; all files are still
   rewritten (the bad one best-effort), stderr carries the per-file warning,
   exit 1.
7. **Write failure (cfg(unix))** — chmod 0444 one file; read succeeds, write
   fails; stderr names it, other files formatted, exit 1.
8. **Missing / non-directory `-d` path** — stderr error, exit 1.
9. **Usage errors** — `-r` without `-d` exits 2 with stderr output; `-d`
   together with a positional `FILE` exits 2.
10. **Single-file mode unchanged** — `java-formatter Foo.java` still writes
    the formatted source to stdout and exits 0, parse warnings on stderr.

### 5. Docs

- **README "Usage"** — add `-d, --dir <DIR>` and `-r, --recursive` to the
  options block; add the `[FILE]`-vs-`--dir` mutual exclusion; add two
  examples (flat and recursive in-place formatting); note stdout stays empty
  in directory mode and the exit-1 semantics.
- **docs/requirements.md** — new use case UC6 («Format a directory in place»)
  tied to U1; new requirement row R42 (functional, medium) describing the
  in-place rewrite, the recursive walk skipping hidden dot-directories, the
  shared `--style`, the per-file stderr logging without stopping the run, and
  the exit-1-if-any-file-failed semantics; append one sentence to the
  「Invalid-Java behaviour (R15)」note recording the directory-mode divergence
  (parse-error file still rewritten best-effort but counts as a failure → exit 1).
- **docs/dev/backlog/index.md** — add this entry to the table.
- The changelog entry is appended on delivery by `fawi-implement`, per that
  skill.

### 6. Verification

The project's CI gates (github-ci-pipeline):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --lib --bins --tests -- -D warnings
cargo test --workspace
```

plus a manual smoke run of `-d` / `-d -r` (and the usage-error cases) against
a scratch tree. The 455 core goldens must stay byte-identical — core is
untouched.

## Steps

- [x] Add `-d` / `--dir` (conflicts with `file`) and `-r` / `--recursive`
      (requires `dir`) to `Args` in `crates/cli/src/main.rs`; extract the
      style-loading block into a `load_style` helper.
- [x] Implement `collect_java_files` (`std::fs::read_dir`, sorted entries,
      depth-first recursion, hidden dot-directories skipped, `.java`
      extension filter, directory symlinks not followed).
- [x] Implement the directory-mode `main` flow: load style once, read →
      format → write-if-changed per file, per-file read/parse/write failures
      logged to stderr with the file path, continue past failures, exit 1 if
      any file failed else 0, nothing on stdout; leave the single-file /
      stdin path byte-for-byte unchanged.
- [x] Add `tempfile` to `workspace.dependencies` and to
      `crates/cli/Cargo.toml` dev-dependencies; add
      `crates/cli/tests/directory_mode.rs` covering the ten test cases above
      (in-place rewrite, recursion + dot-directory skip, `--style` run,
      unreadable / invalid-Java / read-only failure paths, missing-dir,
      usage errors, single-file unchanged).
- [x] Update README "Usage" (flags, mutual exclusion, examples, stdout / exit
      semantics); add UC6 + requirement row R42 (with the R15-divergence
      note) to docs/requirements.md; add this entry to docs/dev/backlog/index.md.
- [x] Run `cargo fmt --all -- --check`, `cargo clippy --workspace --lib --bins
    --tests -- -D warnings` and `cargo test --workspace`; smoke-test `-d` /
      `-d -r` and the usage-error cases on a scratch tree.

## Closing

All 12 CLI integration tests, the 817 core golden tests, and the 6 GUI tests
pass; `cargo fmt --check` and `cargo clippy --workspace --lib --bins --tests
-- -D warnings` are clean. The changes are uncommitted in the worktree (no
commit hash); the changelog entry was added on 2026-09-11.
