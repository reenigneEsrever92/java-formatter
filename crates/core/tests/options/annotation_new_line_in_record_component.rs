//! ANNOTATION_NEW_LINE_IN_RECORD_COMPONENT — put each annotation of a record
//! component that begins its own line on its own line.
//! Fixtures live under tests/java/annotation_new_line_in_record_component/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const COMPONENT_WRAP: &str =
    include_str!("../java/annotation_new_line_in_record_component/component_wrap.java");
const COMPONENT_WRAP_ANNOTATED_OUT: &str = include_str!(
    "../java/annotation_new_line_in_record_component/component_wrap_annotated.out.java"
);
const COMPONENT_WRAP_INLINE_OUT: &str =
    include_str!("../java/annotation_new_line_in_record_component/component_wrap_inline.out.java");
const COMPONENT_WRAP_FIRST_INLINE_OUT: &str = include_str!(
    "../java/annotation_new_line_in_record_component/component_wrap_first_inline.out.java"
);
const COMPONENT_WRAP_SELF: &str =
    include_str!("../java/annotation_new_line_in_record_component/component_wrap_self.java");
const COMPONENT_WRAP_SELF_OUT: &str =
    include_str!("../java/annotation_new_line_in_record_component/component_wrap_self.out.java");

fn annotated(lparen_new_line: bool) -> JavaStyle {
    style(|s| {
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.align_multiline_records = false;
        s.new_line_after_lparen_in_record_header = lparen_new_line;
        s.annotation_new_line_in_record_component = true;
    })
}

#[test]
fn each_annotation_goes_on_its_own_line_above_the_component() {
    // Own-line components split into one annotation per line plus the
    // declaration core (annotation tokens verbatim); the plain component and
    // the wrapped header's parens are unchanged.
    let style = annotated(true);
    assert_eq!(
        format_with(COMPONENT_WRAP, &style),
        COMPONENT_WRAP_ANNOTATED_OUT
    );
}

#[test]
fn annotations_stay_inline_when_disabled() {
    // The shipped wrapped layout keeps each component's annotations inline.
    let style = style(|s| {
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.align_multiline_records = false;
        s.new_line_after_lparen_in_record_header = true;
    });
    assert_eq!(
        format_with(COMPONENT_WRAP, &style),
        COMPONENT_WRAP_INLINE_OUT
    );
}

#[test]
fn the_first_inline_component_keeps_its_annotation_inline() {
    // In the lparen-attached layout the first component shares the '(' line,
    // so its annotation stays inline; the own-line components split.
    let style = annotated(false);
    assert_eq!(
        format_with(COMPONENT_WRAP, &style),
        COMPONENT_WRAP_FIRST_INLINE_OUT
    );
}

#[test]
fn reformatting_the_annotated_layout_is_a_no_op() {
    let style = annotated(true);
    assert_eq!(
        format_with(COMPONENT_WRAP_SELF, &style),
        COMPONENT_WRAP_SELF_OUT
    );
}
