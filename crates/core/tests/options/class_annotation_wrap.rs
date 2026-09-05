//! CLASS_ANNOTATION_WRAP — placement of annotations on class / interface /
//! enum / record declarations.
//! Fixtures live under tests/java/class_annotation_wrap/.
//!
//! `DoNotWrap` joins annotations inline with the declaration; `WrapAlways`
//! (the default) puts each annotation on its own line; `WrapIfLong` and
//! `ChopDownIfLong` keep the inline form unless the composed first line
//! overflows the margin (codes 1 and 5 behave identically here).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const CLASS: &str = include_str!("../java/class_annotation_wrap/class.java");
const CLASS_0_OUT: &str = include_str!("../java/class_annotation_wrap/class_0.out.java");
const CLASS_1_OUT: &str = include_str!("../java/class_annotation_wrap/class_1.out.java");
const CLASS_2_OUT: &str = include_str!("../java/class_annotation_wrap/class_2.out.java");
const CLASS_5_OUT: &str = include_str!("../java/class_annotation_wrap/class_5.out.java");
const CLASS_DEFAULT_OUT: &str =
    include_str!("../java/class_annotation_wrap/class_default.out.java");
const CLASS_INLINE: &str = include_str!("../java/class_annotation_wrap/class_inline.java");
const CLASS_INLINE_OUT: &str = include_str!("../java/class_annotation_wrap/class_inline.out.java");

fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.class_annotation_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_joins_all_modifiers_inline() {
    assert_eq!(
        format_with(CLASS, &narrow(WrapStyle::DoNotWrap)),
        CLASS_0_OUT
    );
}

#[test]
fn wrap_if_long_keeps_short_classes_inline_and_breaks_long_ones() {
    assert_eq!(
        format_with(CLASS, &narrow(WrapStyle::WrapIfLong)),
        CLASS_1_OUT
    );
}

#[test]
fn wrap_always_puts_each_annotation_on_its_own_line() {
    let s = style(|s| s.class_annotation_wrap = WrapStyle::WrapAlways);
    assert_eq!(format_with(CLASS, &s), CLASS_2_OUT);
}

#[test]
fn chop_down_if_long_breaks_long_classes_like_wrap_if_long() {
    assert_eq!(
        format_with(CLASS, &narrow(WrapStyle::ChopDownIfLong)),
        CLASS_5_OUT
    );
}

#[test]
fn absent_option_defaults_to_wrap_always() {
    assert_eq!(format(CLASS), CLASS_DEFAULT_OUT);
}

#[test]
fn reformatting_inline_class_output_is_a_no_op() {
    let s = style(|s| s.class_annotation_wrap = WrapStyle::DoNotWrap);
    assert_eq!(format_with(CLASS_INLINE, &s), CLASS_INLINE_OUT);
}
