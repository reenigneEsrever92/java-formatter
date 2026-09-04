//! WRAP_LONG_LINES — hard-wrapping lines longer than the right margin at the
//! last whitespace boundary at or before the margin.
//!
//! Fixtures live under tests/java/wrap_long_lines/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const LONG_SUM: &str = include_str!("../java/wrap_long_lines/long_sum.java");
const LONG_SUM_OUT: &str = include_str!("../java/wrap_long_lines/long_sum.out.java");
const LONG_SUM_FLAT_OUT: &str = include_str!("../java/wrap_long_lines/long_sum_flat.out.java");
const LONG_STRING: &str = include_str!("../java/wrap_long_lines/long_string.java");
const LONG_STRING_OUT: &str = include_str!("../java/wrap_long_lines/long_string.out.java");
const LONG_STRING_FLAT_OUT: &str =
    include_str!("../java/wrap_long_lines/long_string_flat.out.java");

// The hard-wrap pass is a pure function of the flat text: `KEEP_LINE_BREAKS`
// is pinned to false here so that re-formatting the wrapped output reflows it
// to the same flat text first and the wrap points reproduce (idempotent).
fn wrap_style() -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.wrap_long_lines = true;
        s.keep_line_breaks = false;
    })
}

fn flat_style() -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.keep_line_breaks = false;
    })
}

#[test]
fn wrap_long_lines_breaks_an_over_margin_line_at_a_space() {
    assert_eq!(format_with(LONG_SUM, &wrap_style()), LONG_SUM_OUT);
}

#[test]
fn absent_and_false_leave_the_over_margin_line_intact() {
    // The option's default (absent → false) and an explicit false both leave
    // the line alone; only the hard-wrap pass changes it.
    assert_eq!(format_with(LONG_SUM, &flat_style()), LONG_SUM_FLAT_OUT);
    assert_eq!(format(LONG_SUM), LONG_SUM_FLAT_OUT);
}

#[test]
fn wrap_long_lines_never_splits_a_string_literal() {
    // The string literal holds internal spaces but they are not break points,
    // so it is moved whole onto the continuation line and left over-long.
    assert_eq!(format_with(LONG_STRING, &wrap_style()), LONG_STRING_OUT);
}

#[test]
fn absent_and_false_leave_the_long_string_line_intact() {
    assert_eq!(
        format_with(LONG_STRING, &flat_style()),
        LONG_STRING_FLAT_OUT
    );
    assert_eq!(format(LONG_STRING), LONG_STRING_FLAT_OUT);
}
