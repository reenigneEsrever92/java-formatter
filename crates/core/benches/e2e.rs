//! End-to-end benchmark: the full pipeline the CLI runs per file — read the
//! source from disk, parse and format it, compare, and write the result back.
//!
//! This measures `read → format_java_diagnosed → write-if-changed → fs::write`
//! against real files in a temp directory, with the OS page cache warm (the
//! scenario of re-running the formatter on a repo you just checked out or
//! generated). Compare against the in-memory numbers in `format.rs` to see how
//! much of a file's wall time is I/O vs. formatting.
//!
//! Run with:
//!
//! ```sh
//! cargo bench -p java-formatter-core --bench e2e
//! ```

mod common;

use std::fs;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tempfile::TempDir;

use java_formatter_core::formatter::format_java_diagnosed;

use common::{codestyle, default_style, generated_source};

/// One end-to-end pipeline iteration, mirroring `cli::format_directory`'s
/// per-file body: read the source, format (with diagnostics), write only when
/// the output differs. The generated fixtures are deliberately not
/// pre-formatted, so the write always fires and the I/O is fully exercised.
fn pipeline(
    in_path: &std::path::Path,
    out_path: &std::path::Path,
    style: &java_formatter_core::config::JavaStyle,
) {
    let source = fs::read_to_string(in_path).expect("fixture file must be readable");

    let (formatted, _issues) = format_java_diagnosed(&source, style);

    // Format-in-place semantics: touch the file system only on change.
    if formatted != source {
        fs::write(out_path, &formatted).expect("output file must be writable");
    }
}

fn bench_e2e(c: &mut Criterion) {
    for &size in &[50usize, 200, 600] {
        let src = generated_source(size);
        let bytes = src.len() as u64;

        let dir = TempDir::new().expect("temp dir must be creatable");
        let in_path = dir.path().join("input.java");
        let out_path = dir.path().join("output.java");
        fs::write(&in_path, &src).expect("fixture must be writable");

        let mut group = c.benchmark_group("e2e/read_format_write");
        group.throughput(Throughput::Bytes(bytes));

        group.bench_with_input(
            BenchmarkId::new("default", size),
            &(in_path.clone(), out_path.clone()),
            |b, (in_path, out_path)| {
                let style = default_style();
                b.iter(|| {
                    pipeline(in_path, out_path, style);
                    black_box(());
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("codestyle", size),
            &(in_path.clone(), out_path.clone()),
            |b, (in_path, out_path)| {
                let style = codestyle();
                b.iter(|| {
                    pipeline(in_path, out_path, style);
                    black_box(());
                });
            },
        );
        group.finish();
    }
}

criterion_group! {
    name = e2e_benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(std::time::Duration::from_millis(100))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = bench_e2e
}
criterion_main!(e2e_benches);
