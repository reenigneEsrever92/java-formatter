//! DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION — keep a lone annotation on the same
//! line as its field / method / class / local variable declaration.
//! Fixtures live under tests/java/do_not_wrap_after_single_annotation/.
//!
//! When `true`, a declaration carrying exactly one annotation keeps that
//! annotation inline regardless of the placement wrap code; declarations with
//! multiple annotations still break per the wrap code. Exercises field, local
//! variable and nested class sites under a wrap-always style.

use super::common::*;

const SINGLE: &str = include_str!("../java/do_not_wrap_after_single_annotation/single.java");
const SINGLE_FALSE_OUT: &str =
    include_str!("../java/do_not_wrap_after_single_annotation/single_false.out.java");
const SINGLE_TRUE_OUT: &str =
    include_str!("../java/do_not_wrap_after_single_annotation/single_true.out.java");
const SINGLE_DEFAULT_OUT: &str =
    include_str!("../java/do_not_wrap_after_single_annotation/single_default.out.java");
const SINGLE_SELF: &str =
    include_str!("../java/do_not_wrap_after_single_annotation/single_self.java");
const SINGLE_SELF_OUT: &str =
    include_str!("../java/do_not_wrap_after_single_annotation/single_self.out.java");

fn style_with(exempt: bool) -> java_formatter_core::config::JavaStyle {
    style(|s| {
        s.field_annotation_wrap = java_formatter_core::config::WrapStyle::WrapAlways;
        s.variable_annotation_wrap = java_formatter_core::config::WrapStyle::WrapAlways;
        s.do_not_wrap_after_single_annotation = exempt;
    })
}

#[test]
fn off_breaks_single_and_multiple_annotations_alike() {
    assert_eq!(format_with(SINGLE, &style_with(false)), SINGLE_FALSE_OUT);
}

#[test]
fn on_keeps_a_lone_annotation_inline() {
    assert_eq!(format_with(SINGLE, &style_with(true)), SINGLE_TRUE_OUT);
}

#[test]
fn absent_option_defaults_to_off() {
    let s = style(|s| {
        s.field_annotation_wrap = java_formatter_core::config::WrapStyle::WrapAlways;
        s.variable_annotation_wrap = java_formatter_core::config::WrapStyle::WrapAlways;
    });
    assert_eq!(format_with(SINGLE, &s), SINGLE_DEFAULT_OUT);
}

#[test]
fn reformatting_exempted_output_is_a_no_op() {
    assert_eq!(format_with(SINGLE_SELF, &style_with(true)), SINGLE_SELF_OUT);
}
