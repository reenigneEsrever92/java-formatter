//! ALIGN_MULTILINE_PARENTHESIZED_EXPRESSION — align a wrapped parenthesized
//! expression's continuation under the `(`.
//! Fixtures live under tests/java/align_multiline_parenthesized_expression/.
//!
//! When the content of a parenthesized expression wraps, its continuation
//! lines start at the column right after `(` (under the first token) instead
//! of the continuation indent — unless the nested expression's own align
//! option governs them.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const SAMPLE: &str = include_str!("../java/align_multiline_parenthesized_expression/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_multiline_parenthesized_expression/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_multiline_parenthesized_expression/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_multiline_parenthesized_expression/sample_default.out.java");
const SELF_ALIGNED: &str =
    include_str!("../java/align_multiline_parenthesized_expression/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_multiline_parenthesized_expression/self_aligned.out.java");

fn wrap(align: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.binary_operation_wrap = WrapStyle::ChopDownIfLong;
        s.align_multiline_parenthesized_expression = align;
    })
}

#[test]
fn align_on_aligns_the_wrapped_content_under_the_open_paren() {
    assert_eq!(format_with(SAMPLE, &wrap(true)), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_the_wrapped_content_at_the_continuation_indent() {
    assert_eq!(format_with(SAMPLE, &wrap(false)), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_keeps_the_wrapped_content_at_the_continuation_indent() {
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
