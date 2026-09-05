//! ALIGN_THROWS_KEYWORD — align the `throws` keyword when it is wrapped onto
//! its own continuation line.
//! Fixtures live under tests/java/align_throws_keyword/.
//!
//! With `THROWS_KEYWORD_WRAP` the keyword starts a continuation line; when
//! this option is on that line starts at the column the keyword would occupy
//! if it had stayed on the header line after the parameter list (the shape is
//! pinned by the goldens — IntelliJ's exact behaviour for this option is
//! ambiguous and the natural-position model is followed).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const SAMPLE: &str = include_str!("../java/align_throws_keyword/sample.java");
const SAMPLE_ALIGN_OUT: &str = include_str!("../java/align_throws_keyword/sample_align.out.java");
const SAMPLE_CONT_OUT: &str = include_str!("../java/align_throws_keyword/sample_cont.out.java");
const SAMPLE_DEFAULT_OUT: &str =
    include_str!("../java/align_throws_keyword/sample_default.out.java");
const SELF_ALIGNED: &str = include_str!("../java/align_throws_keyword/self_aligned.java");
const SELF_ALIGNED_OUT: &str = include_str!("../java/align_throws_keyword/self_aligned.out.java");

fn wrap(align: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.throws_list_wrap = WrapStyle::WrapIfLong;
        s.throws_keyword_wrap = true;
        s.align_throws_keyword = align;
    })
}

#[test]
fn align_on_places_the_keyword_at_its_natural_header_column() {
    assert_eq!(format_with(SAMPLE, &wrap(true)), SAMPLE_ALIGN_OUT);
}

#[test]
fn align_off_places_the_keyword_at_the_continuation_indent() {
    assert_eq!(format_with(SAMPLE, &wrap(false)), SAMPLE_CONT_OUT);
}

#[test]
fn absent_default_places_the_keyword_at_the_continuation_indent() {
    // The option defaults to false; the style sets only the wrap toggles.
    let style = style(|s| {
        s.right_margin = 60;
        s.throws_list_wrap = WrapStyle::WrapIfLong;
        s.throws_keyword_wrap = true;
    });
    assert_eq!(format_with(SAMPLE, &style), SAMPLE_DEFAULT_OUT);
}

#[test]
fn reformatting_the_aligned_layout_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    assert_eq!(format_with(SELF_ALIGNED, &wrap(true)), SELF_ALIGNED_OUT);
}
