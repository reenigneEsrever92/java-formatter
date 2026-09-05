//! BINARY_OPERATION_SIGN_ON_NEXT_LINE — operator placement on wrapped
//! binary expressions.
//! Fixtures live under tests/java/binary_operation_sign_on_next_line/.
//!
//! The bool only steers the operator's position once `BINARY_OPERATION_WRAP`
//! breaks the expression: `false` (the default) ends each line with the
//! operator, `true` starts the continuation lines with it. The false-state
//! layout is asserted here directly (the `binary_operation_wrap` goldens
//! cover it too); the true state is pinned by the sign-next goldens.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_SUM: &str = include_str!("../java/binary_operation_sign_on_next_line/long_sum.java");
const LONG_SUM_SIGN_NEXT_OUT: &str =
    include_str!("../java/binary_operation_sign_on_next_line/long_sum_sign_next.out.java");
const LONG_SUM_SIGN_END_OUT: &str =
    include_str!("../java/binary_operation_sign_on_next_line/long_sum_sign_end.out.java");
const SIGN_NEXT_WRAPPED: &str =
    include_str!("../java/binary_operation_sign_on_next_line/sign_next_wrapped.java");
const SIGN_NEXT_WRAPPED_OUT: &str =
    include_str!("../java/binary_operation_sign_on_next_line/sign_next_wrapped.out.java");

fn style_with(sign_next: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.binary_operation_wrap = WrapStyle::WrapIfLong;
        s.binary_operation_sign_on_next_line = sign_next;
    })
}

#[test]
fn sign_on_next_line_starts_continuation_lines_with_the_operator() {
    assert_eq!(
        format_with(LONG_SUM, &style_with(true)),
        LONG_SUM_SIGN_NEXT_OUT
    );
}

#[test]
fn sign_off_keeps_the_operator_at_the_end_of_the_line() {
    assert_eq!(
        format_with(LONG_SUM, &style_with(false)),
        LONG_SUM_SIGN_END_OUT
    );
}

#[test]
fn reformatting_sign_next_output_is_a_no_op() {
    // A self-golden: the fixture already uses the sign-first layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(SIGN_NEXT_WRAPPED, &style_with(true)),
        SIGN_NEXT_WRAPPED_OUT
    );
}
