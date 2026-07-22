//! NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER — put the '(' of a wrapped record
//! header on its own line.
//! Fixtures live under tests/java/new_line_after_lparen_in_record_header/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const COMPONENT_WRAP: &str =
    include_str!("../java/new_line_after_lparen_in_record_header/component_wrap.java");
const COMPONENT_WRAP_NEWLINE_LPAREN_OUT: &str = include_str!(
    "../java/new_line_after_lparen_in_record_header/component_wrap_newline_lparen.out.java"
);
const COMPONENT_WRAP_LPAREN_OFF_OUT: &str = include_str!(
    "../java/new_line_after_lparen_in_record_header/component_wrap_lparen_off.out.java"
);

#[test]
fn lparen_goes_on_its_own_line_in_wrapped_header() {
    // The '(' goes alone, every component on its own line and the ')' closes at
    // the record indent.
    let style = style(|s| {
        s.right_margin = 40;
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.align_multiline_records = false;
        s.new_line_after_lparen_in_record_header = true;
    });
    assert_eq!(
        format_with(COMPONENT_WRAP, &style),
        COMPONENT_WRAP_NEWLINE_LPAREN_OUT
    );
}

#[test]
fn lparen_stays_attached_when_disabled() {
    // The '(' stays attached and the first component follows it on the record
    // header line.
    let style = style(|s| {
        s.right_margin = 40;
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.align_multiline_records = false;
        s.new_line_after_lparen_in_record_header = false;
    });
    assert_eq!(format_with(COMPONENT_WRAP, &style), COMPONENT_WRAP_LPAREN_OFF_OUT);
}
