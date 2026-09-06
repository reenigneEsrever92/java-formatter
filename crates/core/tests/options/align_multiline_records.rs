//! ALIGN_MULTILINE_RECORDS — align wrapped record components under the first
//! component.
//! Fixtures live under tests/java/align_multiline_records/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const COMPONENT_WRAP: &str = include_str!("../java/align_multiline_records/component_wrap.java");
const COMPONENT_WRAP_ALIGN_OUT: &str =
    include_str!("../java/align_multiline_records/component_wrap_align.out.java");
const COMPONENT_WRAP_CONT_OUT: &str =
    include_str!("../java/align_multiline_records/component_wrap_cont.out.java");

#[test]
fn align_true_aligns_components_under_first_component() {
    let style = style(|s| {
        s.right_margin = 40;
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.align_multiline_records = true;
        s.new_line_after_lparen_in_record_header = false;
    });
    assert_eq!(
        format_with(COMPONENT_WRAP, &style),
        COMPONENT_WRAP_ALIGN_OUT
    );
}

#[test]
fn align_false_uses_plain_continuation_indent() {
    let style = style(|s| {
        s.right_margin = 40;
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.align_multiline_records = false;
        s.new_line_after_lparen_in_record_header = false;
    });
    assert_eq!(format_with(COMPONENT_WRAP, &style), COMPONENT_WRAP_CONT_OUT);
}
