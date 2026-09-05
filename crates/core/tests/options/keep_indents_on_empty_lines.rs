//! KEEP_INDENTS_ON_EMPTY_LINES — preserved blank lines inside a block carry
//! the block's inner indent instead of being stripped.
//! Fixtures live under tests/java/keep_indents_on_empty_lines/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const BLANK_LINES: &str = include_str!("../java/keep_indents_on_empty_lines/blank_lines.java");
const BLANK_LINES_OUT: &str =
    include_str!("../java/keep_indents_on_empty_lines/blank_lines.out.java");
const BLANK_LINES_DEFAULT_OUT: &str =
    include_str!("../java/keep_indents_on_empty_lines/blank_lines_default.out.java");

fn style_on() -> JavaStyle {
    style(|s| s.keep_indents_on_empty_lines = true)
}

#[test]
fn keep_indents_on_empty_lines_keeps_the_inner_indent() {
    // The blank lines inside the method / if bodies keep the statement indent
    // (8 / 12 columns); the class-member gap stays a plain blank line.
    assert_eq!(format_with(BLANK_LINES, &style_on()), BLANK_LINES_OUT);
}

#[test]
fn keep_indents_on_empty_lines_idempotent() {
    assert_eq!(format_with(BLANK_LINES_OUT, &style_on()), BLANK_LINES_OUT);
}

#[test]
fn absent_keep_indents_on_empty_lines_strips_blank_line_indents() {
    assert_eq!(format(BLANK_LINES), BLANK_LINES_DEFAULT_OUT);
}
