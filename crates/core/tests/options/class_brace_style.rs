//! CLASS_BRACE_STYLE — brace placement for class / interface / enum / record
//! bodies.
//! Fixtures live under tests/java/class_brace_style/.

use super::common::*;
use java_formatter_core::config::BraceStyle;

const CLASS_BRACE_STYLE: &str = include_str!("../java/class_brace_style/class_brace_style.java");
const CLASS_BRACE_STYLE_OUT: &str =
    include_str!("../java/class_brace_style/class_brace_style.out.java");
const CLASS_BRACE: &str = include_str!("../java/class_brace_style/class_brace.java");
const CLASS_BRACE_OUT: &str = include_str!("../java/class_brace_style/class_brace.out.java");
const CLASS_BRACE_DEFAULT_OUT: &str =
    include_str!("../java/class_brace_style/class_brace_default.out.java");

#[test]
fn record_class_brace_style_is_honoured() {
    let style = style(|s| s.class_brace_style = BraceStyle::NextLine);
    assert_eq!(
        format_with(CLASS_BRACE_STYLE, &style),
        CLASS_BRACE_STYLE_OUT
    );
}

#[test]
fn next_line_style_puts_class_braces_on_next_line() {
    let style = style(|s| s.class_brace_style = BraceStyle::NextLine);
    assert_eq!(format_with(CLASS_BRACE, &style), CLASS_BRACE_OUT);
}

#[test]
fn default_style_keeps_class_braces_on_same_line() {
    assert_eq!(format(CLASS_BRACE), CLASS_BRACE_DEFAULT_OUT);
}
