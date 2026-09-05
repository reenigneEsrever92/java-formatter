//! TERNARY_OPERATION_SIGNS_ON_NEXT_LINE — `?` / `:` placement on wrapped
//! ternary expressions.
//! Fixtures live under tests/java/ternary_operation_signs_on_next_line/.
//!
//! The bool only steers the signs once TERNARY_OPERATION_WRAP breaks the
//! expression: `false` (the default) keeps `?` / `:` at the end of the
//! preceding line (operator-end, like the binary default), `true` starts
//! the continuation lines with them.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_TERNARY: &str =
    include_str!("../java/ternary_operation_signs_on_next_line/long_ternary.java");
const LONG_TERNARY_SIGNS_NEXT_OUT: &str =
    include_str!("../java/ternary_operation_signs_on_next_line/long_ternary_signs_next.out.java");
const LONG_TERNARY_SIGNS_END_OUT: &str =
    include_str!("../java/ternary_operation_signs_on_next_line/long_ternary_signs_end.out.java");
const SIGNS_NEXT_WRAPPED: &str =
    include_str!("../java/ternary_operation_signs_on_next_line/signs_next_wrapped.java");
const SIGNS_NEXT_WRAPPED_OUT: &str =
    include_str!("../java/ternary_operation_signs_on_next_line/signs_next_wrapped.out.java");

fn style_with(signs_next: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 50;
        s.ternary_operation_wrap = WrapStyle::WrapIfLong;
        s.ternary_operation_signs_on_next_line = signs_next;
    })
}

#[test]
fn signs_on_next_line_start_continuation_lines_with_the_signs() {
    assert_eq!(
        format_with(LONG_TERNARY, &style_with(true)),
        LONG_TERNARY_SIGNS_NEXT_OUT
    );
}

#[test]
fn signs_off_keep_the_signs_at_the_end_of_the_line() {
    assert_eq!(
        format_with(LONG_TERNARY, &style_with(false)),
        LONG_TERNARY_SIGNS_END_OUT
    );
}

#[test]
fn reformatting_signs_next_output_is_a_no_op() {
    // A self-golden: the fixture already uses the signs-first layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(SIGNS_NEXT_WRAPPED, &style_with(true)),
        SIGNS_NEXT_WRAPPED_OUT
    );
}
