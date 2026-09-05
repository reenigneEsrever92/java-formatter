//! ASSERT_STATEMENT_COLON_ON_NEXT_LINE — `:` placement on wrapped `assert`
//! statements.
//! Fixtures live under tests/java/assert_statement_colon_on_next_line/.
//!
//! The bool only steers the `:` once ASSERT_STATEMENT_WRAP breaks the
//! statement: `false` (the default) keeps the `:` at the end of the
//! expression's line, `true` starts the continuation line with it.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const ASSERTS: &str = include_str!("../java/assert_statement_colon_on_next_line/asserts.java");
const ASSERTS_COLON_NEXT_OUT: &str =
    include_str!("../java/assert_statement_colon_on_next_line/asserts_colon_next.out.java");
const ASSERTS_COLON_END_OUT: &str =
    include_str!("../java/assert_statement_colon_on_next_line/asserts_colon_end.out.java");
const COLON_NEXT_WRAPPED: &str =
    include_str!("../java/assert_statement_colon_on_next_line/colon_next_wrapped.java");
const COLON_NEXT_WRAPPED_OUT: &str =
    include_str!("../java/assert_statement_colon_on_next_line/colon_next_wrapped.out.java");

fn style_with(colon_on_next_line: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.assert_statement_wrap = WrapStyle::WrapIfLong;
        s.assert_statement_colon_on_next_line = colon_on_next_line;
    })
}

#[test]
fn colon_on_next_line_starts_the_continuation_line_with_the_colon() {
    assert_eq!(
        format_with(ASSERTS, &style_with(true)),
        ASSERTS_COLON_NEXT_OUT
    );
}

#[test]
fn colon_off_keeps_the_colon_at_the_end_of_the_expression_line() {
    assert_eq!(
        format_with(ASSERTS, &style_with(false)),
        ASSERTS_COLON_END_OUT
    );
}

#[test]
fn reformatting_colon_next_output_is_a_no_op() {
    // A self-golden: the fixture already uses the own-line `:` layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(COLON_NEXT_WRAPPED, &style_with(true)),
        COLON_NEXT_WRAPPED_OUT
    );
}
