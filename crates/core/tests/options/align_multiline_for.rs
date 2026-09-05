//! ALIGN_MULTILINE_FOR — align the parts of a wrapped `for` header.
//! Fixtures live under tests/java/align_multiline_for/.
//!
//! When the header wraps, the cond / update (and the enhanced `for` value)
//! continuation lines start under the first slot after `(` — the init / type
//! column — instead of the continuation indent. The option defaults to true,
//! so the absent case is the aligned layout.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const SAMPLE: &str = include_str!("../java/align_multiline_for/sample.java");
const SAMPLE_ALIGN_OUT: &str = include_str!("../java/align_multiline_for/sample_align.out.java");
const SAMPLE_CONT_OUT: &str = include_str!("../java/align_multiline_for/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_multiline_for/sample_default.out.java");
const SELF_ALIGNED: &str = include_str!("../java/align_multiline_for/self_aligned.java");
const SELF_ALIGNED_OUT: &str = include_str!("../java/align_multiline_for/self_aligned.out.java");

fn wrap(align: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.for_statement_wrap = WrapStyle::WrapIfLong;
        s.align_multiline_for = align;
    })
}

#[test]
fn align_on_aligns_header_parts_under_the_first_slot() {
    assert_eq!(format_with(SAMPLE, &wrap(true)), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_header_parts_at_the_continuation_indent() {
    assert_eq!(format_with(SAMPLE, &wrap(false)), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_aligns_header_parts() {
    // The option defaults to true: the style sets only the wrap toggle, and
    // the wrapped parts align under the first slot.
    let style = style(|s| {
        s.right_margin = 60;
        s.for_statement_wrap = WrapStyle::WrapIfLong;
    });
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_the_aligned_layout_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    assert_eq!(format_with(SELF_ALIGNED, &wrap(true)), SELF_ALIGNED_OUT);
}
