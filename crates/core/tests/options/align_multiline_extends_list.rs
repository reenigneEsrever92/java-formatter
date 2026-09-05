//! ALIGN_MULTILINE_EXTENDS_LIST — align wrapped `extends` / `implements` list
//! entries under the first entry.
//! Fixtures live under tests/java/align_multiline_extends_list/.
//!
//! When a class / interface / enum / record header's clause list wraps, the
//! continuation entry lines start at the first entry's column instead of the
//! continuation indent. `EXTENDS_KEYWORD_WRAP` still steers the keyword
//! independently.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const SAMPLE: &str = include_str!("../java/align_multiline_extends_list/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_multiline_extends_list/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_multiline_extends_list/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_multiline_extends_list/sample_default.out.java");
const SELF_ALIGNED: &str = include_str!("../java/align_multiline_extends_list/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_multiline_extends_list/self_aligned.out.java");

fn wrap(align: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.extends_list_wrap = WrapStyle::WrapIfLong;
        s.align_multiline_extends_list = align;
    })
}

#[test]
fn align_on_aligns_continuation_entries_under_the_first_entry() {
    assert_eq!(format_with(SAMPLE, &wrap(true)), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_continuation_entries_at_the_continuation_indent() {
    assert_eq!(format_with(SAMPLE, &wrap(false)), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_keeps_continuation_entries_at_the_continuation_indent() {
    // The option defaults to false; the style sets only the wrap toggle.
    let style = style(|s| {
        s.right_margin = 60;
        s.extends_list_wrap = WrapStyle::WrapIfLong;
    });
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_the_aligned_layout_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    assert_eq!(format_with(SELF_ALIGNED, &wrap(true)), SELF_ALIGNED_OUT);
}
