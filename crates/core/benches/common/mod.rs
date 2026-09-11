//! Shared fixtures and styles for the benchmark suite.
//!
//! Both [`format`](crate) benches (`format.rs` — pure in-memory formatting
//! throughput — and `e2e.rs` — the full read/format/write pipeline) exercise
//! the same synthetic Java sources and style schemes so their numbers are
//! directly comparable.

use std::fmt::Write;
use std::sync::OnceLock;

use java_formatter_core::config::{parse_codestyle, JavaStyle};

// ─────────────────────────────────────────────────────────────────────────────
// Styles
// ─────────────────────────────────────────────────────────────────────────────

pub fn default_style() -> &'static JavaStyle {
    static STYLE: OnceLock<JavaStyle> = OnceLock::new();
    STYLE.get_or_init(JavaStyle::default)
}

pub fn codestyle() -> &'static JavaStyle {
    static STYLE: OnceLock<JavaStyle> = OnceLock::new();
    STYLE.get_or_init(|| {
        let xml = include_str!("../../../../codestyle.xml");
        parse_codestyle(xml).expect("codestyle.xml must parse")
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// A small, realistic file: package + imports, a class with methods and
/// `throws`, a record, an enum and an interface.
#[allow(dead_code)] // used by the `format` bench, a separate binary from `e2e`
pub const KITCHEN_SINK: &str = r#"package demo;

import java.util.ArrayList;
import java.util.List;

public class KitchenSink {
    private final String name;

    public KitchenSink(String name) {
        this.name = name;
    }

    public String describe(int offset) throws java.io.IOException {
        if (name == null) {
            throw new java.io.IOException("no name");
        }
        int total = offset * 2 - 1;
        List<String> parts = new ArrayList<>();
        parts.add(name);
        parts.add(String.valueOf(total));
        return String.join("-", parts);
    }

    public static int compute(int a, int b, int c, int d, int e) {
        int x = (a + b) * (c - d);
        int y = e == 0 ? 1 : e;
        return x + y;
    }
}

record Pair(int left, int right) {
    public Pair {
        if (left > right) {
            throw new IllegalArgumentException("left > right");
        }
    }

    int sum() {
        return left + right;
    }
}

interface Worker {
    void run() throws java.io.IOException;
}

enum Level {
    LOW,
    HIGH;

    boolean isHigh() {
        return this == HIGH;
    }
}
"#;

/// Synthetically generated source: `count` package-private classes, each with a
/// constructor, fields, control flow, `throws`, generics and a chained builder
/// method, followed by one record, one enum and one interface.
pub fn generated_source(count: usize) -> String {
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
