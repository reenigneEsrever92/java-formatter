//! WRAP_FIRST_METHOD_IN_CALL_CHAIN — whether the first link of a wrapped
//! method-call chain also goes on a continuation line.
//! Fixtures live under tests/java/wrap_first_method_in_call_chain/.
//!
//! When `true`, the chain breaks after its receiver base: the first link
//! starts a continuation line at the continuation indent. `false` (the
//! default) keeps the first link on the header line after the base.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const CHAIN_STMT: &str = include_str!("../java/wrap_first_method_in_call_chain/chain_stmt.java");
const CHAIN_STMT_FIRST_ON_OUT: &str =
    include_str!("../java/wrap_first_method_in_call_chain/chain_stmt_first_on.out.java");
const CHAIN_STMT_FIRST_OFF_OUT: &str =
    include_str!("../java/wrap_first_method_in_call_chain/chain_stmt_first_off.out.java");
const FIRST_WRAPPED: &str =
    include_str!("../java/wrap_first_method_in_call_chain/first_wrapped.java");
const FIRST_WRAPPED_OUT: &str =
    include_str!("../java/wrap_first_method_in_call_chain/first_wrapped.out.java");

fn style_with(first_on_next_line: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.method_call_chain_wrap = WrapStyle::WrapIfLong;
        s.wrap_first_method_in_call_chain = first_on_next_line;
    })
}

#[test]
fn first_method_on_next_line_breaks_the_chain_after_the_base() {
    assert_eq!(
        format_with(CHAIN_STMT, &style_with(true)),
        CHAIN_STMT_FIRST_ON_OUT
    );
}

#[test]
fn first_method_off_keeps_the_first_link_on_the_header_line() {
    assert_eq!(
        format_with(CHAIN_STMT, &style_with(false)),
        CHAIN_STMT_FIRST_OFF_OUT
    );
}

#[test]
fn reformatting_first_wrapped_output_is_a_no_op() {
    // A self-golden: the fixture already uses the own-line first link, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(FIRST_WRAPPED, &style_with(true)),
        FIRST_WRAPPED_OUT
    );
}
