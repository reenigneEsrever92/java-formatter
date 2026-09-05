//! ALIGN_GROUP_FIELD_DECLARATIONS — align the declared names of output-adjacent
//! field declarations in a column.
//! Fixtures live under tests/java/align_group_field_declarations/.
//!
//! A run is a maximal stretch of adjacent single-declarator field members with
//! no blank line and no comment between them (fields are output-adjacent when
//! `BLANK_LINES_AROUND_FIELD` is 0, the default); each member's
//! `[modifiers ]type name` prefix is padded so the names share one column.
//! Annotated, multi-declarator and multi-line fields break runs.

use super::common::*;

const SAMPLE: &str = include_str!("../java/align_group_field_declarations/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_group_field_declarations/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_group_field_declarations/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_group_field_declarations/sample_default.out.java");
const SELF_ALIGNED: &str = include_str!("../java/align_group_field_declarations/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_group_field_declarations/self_aligned.out.java");

#[test]
fn align_on_pads_field_names_into_one_column() {
    let style = style(|s| s.align_group_field_declarations = true);
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_each_field_at_its_natural_column() {
    let style = style(|s| s.align_group_field_declarations = false);
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_leaves_fields_unaligned() {
    // The option defaults to false, so the plain default style emits each
    // field at its natural column.
    assert_eq!(format(SAMPLE), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_aligned_fields_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    let style = style(|s| s.align_group_field_declarations = true);
    assert_eq!(format_with(SELF_ALIGNED, &style), SELF_ALIGNED_OUT);
}
