//! METHOD_BRACE_STYLE — brace placement for method / constructor bodies.
//! Fixtures live under tests/java/method_brace_style/.

use super::common::*;
use java_formatter_core::config::BraceStyle;

const METHOD_BRACE: &str = include_str!("../java/method_brace_style/method_brace.java");
const METHOD_BRACE_OUT: &str =
    include_str!("../java/method_brace_style/method_brace.out.java");
const METHOD_BRACE_DEFAULT_OUT: &str =
    include_str!("../java/method_brace_style/method_brace_default.out.java");

#[test]
fn next_line_puts_the_body_brace_on_the_next_line() {
    let style = style(|s| s.method_brace_style = BraceStyle::NextLine);
    assert_eq!(format_with(METHOD_BRACE, &style), METHOD_BRACE_OUT);
}

#[test]
fn default_end_of_line_keeps_the_brace_on_the_header_line() {
    assert_eq!(format(METHOD_BRACE), METHOD_BRACE_DEFAULT_OUT);
}
