//! SPACE_INSIDE_ONE_LINE_ENUM_BRACES — padding inside a one-line enum body.
//! Fixtures live under tests/java/space_inside_one_line_enum_braces/.
//!
//! The flat one-line enum body renders flush (`enum E {A, B}`) by default and
//! padded (`enum E { A, B }`) when the option is on; the padding never leaks
//! into the multi-line (wrapped / member-bearing) layout.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const PADDING: &str = include_str!("../java/space_inside_one_line_enum_braces/padding.java");
const PADDING_SPACES_OUT: &str =
    include_str!("../java/space_inside_one_line_enum_braces/padding_spaces.out.java");
const PADDING_PLAIN_OUT: &str =
    include_str!("../java/space_inside_one_line_enum_braces/padding_plain.out.java");
const LONG_ENUM: &str = include_str!("../java/space_inside_one_line_enum_braces/long_enum.java");
const LONG_ENUM_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/space_inside_one_line_enum_braces/long_enum_wrap_always.out.java");

#[test]
fn on_pads_a_one_line_enum_body() {
    let s = style(|s| s.space_inside_one_line_enum_braces = true);
    assert_eq!(format_with(PADDING, &s), PADDING_SPACES_OUT);
}

#[test]
fn absent_option_renders_the_one_line_body_flush() {
    assert_eq!(format(PADDING), PADDING_PLAIN_OUT);
}

#[test]
fn off_renders_the_one_line_body_flush() {
    let s = style(|s| s.space_inside_one_line_enum_braces = false);
    assert_eq!(format_with(PADDING, &s), PADDING_PLAIN_OUT);
}

#[test]
fn padding_does_not_leak_into_the_wrapped_layout() {
    let s = style(|s| {
        s.right_margin = 40;
        s.enum_constants_wrap = WrapStyle::WrapAlways;
        s.space_inside_one_line_enum_braces = true;
    });
    assert_eq!(format_with(LONG_ENUM, &s), LONG_ENUM_WRAP_ALWAYS_OUT);
}
