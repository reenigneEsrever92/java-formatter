//! FIELD_ANNOTATION_WRAP — placement of annotations on field declarations.
//! Fixtures live under tests/java/field_annotation_wrap/.
//!
//! `DoNotWrap` joins annotations inline with the declaration; `WrapAlways`
//! (the default) puts each annotation on its own line; `WrapIfLong` and
//! `ChopDownIfLong` keep the inline form unless the composed first line
//! overflows the margin (codes 1 and 5 behave identically here).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const FIELD: &str = include_str!("../java/field_annotation_wrap/field.java");
const FIELD_0_OUT: &str = include_str!("../java/field_annotation_wrap/field_0.out.java");
const FIELD_1_OUT: &str = include_str!("../java/field_annotation_wrap/field_1.out.java");
const FIELD_2_OUT: &str = include_str!("../java/field_annotation_wrap/field_2.out.java");
const FIELD_5_OUT: &str = include_str!("../java/field_annotation_wrap/field_5.out.java");
const FIELD_DEFAULT_OUT: &str =
    include_str!("../java/field_annotation_wrap/field_default.out.java");
const FIELD_INLINE: &str = include_str!("../java/field_annotation_wrap/field_inline.java");
const FIELD_INLINE_OUT: &str = include_str!("../java/field_annotation_wrap/field_inline.out.java");

fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.field_annotation_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_joins_all_modifiers_inline() {
    assert_eq!(
        format_with(FIELD, &narrow(WrapStyle::DoNotWrap)),
        FIELD_0_OUT
    );
}

#[test]
fn wrap_if_long_keeps_short_fields_inline_and_breaks_long_ones() {
    assert_eq!(
        format_with(FIELD, &narrow(WrapStyle::WrapIfLong)),
        FIELD_1_OUT
    );
}

#[test]
fn wrap_always_puts_each_annotation_on_its_own_line() {
    let s = style(|s| s.field_annotation_wrap = WrapStyle::WrapAlways);
    assert_eq!(format_with(FIELD, &s), FIELD_2_OUT);
}

#[test]
fn chop_down_if_long_breaks_long_fields_like_wrap_if_long() {
    assert_eq!(
        format_with(FIELD, &narrow(WrapStyle::ChopDownIfLong)),
        FIELD_5_OUT
    );
}

#[test]
fn absent_option_defaults_to_wrap_always() {
    assert_eq!(format(FIELD), FIELD_DEFAULT_OUT);
}

#[test]
fn reformatting_inline_field_output_is_a_no_op() {
    let s = style(|s| s.field_annotation_wrap = WrapStyle::DoNotWrap);
    assert_eq!(format_with(FIELD_INLINE, &s), FIELD_INLINE_OUT);
}
