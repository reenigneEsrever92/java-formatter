//! TERNARY_OPERATION_WRAP — wrapping of ternary (`?:`) expressions.
//! Fixtures live under tests/java/ternary_operation_wrap/.
//!
//! Code `0` (DoNotWrap) keeps the flat form even when long, `1`
//! (WrapIfLong) breaks at `?` / `:` only when the flat form overflows, `2`
//! (WrapAlways) always breaks, and `5` (ChopDownIfLong) additionally
//! recurses into a side that is itself a ternary expression. Sign placement
//! is governed by TERNARY_OPERATION_SIGNS_ON_NEXT_LINE (its own option
//! file); the default operator-end layout is asserted here.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_TERNARY: &str = include_str!("../java/ternary_operation_wrap/long_ternary.java");
const LONG_TERNARY_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/ternary_operation_wrap/long_ternary_do_not_wrap.out.java");
const LONG_TERNARY_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/ternary_operation_wrap/long_ternary_wrap_if_long.out.java");
const LONG_TERNARY_DEFAULT_OUT: &str =
    include_str!("../java/ternary_operation_wrap/long_ternary_default.out.java");
const SHORT_TERNARY: &str = include_str!("../java/ternary_operation_wrap/short_ternary.java");
const SHORT_TERNARY_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/ternary_operation_wrap/short_ternary_wrap_always.out.java");
const NESTED_TERNARY: &str = include_str!("../java/ternary_operation_wrap/nested_ternary.java");
const NESTED_TERNARY_CHOP_DOWN_OUT: &str =
    include_str!("../java/ternary_operation_wrap/nested_ternary_chop_down.out.java");
const NESTED_TERNARY_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/ternary_operation_wrap/nested_ternary_wrap_if_long.out.java");
const SELF_WRAPPED: &str = include_str!("../java/ternary_operation_wrap/self_wrapped.java");
const SELF_WRAPPED_OUT: &str = include_str!("../java/ternary_operation_wrap/self_wrapped.out.java");

/// A narrow margin so the long fixture's ternary overflows.
fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 50;
        s.ternary_operation_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_keeps_the_long_ternary_on_one_line() {
    assert_eq!(
        format_with(LONG_TERNARY, &narrow(WrapStyle::DoNotWrap)),
        LONG_TERNARY_DO_NOT_WRAP_OUT
    );
}

#[test]
fn wrap_if_long_breaks_the_ternary_at_its_signs() {
    assert_eq!(
        format_with(LONG_TERNARY, &narrow(WrapStyle::WrapIfLong)),
        LONG_TERNARY_WRAP_IF_LONG_OUT
    );
}

#[test]
fn wrap_always_breaks_a_ternary_that_would_fit() {
    assert_eq!(
        format_with(SHORT_TERNARY, &narrow(WrapStyle::WrapAlways)),
        SHORT_TERNARY_WRAP_ALWAYS_OUT
    );
}

#[test]
fn chop_down_recurses_into_a_nested_ternary_side() {
    assert_eq!(
        format_with(NESTED_TERNARY, &narrow(WrapStyle::ChopDownIfLong)),
        NESTED_TERNARY_CHOP_DOWN_OUT
    );
}

#[test]
fn wrap_if_long_keeps_a_nested_ternary_side_flat() {
    // WrapIfLong breaks only the outer ternary; the nested consequence stays
    // on one line (chop-down's recursion is the distinguishing behaviour).
    assert_eq!(
        format_with(NESTED_TERNARY, &narrow(WrapStyle::WrapIfLong)),
        NESTED_TERNARY_WRAP_IF_LONG_OUT
    );
}

#[test]
fn default_style_keeps_the_long_ternary_on_one_line() {
    // ternary_operation_wrap defaults to DoNotWrap, so format() (no option
    // set) leaves the over-margin ternary on the header line.
    assert_eq!(format(LONG_TERNARY), LONG_TERNARY_DEFAULT_OUT);
}

#[test]
fn reformatting_wrapped_ternary_output_is_a_no_op() {
    // A self-golden: the fixture already matches the wrapped layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(SELF_WRAPPED, &narrow(WrapStyle::WrapIfLong)),
        SELF_WRAPPED_OUT
    );
}
