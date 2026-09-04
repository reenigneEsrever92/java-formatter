//! SPACE_AROUND_ADDITIVE_OPERATORS — space around additive operators (+, -).
//! Fixtures live under tests/java/space_around_additive_operators/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const MIXED: &str = include_str!("../java/space_around_additive_operators/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_around_additive_operators/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_around_additive_operators/mixed_default.out.java");
const WRAPPED_SUM: &str = include_str!("../java/space_around_additive_operators/wrapped_sum.java");
const WRAPPED_SUM_OUT: &str =
    include_str!("../java/space_around_additive_operators/wrapped_sum.out.java");
const WRAPPED_SUM_DEFAULT_OUT: &str =
    include_str!("../java/space_around_additive_operators/wrapped_sum_default.out.java");

#[test]
fn off_tightens_additive_operators() {
    let style = style(|s| s.space_around_additive_operators = false);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}

#[test]
fn wrapped_sum_glues_operand_to_operator_when_off() {
    let style = style(|s| {
        s.right_margin = 40;
        s.binary_operation_wrap = WrapStyle::WrapIfLong;
        s.space_around_additive_operators = false;
    });
    assert_eq!(format_with(WRAPPED_SUM, &style), WRAPPED_SUM_OUT);
}

#[test]
fn wrapped_sum_keeps_space_after_operator_by_default() {
    let style = style(|s| {
        s.right_margin = 40;
        s.binary_operation_wrap = WrapStyle::WrapIfLong;
    });
    assert_eq!(format_with(WRAPPED_SUM, &style), WRAPPED_SUM_DEFAULT_OUT);
}
