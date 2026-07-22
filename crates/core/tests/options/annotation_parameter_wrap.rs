//! ANNOTATION_PARAMETER_WRAP — wrapping of annotation argument lists.
//!
//! `DoNotWrap` (the default) keeps annotation arguments flat even when they
//! overflow the margin. `ChopDownIfLong` expands to one argument per line
//! when the flat form is too long; `WrapAlways` expands regardless.
//!
//! Fixtures live under tests/java/annotation_parameter_wrap/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const ANNOTATION: &str = include_str!("../java/annotation_parameter_wrap/annotation.java");
const ANNOTATION_OUT: &str =
    include_str!("../java/annotation_parameter_wrap/annotation.out.java");
const ANNOTATION_DEFAULT_OUT: &str =
    include_str!("../java/annotation_parameter_wrap/annotation_default.out.java");
const ANNOTATION_CHOP_DOWN_OUT: &str =
    include_str!("../java/annotation_parameter_wrap/annotation_chop_down.out.java");
const ANNOTATION_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/annotation_parameter_wrap/annotation_wrap_always.out.java");

#[test]
fn default_keeps_long_annotation_flat() {
    let style = style(|s| s.right_margin = 40);
    assert_eq!(format_with(ANNOTATION, &style), ANNOTATION_DEFAULT_OUT);
}

#[test]
fn chop_down_if_long_expands_long_annotation_one_argument_per_line() {
    let style = style(|s| {
        s.right_margin = 80;
        s.annotation_parameter_wrap = WrapStyle::ChopDownIfLong;
    });
    assert_eq!(format_with(ANNOTATION, &style), ANNOTATION_OUT);
}

#[test]
fn chop_down_if_long_keeps_short_annotations_flat() {
    let style = style(|s| s.annotation_parameter_wrap = WrapStyle::ChopDownIfLong);
    assert_eq!(format_with(ANNOTATION, &style), ANNOTATION_CHOP_DOWN_OUT);
}

#[test]
fn wrap_always_expands_even_short_annotations() {
    let style = style(|s| s.annotation_parameter_wrap = WrapStyle::WrapAlways);
    assert_eq!(format_with(ANNOTATION, &style), ANNOTATION_WRAP_ALWAYS_OUT);
}
