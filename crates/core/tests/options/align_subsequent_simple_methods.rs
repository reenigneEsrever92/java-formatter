//! ALIGN_SUBSEQUENT_SIMPLE_METHODS — align the names of output-adjacent
//! one-line method members.
//! Fixtures live under tests/java/align_subsequent_simple_methods/.
//!
//! A run is a maximal stretch of adjacent method members that render on a
//! single line (empty or `KEEP_SIMPLE_METHODS_IN_ONE_LINE` bodies) with no
//! blank line and no comment between them (methods are output-adjacent when
//! `BLANK_LINES_AROUND_METHOD` is 0, the default); each member's
//! `[modifiers ]type ` prefix is padded so the method names share one column.
//! Multi-line methods break runs.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const SAMPLE: &str = include_str!("../java/align_subsequent_simple_methods/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_subsequent_simple_methods/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_subsequent_simple_methods/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_subsequent_simple_methods/sample_default.out.java");
const SELF_ALIGNED: &str =
    include_str!("../java/align_subsequent_simple_methods/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_subsequent_simple_methods/self_aligned.out.java");

fn style_with(align: bool) -> JavaStyle {
    style(|s| {
        s.keep_simple_methods_in_one_line = true;
        s.align_subsequent_simple_methods = align;
    })
}

#[test]
fn align_on_pads_one_line_method_names_into_one_column() {
    assert_eq!(format_with(SAMPLE, &style_with(true)), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_each_method_at_its_natural_column() {
    assert_eq!(format_with(SAMPLE, &style_with(false)), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_leaves_methods_unaligned() {
    // The option defaults to false, so the plain default style emits each
    // method at its natural column.
    assert_eq!(format(SAMPLE), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_aligned_methods_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    assert_eq!(
        format_with(SELF_ALIGNED, &style_with(true)),
        SELF_ALIGNED_OUT
    );
}
