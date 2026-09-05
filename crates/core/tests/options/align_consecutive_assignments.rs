//! ALIGN_CONSECUTIVE_ASSIGNMENTS — align the `=` of consecutive assignment
//! statements in a column.
//! Fixtures live under tests/java/align_consecutive_assignments/.
//!
//! A run is a maximal stretch of adjacent assignment statements with no blank
//! line and no comment between them; the operator column of the run is shared,
//! padding the shorter left sides. Assignment statements in other kinds of
//! blocks (inside control statements) are part of the same block-level runs.

use super::common::*;

const SAMPLE: &str = include_str!("../java/align_consecutive_assignments/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_consecutive_assignments/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_consecutive_assignments/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_consecutive_assignments/sample_default.out.java");
const SELF_ALIGNED: &str = include_str!("../java/align_consecutive_assignments/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_consecutive_assignments/self_aligned.out.java");

#[test]
fn align_on_pads_the_assignment_operators_of_a_run() {
    // `LINE_COMMENT_AT_FIRST_COLUMN` is pinned off so the run-breaking
    // comment stays at the statement indent in the fixtures.
    let style = style(|s| {
        s.line_comment_at_first_column = false;
        s.align_consecutive_assignments = true;
    });
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_each_assignment_at_its_natural_column() {
    let style = style(|s| {
        s.line_comment_at_first_column = false;
        s.align_consecutive_assignments = false;
    });
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_leaves_assignments_unaligned() {
    // The option defaults to false, so the plain default style emits each
    // assignment statement at its natural column.
    assert_eq!(format(SAMPLE), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_aligned_assignments_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    let style = style(|s| {
        s.line_comment_at_first_column = false;
        s.align_consecutive_assignments = true;
    });
    assert_eq!(format_with(SELF_ALIGNED, &style), SELF_ALIGNED_OUT);
}
