//! FOR_STATEMENT_LPAREN_ON_NEXT_LINE — `(` placement on wrapped `for`
//! headers.
//! Fixtures live under tests/java/for_statement_lparen_on_next_line/.
//!
//! The bool only steers the `(` once FOR_STATEMENT_WRAP breaks the header:
//! `true` puts the `(` on its own line at the continuation indent,
//! `false` (the default) keeps it on the `for` line.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_FOR: &str = include_str!("../java/for_statement_lparen_on_next_line/long_for.java");
const LONG_FOR_LPAREN_ON_OUT: &str =
    include_str!("../java/for_statement_lparen_on_next_line/long_for_lparen_on.out.java");
const LONG_FOR_LPAREN_OFF_OUT: &str =
    include_str!("../java/for_statement_lparen_on_next_line/long_for_lparen_off.out.java");
const LPAREN_WRAPPED: &str =
    include_str!("../java/for_statement_lparen_on_next_line/lparen_wrapped.java");
const LPAREN_WRAPPED_OUT: &str =
    include_str!("../java/for_statement_lparen_on_next_line/lparen_wrapped.out.java");

fn style_with(lparen_on_next_line: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.for_statement_wrap = WrapStyle::WrapIfLong;
        s.for_statement_lparen_on_next_line = lparen_on_next_line;
    })
}

#[test]
fn lparen_on_next_line_puts_the_lparen_on_its_own_line() {
    assert_eq!(
        format_with(LONG_FOR, &style_with(true)),
        LONG_FOR_LPAREN_ON_OUT
    );
}

#[test]
fn lparen_off_keeps_the_lparen_on_the_for_line() {
    assert_eq!(
        format_with(LONG_FOR, &style_with(false)),
        LONG_FOR_LPAREN_OFF_OUT
    );
}

#[test]
fn reformatting_lparen_wrapped_output_is_a_no_op() {
    // A self-golden: the fixture already uses the own-line `(` layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(LPAREN_WRAPPED, &style_with(true)),
        LPAREN_WRAPPED_OUT
    );
}
