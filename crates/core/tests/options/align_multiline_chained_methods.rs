//! ALIGN_MULTILINE_CHAINED_METHODS — align the dots of a wrapped chained call
//! under the first call's dot.
//! Fixtures live under tests/java/align_multiline_chained_methods/.
//!
//! When the chain wraps and its first link stays on the header line, each
//! continuation link line starts at the first link's dot column instead of the
//! continuation indent.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const SAMPLE: &str = include_str!("../java/align_multiline_chained_methods/sample.java");
const SAMPLE_ALIGN_OUT: &str =
    include_str!("../java/align_multiline_chained_methods/sample_align.out.java");
const SAMPLE_CONT_OUT: &str =
    include_str!("../java/align_multiline_chained_methods/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_multiline_chained_methods/sample_default.out.java");
const SELF_ALIGNED: &str =
    include_str!("../java/align_multiline_chained_methods/self_aligned.java");
const SELF_ALIGNED_OUT: &str =
    include_str!("../java/align_multiline_chained_methods/self_aligned.out.java");

fn wrap(align: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.method_call_chain_wrap = WrapStyle::WrapAlways;
        s.align_multiline_chained_methods = align;
    })
}

#[test]
fn align_on_aligns_continuation_links_under_the_first_dot() {
    assert_eq!(format_with(SAMPLE, &wrap(true)), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_keeps_continuation_links_at_the_continuation_indent() {
    assert_eq!(format_with(SAMPLE, &wrap(false)), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_keeps_continuation_links_at_the_continuation_indent() {
    // The option defaults to false; the style sets only the wrap toggle.
    let style = style(|s| {
        s.right_margin = 60;
        s.method_call_chain_wrap = WrapStyle::WrapAlways;
    });
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_the_aligned_layout_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    assert_eq!(format_with(SELF_ALIGNED, &wrap(true)), SELF_ALIGNED_OUT);
}
