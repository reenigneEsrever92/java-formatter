//! ALIGN_MULTILINE_RESOURCES — align the clauses of a wrapped
//! try-with-resources list under the first resource.
//! Fixtures live under tests/java/align_multiline_resources/.
//!
//! The option defaults to true: when the first resource stays on the header
//! line after `(` and the rest wrap (the lparen-stays / rparen-alone
//! layout), the wrapped lines pad to the column after `(` instead of the
//! continuation indent. Where every resource begins its own line the clauses
//! already share the first resource's column, so the option leaves them
//! unchanged. The absent case (default true) is the aligned layout.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const SAMPLE: &str = include_str!("../java/align_multiline_resources/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_multiline_resources/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_multiline_resources/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_multiline_resources/sample_default.out.java");
const SELF_ALIGNED: &str = include_str!("../java/align_multiline_resources/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_multiline_resources/self_aligned.out.java");

fn wrap(align: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.resource_list_wrap = WrapStyle::WrapIfLong;
        s.resource_list_rparen_on_next_line = true;
        s.align_multiline_resources = align;
    })
}

#[test]
fn align_on_aligns_wrapped_resources_under_the_first_resource() {
    assert_eq!(format_with(SAMPLE, &wrap(true)), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_wrapped_resources_at_the_continuation_indent() {
    assert_eq!(format_with(SAMPLE, &wrap(false)), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_aligns_wrapped_resources() {
    // The option defaults to true: the style sets only the wrap and rparen
    // toggles, and the wrapped resources align under the first resource.
    let style = style(|s| {
        s.right_margin = 60;
        s.resource_list_wrap = WrapStyle::WrapIfLong;
        s.resource_list_rparen_on_next_line = true;
    });
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_the_aligned_layout_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    assert_eq!(format_with(SELF_ALIGNED, &wrap(true)), SELF_ALIGNED_OUT);
}
