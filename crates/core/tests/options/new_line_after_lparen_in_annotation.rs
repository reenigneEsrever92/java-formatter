//! NEW_LINE_AFTER_LPAREN_IN_ANNOTATION — put the '(' of a wrapped annotation's
//! argument list on its own line.
//! Fixtures live under tests/java/new_line_after_lparen_in_annotation/.
//!
//! `false` (the default) keeps the first argument on the '(' line and starts
//! later arguments on their own lines; `true` starts every argument on its own
//! line after '('.
//! Only wrapped (expanded) annotation argument lists are affected.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const LPAREN: &str = include_str!("../java/new_line_after_lparen_in_annotation/lparen.java");
const LPAREN_FALSE_OUT: &str =
    include_str!("../java/new_line_after_lparen_in_annotation/lparen_false.out.java");
const LPAREN_TRUE_OUT: &str =
    include_str!("../java/new_line_after_lparen_in_annotation/lparen_true.out.java");
const LPAREN_DEFAULT_OUT: &str =
    include_str!("../java/new_line_after_lparen_in_annotation/lparen_default.out.java");
const LPAREN_SELF: &str =
    include_str!("../java/new_line_after_lparen_in_annotation/lparen_self.java");
const LPAREN_SELF_OUT: &str =
    include_str!("../java/new_line_after_lparen_in_annotation/lparen_self.out.java");

fn style_with(lparen_nl: bool) -> java_formatter_core::config::JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.annotation_parameter_wrap = WrapStyle::WrapAlways;
        s.new_line_after_lparen_in_annotation = lparen_nl;
    })
}

#[test]
fn lparen_stays_attached_when_disabled() {
    assert_eq!(format_with(LPAREN, &style_with(false)), LPAREN_FALSE_OUT);
}

#[test]
fn every_argument_starts_on_its_own_line_when_enabled() {
    assert_eq!(format_with(LPAREN, &style_with(true)), LPAREN_TRUE_OUT);
}

#[test]
fn absent_option_defaults_to_lparen_attached() {
    let s = style(|s| {
        s.right_margin = 60;
        s.annotation_parameter_wrap = WrapStyle::WrapAlways;
    });
    assert_eq!(format_with(LPAREN, &s), LPAREN_DEFAULT_OUT);
}

#[test]
fn reformatting_lparen_on_new_line_output_is_a_no_op() {
    assert_eq!(format_with(LPAREN_SELF, &style_with(true)), LPAREN_SELF_OUT);
}
