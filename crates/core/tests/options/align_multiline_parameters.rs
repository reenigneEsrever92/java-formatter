//! ALIGN_MULTILINE_PARAMETERS — align the parameters of a wrapped method /
//! constructor declaration under the first parameter.
//! Fixtures live under tests/java/align_multiline_parameters/.
//!
//! The option defaults to true: when the first parameter stays on the header
//! line after `(` and the rest wrap (the lparen-stays / rparen-alone layout),
//! the wrapped lines pad to the column after `(` instead of the continuation
//! indent. Where every parameter begins its own line the elements already
//! share the first parameter's column, so the option leaves them unchanged.
//! The absent case (default true) is the aligned layout.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const SAMPLE: &str = include_str!("../java/align_multiline_parameters/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_multiline_parameters/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_multiline_parameters/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_multiline_parameters/sample_default.out.java");
const SELF_ALIGNED: &str = include_str!("../java/align_multiline_parameters/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_multiline_parameters/self_aligned.out.java");

fn wrap(align: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.method_parameters_wrap = WrapStyle::ChopDownIfLong;
        s.method_parameters_lparen_on_next_line = false;
        s.method_parameters_rparen_on_next_line = true;
        s.align_multiline_parameters = align;
    })
}

#[test]
fn align_on_aligns_wrapped_parameters_under_the_first_parameter() {
    assert_eq!(format_with(SAMPLE, &wrap(true)), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_wrapped_parameters_at_the_continuation_indent() {
    assert_eq!(format_with(SAMPLE, &wrap(false)), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_aligns_wrapped_parameters() {
    // The option defaults to true: the style sets only the wrap and paren
    // toggles, and the wrapped parameters align under the first parameter.
    let style = style(|s| {
        s.right_margin = 40;
        s.method_parameters_wrap = WrapStyle::ChopDownIfLong;
        s.method_parameters_lparen_on_next_line = false;
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
