//! WRAP_SEMICOLON_AFTER_CALL_CHAIN — `;` placement after a wrapped chained
//! method-call statement.
//! Fixtures live under tests/java/wrap_semicolon_after_call_chain/.
//!
//! When the statement's method-call chain wraps and the option is `true`,
//! the terminating `;` moves to its own line at the statement indent;
//! `false` (the default) keeps it attached to the last link.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const CHAIN_STMT: &str = include_str!("../java/wrap_semicolon_after_call_chain/chain_stmt.java");
const CHAIN_STMT_SEMICOLON_ON_OUT: &str =
    include_str!("../java/wrap_semicolon_after_call_chain/chain_stmt_semicolon_on.out.java");
const CHAIN_STMT_SEMICOLON_OFF_OUT: &str =
    include_str!("../java/wrap_semicolon_after_call_chain/chain_stmt_semicolon_off.out.java");
const SEMICOLON_WRAPPED: &str =
    include_str!("../java/wrap_semicolon_after_call_chain/semicolon_wrapped.java");
const SEMICOLON_WRAPPED_OUT: &str =
    include_str!("../java/wrap_semicolon_after_call_chain/semicolon_wrapped.out.java");

fn style_with(semicolon_on_next_line: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.method_call_chain_wrap = WrapStyle::WrapIfLong;
        s.wrap_semicolon_after_call_chain = semicolon_on_next_line;
    })
}

#[test]
fn semicolon_on_next_line_puts_the_semicolon_on_its_own_line() {
    assert_eq!(
        format_with(CHAIN_STMT, &style_with(true)),
        CHAIN_STMT_SEMICOLON_ON_OUT
    );
}

#[test]
fn semicolon_off_keeps_the_semicolon_attached_to_the_last_link() {
    assert_eq!(
        format_with(CHAIN_STMT, &style_with(false)),
        CHAIN_STMT_SEMICOLON_OFF_OUT
    );
}

#[test]
fn reformatting_semicolon_wrapped_output_is_a_no_op() {
    // A self-golden: the fixture already uses the own-line `;` layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(SEMICOLON_WRAPPED, &style_with(true)),
        SEMICOLON_WRAPPED_OUT
    );
}
