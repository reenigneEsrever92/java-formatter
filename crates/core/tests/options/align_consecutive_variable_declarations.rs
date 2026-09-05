//! ALIGN_CONSECUTIVE_VARIABLE_DECLARATIONS — align the declared names of
//! consecutive local variable declarations in a column.
//! Fixtures live under tests/java/align_consecutive_variable_declarations/.
//!
//! A run is a maximal stretch of adjacent single-declarator declarations with
//! no blank line and no comment between them; each member's `type name` prefix
//! is padded so the names share one column. Multi-declarator declarations and
//! declarations that render over several lines break runs.

use super::common::*;

const SAMPLE: &str = include_str!("../java/align_consecutive_variable_declarations/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_consecutive_variable_declarations/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_consecutive_variable_declarations/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_consecutive_variable_declarations/sample_default.out.java");
const SELF_ALIGNED: &str =
    include_str!("../java/align_consecutive_variable_declarations/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_consecutive_variable_declarations/self_aligned.out.java");

#[test]
fn align_on_pads_declaration_names_into_one_column() {
    let style = style(|s| s.align_consecutive_variable_declarations = true);
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_each_declaration_at_its_natural_column() {
    let style = style(|s| s.align_consecutive_variable_declarations = false);
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_leaves_declarations_unaligned() {
    // The option defaults to false, so the plain default style emits each
    // declaration at its natural column.
    assert_eq!(format(SAMPLE), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_aligned_declarations_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    let style = style(|s| s.align_consecutive_variable_declarations = true);
    assert_eq!(format_with(SELF_ALIGNED, &style), SELF_ALIGNED_OUT);
}
