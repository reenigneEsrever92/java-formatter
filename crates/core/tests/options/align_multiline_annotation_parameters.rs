//! ALIGN_MULTILINE_ANNOTATION_PARAMETERS — align wrapped annotation arguments
//! under the first argument.
//! Fixtures live under tests/java/align_multiline_annotation_parameters/.
//!
//! `false` (the default) indents wrapped argument lines with the continuation
//! indent; `true` pads them to one column after the annotation's `(`, the
//! record-header model. Only wrapped (expanded) annotation argument lists are
//! affected.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const ALIGN: &str = include_str!("../java/align_multiline_annotation_parameters/align.java");
const ALIGN_FALSE_OUT: &str =
    include_str!("../java/align_multiline_annotation_parameters/align_false.out.java");
const ALIGN_TRUE_OUT: &str =
    include_str!("../java/align_multiline_annotation_parameters/align_true.out.java");
const ALIGN_DEFAULT_OUT: &str =
    include_str!("../java/align_multiline_annotation_parameters/align_default.out.java");
const ALIGN_SELF: &str =
    include_str!("../java/align_multiline_annotation_parameters/align_self.java");
const ALIGN_SELF_OUT: &str =
    include_str!("../java/align_multiline_annotation_parameters/align_self.out.java");

#[test]
fn align_off_uses_the_continuation_indent() {
    let s = style(|s| {
        s.right_margin = 60;
        s.annotation_parameter_wrap = WrapStyle::WrapAlways;
        s.align_multiline_annotation_parameters = false;
    });
    assert_eq!(format_with(ALIGN, &s), ALIGN_FALSE_OUT);
}

#[test]
fn align_on_pads_wrapped_arguments_under_the_first() {
    let s = style(|s| {
        s.right_margin = 60;
        s.annotation_parameter_wrap = WrapStyle::WrapAlways;
        s.align_multiline_annotation_parameters = true;
    });
    assert_eq!(format_with(ALIGN, &s), ALIGN_TRUE_OUT);
}

#[test]
fn absent_option_defaults_to_no_alignment() {
    let s = style(|s| {
        s.right_margin = 60;
        s.annotation_parameter_wrap = WrapStyle::WrapAlways;
    });
    assert_eq!(format_with(ALIGN, &s), ALIGN_DEFAULT_OUT);
}

#[test]
fn reformatting_unaligned_wrapped_output_is_a_no_op() {
    let s = style(|s| {
        s.right_margin = 60;
        s.annotation_parameter_wrap = WrapStyle::WrapAlways;
        s.align_multiline_annotation_parameters = false;
    });
    assert_eq!(format_with(ALIGN_SELF, &s), ALIGN_SELF_OUT);
}
