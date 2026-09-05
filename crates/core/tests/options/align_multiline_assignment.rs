//! ALIGN_MULTILINE_ASSIGNMENT — align a wrapped assignment's right-hand side.
//! Fixtures live under tests/java/align_multiline_assignment/.
//!
//! When the RHS moves to a continuation line it starts at the column right
//! after the operator (where it would sit on the header line) instead of the
//! continuation indent, and nested expression continuations inside the RHS
//! align to that column too. The statement, local-variable and field
//! initialiser sites are covered.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const SAMPLE: &str = include_str!("../java/align_multiline_assignment/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_multiline_assignment/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_multiline_assignment/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_multiline_assignment/sample_default.out.java");
const SELF_ALIGNED: &str = include_str!("../java/align_multiline_assignment/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_multiline_assignment/self_aligned.out.java");

fn wrap(align: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.assignment_wrap = WrapStyle::WrapAlways;
        s.align_multiline_assignment = align;
    })
}

#[test]
fn align_on_starts_the_wrapped_rhs_under_the_operator() {
    assert_eq!(format_with(SAMPLE, &wrap(true)), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_the_wrapped_rhs_at_the_continuation_indent() {
    assert_eq!(format_with(SAMPLE, &wrap(false)), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_keeps_the_wrapped_rhs_at_the_continuation_indent() {
    // The option defaults to false; the style sets only the wrap toggle.
    let style = style(|s| {
        s.right_margin = 40;
        s.assignment_wrap = WrapStyle::WrapAlways;
    });
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_the_aligned_layout_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    assert_eq!(format_with(SELF_ALIGNED, &wrap(true)), SELF_ALIGNED_OUT);
}
