//! ALIGN_MULTILINE_METHOD_BRACKETS — align the closing paren of a wrapped
//! method / constructor declaration under its opening paren.
//! Fixtures live under tests/java/align_multiline_method_brackets/.
//!
//! When a wrapped declaration's `)` sits on its own line it starts at the
//! opening paren's column instead of the declaration indent. (The shape is
//! pinned by the goldens — IntelliJ's exact behaviour for this option is
//! ambiguous and the record model is followed.)

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const SAMPLE: &str = include_str!("../java/align_multiline_method_brackets/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_multiline_method_brackets/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_multiline_method_brackets/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_multiline_method_brackets/sample_default.out.java");
const SELF_ALIGNED: &str =
    include_str!("../java/align_multiline_method_brackets/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_multiline_method_brackets/self_aligned.out.java");

fn wrap(align: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.method_parameters_wrap = WrapStyle::ChopDownIfLong;
        s.method_parameters_lparen_on_next_line = true;
        s.method_parameters_rparen_on_next_line = true;
        s.align_multiline_method_brackets = align;
    })
}

#[test]
fn align_on_places_the_rparen_under_the_lparen() {
    assert_eq!(format_with(SAMPLE, &wrap(true)), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_places_the_rparen_at_the_declaration_indent() {
    assert_eq!(format_with(SAMPLE, &wrap(false)), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_places_the_rparen_at_the_declaration_indent() {
    // The option defaults to false; the style sets only the wrap and paren
    // toggles.
    let style = style(|s| {
        s.right_margin = 40;
        s.method_parameters_wrap = WrapStyle::ChopDownIfLong;
        s.method_parameters_lparen_on_next_line = true;
        s.method_parameters_rparen_on_next_line = true;
    });
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_the_aligned_layout_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    assert_eq!(format_with(SELF_ALIGNED, &wrap(true)), SELF_ALIGNED_OUT);
}
