//! FOR_STATEMENT_RPAREN_ON_NEXT_LINE — `)` placement on wrapped `for`
//! headers.
//! Fixtures live under tests/java/for_statement_rparen_on_next_line/.
//!
//! The bool only steers the `)` once FOR_STATEMENT_WRAP breaks the header:
//! `true` puts the `)` on its own line at the statement indent,
//! `false` (the default) keeps it attached to the last header line.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_FOR: &str = include_str!("../java/for_statement_rparen_on_next_line/long_for.java");
const LONG_FOR_RPAREN_ON_OUT: &str =
    include_str!("../java/for_statement_rparen_on_next_line/long_for_rparen_on.out.java");
const LONG_FOR_RPAREN_OFF_OUT: &str =
    include_str!("../java/for_statement_rparen_on_next_line/long_for_rparen_off.out.java");
const RPAREN_WRAPPED: &str =
    include_str!("../java/for_statement_rparen_on_next_line/rparen_wrapped.java");
const RPAREN_WRAPPED_OUT: &str =
    include_str!("../java/for_statement_rparen_on_next_line/rparen_wrapped.out.java");

fn style_with(rparen_on_next_line: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.for_statement_wrap = WrapStyle::WrapIfLong;
        s.for_statement_rparen_on_next_line = rparen_on_next_line;
    })
}

#[test]
fn rparen_on_next_line_puts_the_rparen_on_its_own_line() {
    assert_eq!(
        format_with(LONG_FOR, &style_with(true)),
        LONG_FOR_RPAREN_ON_OUT
    );
}

#[test]
fn rparen_off_keeps_the_rparen_on_the_last_header_line() {
    assert_eq!(
        format_with(LONG_FOR, &style_with(false)),
        LONG_FOR_RPAREN_OFF_OUT
    );
}

#[test]
fn reformatting_rparen_wrapped_output_is_a_no_op() {
    // A self-golden: the fixture already uses the own-line `)` layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(RPAREN_WRAPPED, &style_with(true)),
        RPAREN_WRAPPED_OUT
    );
}
