//! SWITCH_EXPRESSIONS_WRAP — wrapping of a `switch` expression used as a
//! value (assignment RHS, return value, argument). Defaults to wrap if long
//! (`1`): a switch that fits the margin stays on one line, a long one falls
//! back to the multi-line switch layout. `0` (do not wrap) keeps the
//! one-line form whenever one exists, `2` (wrap always) always uses the
//! multi-line layout, and `5` (chop down if long) wraps when long and
//! additionally breaks an overflowing nested switch expression in the body.
//! Statement-position switches are unaffected. Whitespace/layout only (R5);
//! formatting formatted output is a no-op (R6).
//!
//! Fixtures live under tests/java/switch_expressions_wrap/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const VALUE_SHORT: &str = include_str!("../java/switch_expressions_wrap/value_short.java");
const VALUE_SHORT_DEFAULT_OUT: &str =
    include_str!("../java/switch_expressions_wrap/value_short_default.out.java");
const VALUE_SHORT_ALWAYS_OUT: &str =
    include_str!("../java/switch_expressions_wrap/value_short_always.out.java");
const VALUE_LONG: &str = include_str!("../java/switch_expressions_wrap/value_long.java");
const VALUE_LONG_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/switch_expressions_wrap/value_long_do_not_wrap.out.java");
const VALUE_LONG_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/switch_expressions_wrap/value_long_wrap_if_long.out.java");
const VALUE_NESTED: &str = include_str!("../java/switch_expressions_wrap/value_nested.java");
const VALUE_NESTED_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/switch_expressions_wrap/value_nested_wrap_if_long.out.java");
const VALUE_NESTED_CHOP_DOWN_OUT: &str =
    include_str!("../java/switch_expressions_wrap/value_nested_chop_down.out.java");

/// A style with a tight margin so a long switch expression overflows.
fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.switch_expressions_wrap = wrap;
    })
}

#[test]
fn default_keeps_a_switch_expression_on_one_line_when_it_fits() {
    assert_eq!(format(VALUE_SHORT), VALUE_SHORT_DEFAULT_OUT);
    assert_eq!(format(VALUE_SHORT_DEFAULT_OUT), VALUE_SHORT_DEFAULT_OUT);
}

#[test]
fn wrap_always_breaks_a_switch_expression_that_would_fit() {
    let s = style(|st| st.switch_expressions_wrap = WrapStyle::WrapAlways);
    assert_eq!(format_with(VALUE_SHORT, &s), VALUE_SHORT_ALWAYS_OUT);
    assert_eq!(
        format_with(VALUE_SHORT_ALWAYS_OUT, &s),
        VALUE_SHORT_ALWAYS_OUT
    );
}

#[test]
fn do_not_wrap_keeps_a_long_switch_expression_on_one_line() {
    let s = narrow(WrapStyle::DoNotWrap);
    assert_eq!(format_with(VALUE_LONG, &s), VALUE_LONG_DO_NOT_WRAP_OUT);
    assert_eq!(
        format_with(VALUE_LONG_DO_NOT_WRAP_OUT, &s),
        VALUE_LONG_DO_NOT_WRAP_OUT
    );
}

#[test]
fn wrap_if_long_breaks_a_long_switch_expression_onto_lines() {
    let s = narrow(WrapStyle::WrapIfLong);
    assert_eq!(format_with(VALUE_LONG, &s), VALUE_LONG_WRAP_IF_LONG_OUT);
    assert_eq!(
        format_with(VALUE_LONG_WRAP_IF_LONG_OUT, &s),
        VALUE_LONG_WRAP_IF_LONG_OUT
    );
}

#[test]
fn wrap_if_long_keeps_an_overflowing_nested_switch_body_on_one_line() {
    // The nested switch is not wrapped on its own (wrap code 1 only wraps the
    // outer construct); the one-line body is echoed.
    let s = narrow(WrapStyle::WrapIfLong);
    assert_eq!(format_with(VALUE_NESTED, &s), VALUE_NESTED_WRAP_IF_LONG_OUT);
    assert_eq!(
        format_with(VALUE_NESTED_WRAP_IF_LONG_OUT, &s),
        VALUE_NESTED_WRAP_IF_LONG_OUT
    );
}

#[test]
fn chop_down_breaks_an_overflowing_nested_switch_body() {
    // Wrap code 5 additionally wraps the nested switch expression whose own
    // line overflows the margin.
    let s = narrow(WrapStyle::ChopDownIfLong);
    assert_eq!(format_with(VALUE_NESTED, &s), VALUE_NESTED_CHOP_DOWN_OUT);
    assert_eq!(
        format_with(VALUE_NESTED_CHOP_DOWN_OUT, &s),
        VALUE_NESTED_CHOP_DOWN_OUT
    );
}
