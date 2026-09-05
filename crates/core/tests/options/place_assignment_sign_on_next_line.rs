//! PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE — assignment operator placement on
//! wrapped assignments and initialisers.
//! Fixtures live under tests/java/place_assignment_sign_on_next_line/.
//!
//! The bool only steers the operator once ASSIGNMENT_WRAP (or
//! KEEP_LINE_BREAKS) breaks the assignment: `false` (the default) keeps the
//! operator at the end of the header line, `true` starts the continuation
//! line with it.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_INIT: &str = include_str!("../java/place_assignment_sign_on_next_line/long_init.java");
const LONG_INIT_SIGN_NEXT_OUT: &str =
    include_str!("../java/place_assignment_sign_on_next_line/long_init_sign_next.out.java");
const LONG_INIT_SIGN_END_OUT: &str =
    include_str!("../java/place_assignment_sign_on_next_line/long_init_sign_end.out.java");
const SIGN_NEXT_WRAPPED: &str =
    include_str!("../java/place_assignment_sign_on_next_line/sign_next_wrapped.java");
const SIGN_NEXT_WRAPPED_OUT: &str =
    include_str!("../java/place_assignment_sign_on_next_line/sign_next_wrapped.out.java");

fn style_with(sign_next: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.assignment_wrap = WrapStyle::WrapIfLong;
        s.place_assignment_sign_on_next_line = sign_next;
    })
}

#[test]
fn sign_on_next_line_starts_the_continuation_line_with_the_operator() {
    assert_eq!(
        format_with(LONG_INIT, &style_with(true)),
        LONG_INIT_SIGN_NEXT_OUT
    );
}

#[test]
fn sign_off_keeps_the_operator_at_the_end_of_the_header_line() {
    assert_eq!(
        format_with(LONG_INIT, &style_with(false)),
        LONG_INIT_SIGN_END_OUT
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
