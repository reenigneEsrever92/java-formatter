//! Benchmark suite for `java-formatter`.
//!
//! Benchmarks formatting throughput on a small realistic fixture and on
//! synthetically generated source files of growing size, with both the default
//! style and the project's `codestyle.xml`. The same fixtures and styles are
//! shared with the end-to-end bench (`e2e.rs`, which adds file I/O around the
//! same core call).
//!
//! Run with:
//!
//! ```sh
//! cargo bench
//! ```
//!
//! The criterion defaults are tuned down so a full run finishes quickly; pass
//! standard criterion flags (e.g. `-- --sample-size 100`) to override.

mod common;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use java_formatter_core::formatter::format_java;

use common::{codestyle, default_style, generated_source, KITCHEN_SINK};

fn bench_config() -> Criterion {
    Criterion::default()
        .sample_size(20)
        .warm_up_time(std::time::Duration::from_millis(100))
        .measurement_time(std::time::Duration::from_secs(2))
}

fn bench_format_realistic(c: &mut Criterion) {
    let src = KITCHEN_SINK;

    let mut group = c.benchmark_group("format/realistic");
    group.throughput(Throughput::Bytes(src.len() as u64));
    group.bench_function("default", |b| {
        let style = default_style();
        b.iter(|| format_java(black_box(src), style));
    });
    group.bench_function("codestyle", |b| {
        let style = codestyle();
        b.iter(|| format_java(black_box(src), style));
    });
    group.finish();
}

fn bench_format_generated(c: &mut Criterion) {
    let mut group = c.benchmark_group("format/generated");

    for &size in &[50usize, 200, 600] {
        let src = generated_source(size);
        let bytes = src.len() as u64;
        group.throughput(Throughput::Bytes(bytes));

        group.bench_with_input(BenchmarkId::new("default", size), &src, |b, src| {
            let style = default_style();
            b.iter(|| format_java(black_box(src), style));
        });
        group.bench_with_input(BenchmarkId::new("codestyle", size), &src, |b, src| {
            let style = codestyle();
            b.iter(|| format_java(black_box(src), style));
        });
    }
    group.finish();
}

criterion_group! {
    name = format_benches;
    config = bench_config();
    targets = bench_format_realistic, bench_format_generated
}
criterion_main!(format_benches);
