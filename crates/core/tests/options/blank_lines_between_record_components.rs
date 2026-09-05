//! BLANK_LINES_BETWEEN_RECORD_COMPONENTS — blank lines between the components
//! of a wrapped record header.
//! Fixtures live under tests/java/blank_lines_between_record_components/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const COMPONENT_WRAP: &str =
    include_str!("../java/blank_lines_between_record_components/component_wrap.java");
const COMPONENT_WRAP_ONE_OUT: &str =
    include_str!("../java/blank_lines_between_record_components/component_wrap_one.out.java");
const COMPONENT_WRAP_NONE_OUT: &str =
    include_str!("../java/blank_lines_between_record_components/component_wrap_none.out.java");
const COMPONENT_WRAP_LPAREN_ON_ONE_OUT: &str = include_str!(
    "../java/blank_lines_between_record_components/component_wrap_lparen_on_one.out.java"
);
const COMPONENT_WRAP_SELF: &str =
    include_str!("../java/blank_lines_between_record_components/component_wrap_self.java");
const COMPONENT_WRAP_SELF_OUT: &str =
    include_str!("../java/blank_lines_between_record_components/component_wrap_self.out.java");

fn blank(blank: u32, lparen_new_line: bool) -> JavaStyle {
    style(|s| {
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.align_multiline_records = false;
        s.new_line_after_lparen_in_record_header = lparen_new_line;
        s.blank_lines_between_record_components = blank;
    })
}

#[test]
fn absent_option_inserts_no_blank_lines() {
    // The absent option defaults to 0: the shipped wrapped layout stays.
    let style = blank(0, false);
    assert_eq!(format_with(COMPONENT_WRAP, &style), COMPONENT_WRAP_NONE_OUT);
}

#[test]
fn one_blank_line_separates_the_wrapped_components() {
    let style = blank(1, false);
    assert_eq!(format_with(COMPONENT_WRAP, &style), COMPONENT_WRAP_ONE_OUT);
}

#[test]
fn the_lparen_new_line_layout_gets_the_blank_lines_too() {
    let style = blank(1, true);
    assert_eq!(
        format_with(COMPONENT_WRAP, &style),
        COMPONENT_WRAP_LPAREN_ON_ONE_OUT
    );
}

#[test]
fn reformatting_the_blank_line_layout_is_a_no_op() {
    let style = blank(1, false);
    assert_eq!(
        format_with(COMPONENT_WRAP_SELF, &style),
        COMPONENT_WRAP_SELF_OUT
    );
}
