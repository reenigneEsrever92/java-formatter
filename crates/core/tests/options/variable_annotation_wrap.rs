//! VARIABLE_ANNOTATION_WRAP — placement of annotations on local variable
//! declarations.
//! Fixtures live under tests/java/variable_annotation_wrap/.
//!
//! `DoNotWrap` (the default) keeps annotations inline with the statement;
//! `WrapAlways` puts each annotation on its own line before the type / name at
//! the statement indent; `WrapIfLong` and `ChopDownIfLong` keep the inline
//! form unless the composed first line overflows the margin (codes 1 and 5
//! behave identically here).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const VARIABLE: &str = include_str!("../java/variable_annotation_wrap/variable.java");
const VARIABLE_0_OUT: &str = include_str!("../java/variable_annotation_wrap/variable_0.out.java");
const VARIABLE_1_OUT: &str = include_str!("../java/variable_annotation_wrap/variable_1.out.java");
const VARIABLE_2_OUT: &str = include_str!("../java/variable_annotation_wrap/variable_2.out.java");
const VARIABLE_5_OUT: &str = include_str!("../java/variable_annotation_wrap/variable_5.out.java");
const VARIABLE_DEFAULT_OUT: &str =
    include_str!("../java/variable_annotation_wrap/variable_default.out.java");
const VARIABLE_INLINE: &str = include_str!("../java/variable_annotation_wrap/variable_inline.java");
const VARIABLE_INLINE_OUT: &str =
    include_str!("../java/variable_annotation_wrap/variable_inline.out.java");

fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.variable_annotation_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_keeps_local_variable_annotations_inline() {
    assert_eq!(
        format_with(VARIABLE, &narrow(WrapStyle::DoNotWrap)),
        VARIABLE_0_OUT
    );
}

#[test]
fn wrap_if_long_keeps_short_locals_inline_and_breaks_long_ones() {
    assert_eq!(
        format_with(VARIABLE, &narrow(WrapStyle::WrapIfLong)),
        VARIABLE_1_OUT
    );
}

#[test]
fn wrap_always_puts_each_annotation_on_its_own_line() {
    let s = style(|s| s.variable_annotation_wrap = WrapStyle::WrapAlways);
    assert_eq!(format_with(VARIABLE, &s), VARIABLE_2_OUT);
}

#[test]
fn chop_down_if_long_breaks_long_locals_like_wrap_if_long() {
    assert_eq!(
        format_with(VARIABLE, &narrow(WrapStyle::ChopDownIfLong)),
        VARIABLE_5_OUT
    );
}

#[test]
fn absent_option_defaults_to_do_not_wrap() {
    assert_eq!(format(VARIABLE), VARIABLE_DEFAULT_OUT);
}

#[test]
fn reformatting_inline_local_output_is_a_no_op() {
    let s = style(|s| s.variable_annotation_wrap = WrapStyle::DoNotWrap);
    assert_eq!(format_with(VARIABLE_INLINE, &s), VARIABLE_INLINE_OUT);
}
