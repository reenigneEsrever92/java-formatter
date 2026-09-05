//! ENUM_FIELD_ANNOTATION_WRAP — placement of annotations on enum constants.
//! Fixtures live under tests/java/enum_field_annotation_wrap/.
//!
//! `DoNotWrap` (the default) joins annotations inline with the constant.
//! `WrapAlways` puts each annotation on its own line before the constant;
//! `WrapIfLong` and `ChopDownIfLong` keep the inline form unless the composed
//! first line overflows the margin. The constant's name, arguments and body
//! are always echoed verbatim from the source (R4/R5).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const ENUM_FIELD: &str = include_str!("../java/enum_field_annotation_wrap/enum_field.java");
const ENUM_FIELD_0_OUT: &str =
    include_str!("../java/enum_field_annotation_wrap/enum_field_0.out.java");
const ENUM_FIELD_1_OUT: &str =
    include_str!("../java/enum_field_annotation_wrap/enum_field_1.out.java");
const ENUM_FIELD_2_OUT: &str =
    include_str!("../java/enum_field_annotation_wrap/enum_field_2.out.java");
const ENUM_FIELD_5_OUT: &str =
    include_str!("../java/enum_field_annotation_wrap/enum_field_5.out.java");
const ENUM_FIELD_DEFAULT_OUT: &str =
    include_str!("../java/enum_field_annotation_wrap/enum_field_default.out.java");
const ENUM_FIELD_WRAPPED: &str =
    include_str!("../java/enum_field_annotation_wrap/enum_field_wrapped.java");
const ENUM_FIELD_WRAPPED_OUT: &str =
    include_str!("../java/enum_field_annotation_wrap/enum_field_wrapped.out.java");

fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.enum_field_annotation_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_keeps_enum_constant_annotations_inline() {
    assert_eq!(
        format_with(ENUM_FIELD, &narrow(WrapStyle::DoNotWrap)),
        ENUM_FIELD_0_OUT
    );
}

#[test]
fn wrap_if_long_breaks_only_the_overflowing_constant() {
    assert_eq!(
        format_with(ENUM_FIELD, &narrow(WrapStyle::WrapIfLong)),
        ENUM_FIELD_1_OUT
    );
}

#[test]
fn wrap_always_puts_each_annotation_on_its_own_line() {
    let s = style(|s| s.enum_field_annotation_wrap = WrapStyle::WrapAlways);
    assert_eq!(format_with(ENUM_FIELD, &s), ENUM_FIELD_2_OUT);
}

#[test]
fn chop_down_if_long_breaks_the_long_constant_like_wrap_if_long() {
    assert_eq!(
        format_with(ENUM_FIELD, &narrow(WrapStyle::ChopDownIfLong)),
        ENUM_FIELD_5_OUT
    );
}

#[test]
fn absent_option_defaults_to_do_not_wrap() {
    assert_eq!(format(ENUM_FIELD), ENUM_FIELD_DEFAULT_OUT);
}

#[test]
fn reformatting_wrapped_enum_output_is_a_no_op() {
    let s = style(|s| s.enum_field_annotation_wrap = WrapStyle::WrapAlways);
    assert_eq!(format_with(ENUM_FIELD_WRAPPED, &s), ENUM_FIELD_WRAPPED_OUT);
}
