//! ARRAY_INITIALIZER_LBRACE_ON_NEXT_LINE — `{` placement on wrapped array
//! initializers.
//! Fixtures live under tests/java/array_initializer_lbrace_on_next_line/.
//!
//! The bool only steers the `{` once ARRAY_INITIALIZER_WRAP breaks the
//! initializer: `true` puts the `{` on its own line at the statement
//! indent, `false` (the default) keeps it at the end of the preceding line.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_INIT: &str =
    include_str!("../java/array_initializer_lbrace_on_next_line/long_init.java");
const LONG_INIT_LBRACE_ON_OUT: &str =
    include_str!("../java/array_initializer_lbrace_on_next_line/long_init_lbrace_on.out.java");
const LONG_INIT_LBRACE_OFF_OUT: &str =
    include_str!("../java/array_initializer_lbrace_on_next_line/long_init_lbrace_off.out.java");
const LBRACE_WRAPPED: &str =
    include_str!("../java/array_initializer_lbrace_on_next_line/lbrace_wrapped.java");
const LBRACE_WRAPPED_OUT: &str =
    include_str!("../java/array_initializer_lbrace_on_next_line/lbrace_wrapped.out.java");

fn style_with(lbrace_on_next_line: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 50;
        s.array_initializer_wrap = WrapStyle::WrapIfLong;
        s.array_initializer_lbrace_on_next_line = lbrace_on_next_line;
    })
}

#[test]
fn lbrace_on_next_line_puts_the_lbrace_on_its_own_line() {
    assert_eq!(
        format_with(LONG_INIT, &style_with(true)),
        LONG_INIT_LBRACE_ON_OUT
    );
}

#[test]
fn lbrace_off_keeps_the_lbrace_at_the_end_of_the_preceding_line() {
    assert_eq!(
        format_with(LONG_INIT, &style_with(false)),
        LONG_INIT_LBRACE_OFF_OUT
    );
}

#[test]
fn reformatting_lbrace_wrapped_output_is_a_no_op() {
    // A self-golden: the fixture already uses the own-line `{` layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(LBRACE_WRAPPED, &style_with(true)),
        LBRACE_WRAPPED_OUT
    );
}
