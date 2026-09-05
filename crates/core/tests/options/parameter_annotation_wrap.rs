//! PARAMETER_ANNOTATION_WRAP — placement of annotations on formal parameters.
//! Fixtures live under tests/java/parameter_annotation_wrap/.
//!
//! `DoNotWrap` (the default) keeps parameter annotations inline. `WrapAlways`
//! takes the parameter list to its wrapped one-parameter-per-line layout and
//! breaks each annotated parameter so the annotations sit on their own lines
//! and the type / name continues on the next line. `WrapIfLong` and
//! `ChopDownIfLong` do the same only when the flat list overflows the margin.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const PARAMETER: &str = include_str!("../java/parameter_annotation_wrap/parameter.java");
const PARAMETER_0_OUT: &str =
    include_str!("../java/parameter_annotation_wrap/parameter_0.out.java");
const PARAMETER_1_OUT: &str =
    include_str!("../java/parameter_annotation_wrap/parameter_1.out.java");
const PARAMETER_2_OUT: &str =
    include_str!("../java/parameter_annotation_wrap/parameter_2.out.java");
const PARAMETER_5_OUT: &str =
    include_str!("../java/parameter_annotation_wrap/parameter_5.out.java");
const PARAMETER_DEFAULT_OUT: &str =
    include_str!("../java/parameter_annotation_wrap/parameter_default.out.java");
const PARAMETER_WRAPPED: &str =
    include_str!("../java/parameter_annotation_wrap/parameter_wrapped.java");
const PARAMETER_WRAPPED_OUT: &str =
    include_str!("../java/parameter_annotation_wrap/parameter_wrapped.out.java");

fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.parameter_annotation_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_keeps_parameter_annotations_inline() {
    assert_eq!(
        format_with(PARAMETER, &narrow(WrapStyle::DoNotWrap)),
        PARAMETER_0_OUT
    );
}

#[test]
fn wrap_if_long_only_wraps_when_the_flat_list_overflows() {
    assert_eq!(
        format_with(PARAMETER, &narrow(WrapStyle::WrapIfLong)),
        PARAMETER_1_OUT
    );
}

#[test]
fn wrap_always_wraps_the_list_and_breaks_each_annotated_parameter() {
    let s = style(|s| s.parameter_annotation_wrap = WrapStyle::WrapAlways);
    assert_eq!(format_with(PARAMETER, &s), PARAMETER_2_OUT);
}

#[test]
fn chop_down_if_long_wraps_like_wrap_if_long_on_overflow() {
    assert_eq!(
        format_with(PARAMETER, &narrow(WrapStyle::ChopDownIfLong)),
        PARAMETER_5_OUT
    );
}

#[test]
fn absent_option_defaults_to_do_not_wrap() {
    assert_eq!(format(PARAMETER), PARAMETER_DEFAULT_OUT);
}

#[test]
fn reformatting_wrapped_parameter_output_is_a_no_op() {
    let s = style(|s| s.parameter_annotation_wrap = WrapStyle::WrapAlways);
    assert_eq!(format_with(PARAMETER_WRAPPED, &s), PARAMETER_WRAPPED_OUT);
}
