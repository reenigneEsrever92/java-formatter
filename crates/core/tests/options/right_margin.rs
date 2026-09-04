//! RIGHT_MARGIN — the root-level hard right margin that drives line-length
//! decisions when `SOFT_MARGINS` is absent from the scheme.
//!
//! Fixtures live under tests/java/right_margin/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const LONG_CALL: &str = include_str!("../java/right_margin/long_call.java");
const LONG_CALL_OUT: &str = include_str!("../java/right_margin/long_call.out.java");
const LONG_CALL_120_OUT: &str = include_str!("../java/right_margin/long_call_120.out.java");

#[test]
fn right_margin_40_wraps_the_long_call() {
    let style = style(|s| {
        s.right_margin = 40;
        s.call_parameters_wrap = WrapStyle::WrapIfLong;
    });
    assert_eq!(format_with(LONG_CALL, &style), LONG_CALL_OUT);
}

#[test]
fn default_right_margin_120_keeps_the_line_intact() {
    let style = style(|s| s.call_parameters_wrap = WrapStyle::WrapIfLong);
    assert_eq!(format_with(LONG_CALL, &style), LONG_CALL_120_OUT);
}
