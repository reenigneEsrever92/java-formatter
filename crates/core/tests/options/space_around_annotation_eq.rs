//! SPACE_AROUND_ANNOTATION_EQ — spaces around '=' in annotation arguments.
//! Fixtures live under tests/java/space_around_annotation_eq/.
//!
//! `true` (the default) renders `key = value`; `false` renders `key=value`.
//! Applies wherever an annotation `element_value_pair` is rendered — flat,
//! expanded, and the expanded single-pair-with-array branch.

use super::common::*;

const EQ: &str = include_str!("../java/space_around_annotation_eq/eq.java");
const EQ_TRUE_OUT: &str = include_str!("../java/space_around_annotation_eq/eq_true.out.java");
const EQ_FALSE_OUT: &str = include_str!("../java/space_around_annotation_eq/eq_false.out.java");
const EQ_DEFAULT_OUT: &str = include_str!("../java/space_around_annotation_eq/eq_default.out.java");
const EQ_SELF: &str = include_str!("../java/space_around_annotation_eq/eq_self.java");
const EQ_SELF_OUT: &str = include_str!("../java/space_around_annotation_eq/eq_self.out.java");

#[test]
fn eq_spaced_when_enabled() {
    let s = style(|s| {
        s.right_margin = 40;
        s.annotation_parameter_wrap = java_formatter_core::config::WrapStyle::ChopDownIfLong;
        s.space_around_annotation_eq = true;
    });
    assert_eq!(format_with(EQ, &s), EQ_TRUE_OUT);
}

#[test]
fn eq_tight_when_disabled() {
    let s = style(|s| {
        s.right_margin = 40;
        s.annotation_parameter_wrap = java_formatter_core::config::WrapStyle::ChopDownIfLong;
        s.space_around_annotation_eq = false;
    });
    assert_eq!(format_with(EQ, &s), EQ_FALSE_OUT);
}

#[test]
fn absent_option_defaults_to_spaced_eq() {
    let s = style(|s| {
        s.right_margin = 40;
        s.annotation_parameter_wrap = java_formatter_core::config::WrapStyle::ChopDownIfLong;
    });
    assert_eq!(format_with(EQ, &s), EQ_DEFAULT_OUT);
}

#[test]
fn reformatting_tight_eq_output_is_a_no_op() {
    let s = style(|s| {
        s.right_margin = 60;
        s.space_around_annotation_eq = false;
    });
    assert_eq!(format_with(EQ_SELF, &s), EQ_SELF_OUT);
}
