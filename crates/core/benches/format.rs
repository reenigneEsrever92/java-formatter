//! Benchmark suite for `java-formatter`.
//!
//! Benchmarks formatting throughput on a small realistic fixture and on
//! synthetically generated source files of growing size, with both the default
//! style and the project's `codestyle.xml`. Also benchmarks the XML scheme
//! parser.
//!
//! Run with:
//!
//! ```sh
//! cargo bench
//! ```
//!
//! The criterion defaults are tuned down so a full run finishes quickly; pass
//! standard criterion flags (e.g. `-- --sample-size 100`) to override.

use std::fmt::Write;
use std::sync::OnceLock;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use java_formatter_core::config::{parse_codestyle, JavaStyle};
use java_formatter_core::formatter::format_java;

// ─────────────────────────────────────────────────────────────────────────────
// Styles
// ─────────────────────────────────────────────────────────────────────────────

fn default_style() -> &'static JavaStyle {
    static STYLE: OnceLock<JavaStyle> = OnceLock::new();
    STYLE.get_or_init(JavaStyle::default)
}

fn codestyle() -> &'static JavaStyle {
    static STYLE: OnceLock<JavaStyle> = OnceLock::new();
    STYLE.get_or_init(|| {
        let xml = include_str!("../../../codestyle.xml");
        parse_codestyle(xml).expect("codestyle.xml must parse")
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// A small, realistic file: imports, class, methods, `throws`, a record, an
/// enum and an interface.
const KITCHEN_SINK: &str = include_str!("../tests/java/kitchen_sink.java");

/// Synthetically generated source: `count` package-private classes, each with a
/// constructor, fields, control flow, `throws`, generics and a chained builder
/// method, followed by one record, one enum and one interface.
fn generated_source(count: usize) -> String {
    let mut src = String::new();

    for i in 0..count {
        let _ = write!(
            src,
            "class Generated{i} {{\n\
             \x20   private final String name;\n\
             \x20   private final int code;\n\
             \n\
             \x20   Generated{i}(String name, int code) {{\n\
             \x20       this.name = name;\n\
             \x20       this.code = code;\n\
             \x20   }}\n\
             \n\
             \x20   public String describe(int offset) throws java.io.IOException {{\n\
             \x20       if (code < 0) {{\n\
             \x20           throw new java.io.IOException(\"negative code\");\n\
             \x20       }}\n\
             \x20       int total = code + offset * 2 - 1;\n\
             \x20       java.util.List<String> parts = new java.util.ArrayList<>();\n\
             \x20       parts.add(name);\n\
             \x20       parts.add(String.valueOf(total));\n\
             \x20       return String.join(\"-\", parts);\n\
             \x20   }}\n\
             \n\
             \x20   public <T> T orDefault(T primary, T fallback) {{\n\
             \x20       return primary != null ? primary : fallback;\n\
             \x20   }}\n\
             \n\
             \x20   public static int compute(int a, int b, int c, int d, int e) {{\n\
             \x20       int x = (a + b) * (c - d);\n\
             \x20       int y = e == 0 ? 1 : e;\n\
             \x20       return x + y;\n\
             \x20   }}\n\
             }}\n\n"
        );
    }

    src.push_str(
        "record Pair(int left, int right) {\n\
         \x20   public Pair {\n\
         \x20       if (left > right) {\n\
         \x20           throw new IllegalArgumentException(\"left > right\");\n\
         \x20       }\n\
         \x20   }\n\
         \n\
         \x20   int sum() {\n\
         \x20       return left + right;\n\
         \x20   }\n\
         }\n\n\
         interface Worker {\n\
         \x20   void run() throws java.io.IOException;\n\
         }\n\n\
         enum Level {\n\
         \x20   LOW,\n\
         \x20   HIGH;\n\
         \n\
         \x20   boolean isHigh() {\n\
         \x20       return this == HIGH;\n\
         \x20   }\n\
         }\n",
    );

    src
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmarks
// ─────────────────────────────────────────────────────────────────────────────

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

fn bench_parse_codestyle(c: &mut Criterion) {
    let xml = include_str!("../../../codestyle.xml");

    c.bench_function("parse/codestyle", |b| {
        b.iter(|| black_box(parse_codestyle(black_box(xml)).expect("codestyle.xml must parse")))
    });
}

criterion_group! {
    name = format_benches;
    config = bench_config();
    targets = bench_format_realistic, bench_format_generated, bench_parse_codestyle
}
criterion_main!(format_benches);
