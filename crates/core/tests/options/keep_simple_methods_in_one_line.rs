//! KEEP_SIMPLE_METHODS_IN_ONE_LINE — keep single-statement method / constructor
//! bodies on one line.
//! Fixtures live under tests/java/keep_simple_methods_in_one_line/.

use super::common::*;
use java_formatter_core::config::{BraceStyle, JavaStyle};

const SIMPLE_MULTILINE_DEFAULT: &str =
    include_str!("../java/keep_simple_methods_in_one_line/simple_multiline_default.java");
const SIMPLE_MULTILINE_DEFAULT_OUT: &str =
    include_str!("../java/keep_simple_methods_in_one_line/simple_multiline_default.out.java");
const KEEP_SIMPLE_METHODS: &str =
    include_str!("../java/keep_simple_methods_in_one_line/keep_simple_methods.java");
const KEEP_SIMPLE_METHODS_OUT: &str =
    include_str!("../java/keep_simple_methods_in_one_line/keep_simple_methods.out.java");
const NEXT_LINE_BRACE: &str =
    include_str!("../java/keep_simple_methods_in_one_line/next_line_brace.java");
const NEXT_LINE_BRACE_OUT: &str =
    include_str!("../java/keep_simple_methods_in_one_line/next_line_brace.out.java");
const MULTI_STATEMENT_BODY: &str =
    include_str!("../java/keep_simple_methods_in_one_line/multi_statement_body.java");
const MULTI_STATEMENT_BODY_OUT: &str =
    include_str!("../java/keep_simple_methods_in_one_line/multi_statement_body.out.java");

fn keep_simple() -> JavaStyle {
    // The collapse is exercised with the Java padding toggle on so the
    // goldens keep the padded `{ s }` one-line bodies; the faithful
    // absent/false default is flush `{s}`.
    style(|s| {
        s.keep_simple_methods_in_one_line = true;
        s.spaces_inside_block_braces_when_body_is_present = true;
    })
}

fn keep_simple_next_line() -> JavaStyle {
    style(|s| {
        s.keep_simple_methods_in_one_line = true;
        s.method_brace_style = BraceStyle::NextLine;
    })
}

#[test]
fn simple_multiline_default() {
    // Without the option method bodies stay multi-line.
    assert_eq!(
        format(SIMPLE_MULTILINE_DEFAULT),
        SIMPLE_MULTILINE_DEFAULT_OUT
    );
}

#[test]
fn keep_simple_methods() {
    assert_eq!(
        format_with(KEEP_SIMPLE_METHODS, &keep_simple()),
        KEEP_SIMPLE_METHODS_OUT
    );
}

#[test]
fn next_line_brace() {
    // A body cannot be kept on the same line when the brace goes on the next line.
    assert_eq!(
        format_with(NEXT_LINE_BRACE, &keep_simple_next_line()),
        NEXT_LINE_BRACE_OUT
    );
}

#[test]
fn multi_statement_body() {
    // Multi-statement bodies are never collapsed.
    assert_eq!(
        format_with(MULTI_STATEMENT_BODY, &keep_simple()),
        MULTI_STATEMENT_BODY_OUT
    );
}
