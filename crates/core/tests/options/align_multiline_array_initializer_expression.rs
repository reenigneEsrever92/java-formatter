//! ALIGN_MULTILINE_ARRAY_INITIALIZER_EXPRESSION — align the entries of a
//! wrapped array initializer under its first entry.
//! Fixtures live under tests/java/align_multiline_array_initializer_expression/.
//!
//! When the option is on and the `{` stays on the header line, the first
//! entry stays right after `{` and the remaining entry lines pad to the
//! column after `{` (the record-header model this formatter pins for the
//! option). With the `{` on its own line — or the option off — entries begin
//! their own lines at the continuation indent.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const SAMPLE: &str =
    include_str!("../java/align_multiline_array_initializer_expression/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_multiline_array_initializer_expression/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_multiline_array_initializer_expression/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_multiline_array_initializer_expression/sample_default.out.java");
const SELF_ALIGNED: &str =
    include_str!("../java/align_multiline_array_initializer_expression/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_multiline_array_initializer_expression/self_aligned.out.java");

fn wrap(align: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.array_initializer_wrap = WrapStyle::ChopDownIfLong;
        s.align_multiline_array_initializer_expression = align;
    })
}

#[test]
fn align_on_glues_the_first_entry_and_aligns_the_rest_under_it() {
    assert_eq!(format_with(SAMPLE, &wrap(true)), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_entries_on_their_own_lines() {
    assert_eq!(format_with(SAMPLE, &wrap(false)), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_keeps_entries_on_their_own_lines() {
    // The option defaults to false; the style sets only the wrap toggle.
    let style = style(|s| {
        s.right_margin = 60;
        s.array_initializer_wrap = WrapStyle::ChopDownIfLong;
    });
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_the_aligned_layout_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    assert_eq!(format_with(SELF_ALIGNED, &wrap(true)), SELF_ALIGNED_OUT);
}
