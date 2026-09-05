//! PARENTHESES_EXPRESSION_LPAREN_WRAP — `(` placement when the inner
//! expression of a parenthesized expression wraps.
//! Fixtures live under tests/java/parentheses_expression_lparen_wrap/.
//!
//! When the inner expression renders across several lines, `true` puts the
//! `(` on its own line with the inner expression starting at the
//! continuation indent; `false` (the default) keeps the `(` attached to the
//! first line.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const WRAPPED_PAREN: &str =
    include_str!("../java/parentheses_expression_lparen_wrap/wrapped_paren.java");
const WRAPPED_PAREN_LPAREN_ON_OUT: &str =
    include_str!("../java/parentheses_expression_lparen_wrap/wrapped_paren_lparen_on.out.java");
const WRAPPED_PAREN_LPAREN_OFF_OUT: &str =
    include_str!("../java/parentheses_expression_lparen_wrap/wrapped_paren_lparen_off.out.java");
const LPAREN_WRAPPED: &str =
    include_str!("../java/parentheses_expression_lparen_wrap/lparen_wrapped.java");
const LPAREN_WRAPPED_OUT: &str =
    include_str!("../java/parentheses_expression_lparen_wrap/lparen_wrapped.out.java");

/// A style that wraps the inner binary expression so the paren layout
/// decisions become visible.
fn style_with(lparen_on_next_line: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 50;
        s.binary_operation_wrap = WrapStyle::WrapIfLong;
        s.parentheses_expression_lparen_wrap = lparen_on_next_line;
    })
}

#[test]
fn lparen_on_next_line_puts_the_lparen_on_its_own_line() {
    assert_eq!(
        format_with(WRAPPED_PAREN, &style_with(true)),
        WRAPPED_PAREN_LPAREN_ON_OUT
    );
}

#[test]
fn lparen_off_keeps_the_lparen_on_the_first_line() {
    assert_eq!(
        format_with(WRAPPED_PAREN, &style_with(false)),
        WRAPPED_PAREN_LPAREN_OFF_OUT
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
