//! End-to-end benchmark of the CLI in directory mode: `java-formatter -d -r`.
//!
//! This measures the real binary — not the library — on a real, large Java
//! project (spring-framework by default, ~5.7k files; fetched by
//! `scripts/fetch-bench-corpus.sh` into `target/bench-corpus/`). The measured
//! pipeline is exactly what a user runs: process startup, recursive directory
//! walk, rayon-parallel per-file `read → format → compare → write`, and
//! shutdown.
//!
//! Two numbers are reported:
//!
//! * a single-shot **cold** run on a pristine copy of the corpus, where every
//!   file is rewritten (the "first format" of a project);
//! * a criterion-measured **warm** run once the tree already matches the
//!   formatter's style, where `write-if-changed` skips the writes (the
//!   "re-run the formatter in CI" scenario).
//!
//! When the corpus is absent the benchmark prints a hint and exits without
//! measuring, so a plain `cargo bench` stays green for everyone.
//!
//! Run with:
//!
//! ```sh
//! scripts/fetch-bench-corpus.sh
//! cargo bench -p java-formatter-cli --bench dir_mode
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tempfile::TempDir;

const CORPUS: &str = "spring-framework";

/// `target/bench-corpus/<CORPUS>` relative to the workspace root.
fn corpus_dir() -> Option<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/bench-corpus")
        .join(CORPUS);
    dir.is_dir().then_some(dir)
}

/// Count `*.java` files and total source bytes under `dir` (skipping hidden
/// dot-directories, like the CLI's own walk).
fn corpus_stats(dir: &Path) -> (usize, u64) {
    let mut files = 0usize;
    let mut bytes = 0u64;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                if !entry.file_name().to_string_lossy().starts_with('.') {
                    stack.push(path);
                }
            } else if path.extension().is_some_and(|e| e == "java") {
                files += 1;
                bytes += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    (files, bytes)
}

/// Recursive file copy (the corpus is formatted in place, so the benchmark
/// works on a private copy).
fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create work dir");
    let mut stack = vec![(from.to_path_buf(), to.to_path_buf())];
    while let Some((src, dst)) = stack.pop() {
        for entry in fs::read_dir(&src).expect("read dir").flatten() {
            let sp = entry.path();
            let dp = dst.join(entry.file_name());
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                fs::create_dir_all(&dp).expect("create dir");
                stack.push((sp, dp));
            } else {
                fs::copy(&sp, &dp).expect("copy file");
            }
        }
    }
}

/// Run the CLI binary in recursive directory mode on `dir`; panics with the
/// captured stderr when it fails.
fn run_cli(bin: &str, dir: &Path) {
    let out = Command::new(bin)
        .args(["-d", dir.to_str().expect("utf-8 corpus path"), "-r"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn java-formatter");
    assert!(
        out.status.success(),
        "java-formatter exited with {:?}: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    // The CLI summarises non-tty runs on stderr ("Formatted N files").
    if let Some(line) = String::from_utf8_lossy(&out.stderr)
        .lines()
        .find(|l| l.contains("Formatted"))
    {
        eprintln!("    {line}");
    }
}

fn bench_dir_mode(c: &mut Criterion) {
    let Some(corpus) = corpus_dir() else {
        eprintln!(
            "bench corpus not found ({}); run `scripts/fetch-bench-corpus.sh` first",
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/bench-corpus")
                .display()
        );
        return;
    };

    let (file_count, byte_count) = corpus_stats(&corpus);
    eprintln!(
        "corpus: {} java files, {:.1} MiB in {}",
        file_count,
        byte_count as f64 / (1024.0 * 1024.0),
        corpus.display()
    );

    let bin = env!("CARGO_BIN_EXE_java_formatter");

    // Work on a private copy: the CLI rewrites files in place.
    let work = TempDir::new().expect("temp dir");
    copy_tree(&corpus, work.path());

    // Cold: the first run formats (and rewrites) every file.
    eprintln!(
        "cold run (pristine copy, all {} files rewritten):",
        file_count
    );
    let start = std::time::Instant::now();
    run_cli(bin, work.path());
    eprintln!("    took {:?}", start.elapsed());

    // Warm: the tree now matches the formatter's style; write-if-changed
    // leaves the files untouched. This is the stable, reproducible number.
    let mut group = c.benchmark_group("cli/dir_mode");
    group.throughput(Throughput::Elements(file_count as u64));
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(30));
    group.bench_function("reformat_noop", |b| b.iter(|| run_cli(bin, work.path())));
    group.finish();
}

criterion_group! {
    name = dir_mode_benches;
    config = Criterion::default();
    targets = bench_dir_mode
}
criterion_main!(dir_mode_benches);
