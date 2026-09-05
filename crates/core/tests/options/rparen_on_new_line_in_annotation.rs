//! RPAREN_ON_NEW_LINE_IN_ANNOTATION — put the ')' of a wrapped annotation's
//! argument list on its own line.
//! Fixtures live under tests/java/rparen_on_new_line_in_annotation/.
//!
//! `false` (the default) attaches ')' to the last argument's line; `true`
//! places it on its own line at the annotation's indent.
//! Only wrapped (expanded) annotation argument lists are affected.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const RPAREN: &str = include_str!("../java/rparen_on_new_line_in_annotation/rparen.java");
const RPAREN_FALSE_OUT: &str =
    include_str!("../java/rparen_on_new_line_in_annotation/rparen_false.out.java");
const RPAREN_TRUE_OUT: &str =
    include_str!("../java/rparen_on_new_line_in_annotation/rparen_true.out.java");
const RPAREN_DEFAULT_OUT: &str =
    include_str!("../java/rparen_on_new_line_in_annotation/rparen_default.out.java");
const RPAREN_SELF: &str = include_str!("../java/rparen_on_new_line_in_annotation/rparen_self.java");
const RPAREN_SELF_OUT: &str =
    include_str!("../java/rparen_on_new_line_in_annotation/rparen_self.out.java");

fn style_with(rparen_nl: bool) -> java_formatter_core::config::JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.annotation_parameter_wrap = WrapStyle::WrapAlways;
        s.rparen_on_new_line_in_annotation = rparen_nl;
    })
}

#[test]
fn rparen_attaches_to_the_last_argument_when_disabled() {
    assert_eq!(format_with(RPAREN, &style_with(false)), RPAREN_FALSE_OUT);
}

#[test]
fn rparen_goes_on_its_own_line_when_enabled() {
    assert_eq!(format_with(RPAREN, &style_with(true)), RPAREN_TRUE_OUT);
}

#[test]
fn absent_option_defaults_to_rparen_attached() {
    let s = style(|s| {
        s.right_margin = 60;
        s.annotation_parameter_wrap = WrapStyle::WrapAlways;
    });
    assert_eq!(format_with(RPAREN, &s), RPAREN_DEFAULT_OUT);
}

#[test]
fn reformatting_rparen_on_new_line_output_is_a_no_op() {
    assert_eq!(format_with(RPAREN_SELF, &style_with(true)), RPAREN_SELF_OUT);
}
