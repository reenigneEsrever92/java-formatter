//! SOFT_MARGINS — right margin (in columns) beyond which lines get wrapped.
//! Fixtures live under tests/java/right_margin/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const LONG_LINE: &str = include_str!("../java/right_margin/long_line.java");
const LONG_LINE_OUT: &str = include_str!("../java/right_margin/long_line.out.java");
const LONG_LINE_120_OUT: &str = include_str!("../java/right_margin/long_line_120.out.java");

#[test]
fn right_margin_40_wraps_the_long_assignment() {
    let style = style(|s| {
        s.assignment_wrap = WrapStyle::WrapIfLong;
        s.right_margin = 40;
    });
    assert_eq!(format_with(LONG_LINE, &style), LONG_LINE_OUT);
}

#[test]
fn default_right_margin_120_keeps_the_line_intact() {
    let style = style(|s| s.assignment_wrap = WrapStyle::WrapIfLong);
    assert_eq!(format_with(LONG_LINE, &style), LONG_LINE_120_OUT);
}
