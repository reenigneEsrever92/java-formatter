//! BINARY_OPERATION_WRAP — wrapping behaviour for binary expressions.
//! Fixtures live under tests/java/binary_operation_wrap/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_SUM: &str = include_str!("../java/binary_operation_wrap/long_sum.java");
const LONG_SUM_OUT: &str = include_str!("../java/binary_operation_wrap/long_sum.out.java");
const LONG_SUM_DEFAULT_OUT: &str =
    include_str!("../java/binary_operation_wrap/long_sum_default.out.java");
const DO_NOT_WRAP: &str = include_str!("../java/binary_operation_wrap/do_not_wrap.java");
const DO_NOT_WRAP_OUT: &str = include_str!("../java/binary_operation_wrap/do_not_wrap.out.java");
const ALWAYS_WRAP: &str = include_str!("../java/binary_operation_wrap/always_wrap.java");
const ALWAYS_WRAP_OUT: &str = include_str!("../java/binary_operation_wrap/always_wrap.out.java");
const CHOP_DOWN: &str = include_str!("../java/binary_operation_wrap/chop_down.java");
const CHOP_DOWN_OUT: &str = include_str!("../java/binary_operation_wrap/chop_down.out.java");

/// A style with a tight margin so long expressions overflow.
fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.binary_operation_wrap = wrap;
    })
}

#[test]
fn wrap_if_long_wraps_long_sum_at_operators() {
    assert_eq!(
        format_with(LONG_SUM, &narrow(WrapStyle::WrapIfLong)),
        LONG_SUM_OUT
    );
}

#[test]
fn do_not_wrap_keeps_single_line_even_when_long() {
    assert_eq!(
        format_with(DO_NOT_WRAP, &narrow(WrapStyle::DoNotWrap)),
        DO_NOT_WRAP_OUT
    );
}

#[test]
fn default_style_does_not_wrap_binary_expressions() {
    // binary_operation_wrap defaults to DoNotWrap even under a tight margin.
    let style = style(|s| s.right_margin = 40);
    assert_eq!(format_with(LONG_SUM, &style), LONG_SUM_DEFAULT_OUT);
}

#[test]
fn wrap_always_breaks_short_expression() {
    assert_eq!(
        format_with(ALWAYS_WRAP, &narrow(WrapStyle::WrapAlways)),
        ALWAYS_WRAP_OUT
    );
}

#[test]
fn chop_down_wraps_nested_binary_operands() {
    assert_eq!(
        format_with(CHOP_DOWN, &narrow(WrapStyle::ChopDownIfLong)),
        CHOP_DOWN_OUT
    );
}
