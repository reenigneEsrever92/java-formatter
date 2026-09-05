//! METHOD_ANNOTATION_WRAP — placement of annotations on method declarations.
//! Fixtures live under tests/java/method_annotation_wrap/.
//!
//! `DoNotWrap` joins annotations inline with the declaration; `WrapAlways`
//! (the default) puts each annotation on its own line; `WrapIfLong` and
//! `ChopDownIfLong` keep the inline form unless the composed first line
//! overflows the margin (codes 1 and 5 behave identically here).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const METHOD: &str = include_str!("../java/method_annotation_wrap/method.java");
const METHOD_0_OUT: &str = include_str!("../java/method_annotation_wrap/method_0.out.java");
const METHOD_1_OUT: &str = include_str!("../java/method_annotation_wrap/method_1.out.java");
const METHOD_2_OUT: &str = include_str!("../java/method_annotation_wrap/method_2.out.java");
const METHOD_5_OUT: &str = include_str!("../java/method_annotation_wrap/method_5.out.java");
const METHOD_DEFAULT_OUT: &str =
    include_str!("../java/method_annotation_wrap/method_default.out.java");
const METHOD_INLINE: &str = include_str!("../java/method_annotation_wrap/method_inline.java");
const METHOD_INLINE_OUT: &str =
    include_str!("../java/method_annotation_wrap/method_inline.out.java");

fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.method_annotation_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_joins_all_modifiers_inline() {
    assert_eq!(
        format_with(METHOD, &narrow(WrapStyle::DoNotWrap)),
        METHOD_0_OUT
    );
}

#[test]
fn wrap_if_long_keeps_short_methods_inline_and_breaks_long_ones() {
    assert_eq!(
        format_with(METHOD, &narrow(WrapStyle::WrapIfLong)),
        METHOD_1_OUT
    );
}

#[test]
fn wrap_always_puts_each_annotation_on_its_own_line() {
    let s = style(|s| s.method_annotation_wrap = WrapStyle::WrapAlways);
    assert_eq!(format_with(METHOD, &s), METHOD_2_OUT);
}

#[test]
fn chop_down_if_long_breaks_long_methods_like_wrap_if_long() {
    assert_eq!(
        format_with(METHOD, &narrow(WrapStyle::ChopDownIfLong)),
        METHOD_5_OUT
    );
}

#[test]
fn absent_option_defaults_to_wrap_always() {
    assert_eq!(format(METHOD), METHOD_DEFAULT_OUT);
}

#[test]
fn reformatting_inline_method_output_is_a_no_op() {
    let s = style(|s| s.method_annotation_wrap = WrapStyle::DoNotWrap);
    assert_eq!(format_with(METHOD_INLINE, &s), METHOD_INLINE_OUT);
}
