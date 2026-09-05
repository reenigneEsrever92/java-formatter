//! ALIGN_MULTILINE_BINARY_OPERATION — align wrapped binary operands under the
//! first operand.
//! Fixtures live under tests/java/align_multiline_binary_operation/.
//!
//! When a binary expression wraps, each continuation operand line starts at
//! the first operand's column instead of the continuation indent (the
//! operator-end layout keeps the operator on the previous line).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const SAMPLE: &str = include_str!("../java/align_multiline_binary_operation/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_multiline_binary_operation/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_multiline_binary_operation/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_multiline_binary_operation/sample_default.out.java");
const SELF_ALIGNED: &str =
    include_str!("../java/align_multiline_binary_operation/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_multiline_binary_operation/self_aligned.out.java");

fn wrap(align: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.binary_operation_wrap = WrapStyle::ChopDownIfLong;
        s.align_multiline_binary_operation = align;
    })
}

#[test]
fn align_on_aligns_continuation_operands_under_the_first_operand() {
    assert_eq!(format_with(SAMPLE, &wrap(true)), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_continuation_operands_at_the_continuation_indent() {
    assert_eq!(format_with(SAMPLE, &wrap(false)), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_keeps_continuation_operands_at_the_continuation_indent() {
    // The option defaults to false; the style sets only the wrap toggle.
    let style = style(|s| {
        s.right_margin = 60;
        s.binary_operation_wrap = WrapStyle::ChopDownIfLong;
    });
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_the_aligned_layout_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    assert_eq!(format_with(SELF_ALIGNED, &wrap(true)), SELF_ALIGNED_OUT);
}
