//! RECORD_COMPONENTS_WRAP — wrapping of record component lists.
//! Fixtures live under tests/java/record_components_wrap/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const COMPONENT_WRAP: &str = include_str!("../java/record_components_wrap/component_wrap.java");
const COMPONENT_WRAP_NEWLINE_LPAREN_OUT: &str =
    include_str!("../java/record_components_wrap/component_wrap_newline_lparen.out.java");
const COMPONENT_WRAP_ALIGN_OUT: &str =
    include_str!("../java/record_components_wrap/component_wrap_align.out.java");
const COMPONENT_WRAP_CONT_OUT: &str =
    include_str!("../java/record_components_wrap/component_wrap_cont.out.java");
const COMPONENTS_FIT: &str = include_str!("../java/record_components_wrap/components_fit.java");
const COMPONENTS_FIT_OUT: &str =
    include_str!("../java/record_components_wrap/components_fit.out.java");
const DO_NOT_WRAP: &str = include_str!("../java/record_components_wrap/do_not_wrap.java");
const DO_NOT_WRAP_OUT: &str = include_str!("../java/record_components_wrap/do_not_wrap.out.java");
const WRAP_WITH_MEMBERS: &str =
    include_str!("../java/record_components_wrap/wrap_with_members.java");
const WRAP_WITH_MEMBERS_OUT: &str =
    include_str!("../java/record_components_wrap/wrap_with_members.out.java");

#[test]
fn wrap_always_with_newline_lparen_puts_lparen_on_own_line() {
    // NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER = true: every component on its own
    // line with continuation indent and the closing paren alone at record indent.
    let style = style(|s| {
        s.right_margin = 40;
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.new_line_after_lparen_in_record_header = true;
        s.align_multiline_records = false;
    });
    assert_eq!(
        format_with(COMPONENT_WRAP, &style),
        COMPONENT_WRAP_NEWLINE_LPAREN_OUT
    );
}

#[test]
fn wrap_always_with_align_aligns_components_under_first() {
    // ALIGN_MULTILINE_RECORDS = true + '(' not on its own line: following
    // components align under the first component.
    let style = style(|s| {
        s.right_margin = 40;
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.new_line_after_lparen_in_record_header = false;
        s.align_multiline_records = true;
    });
    assert_eq!(
        format_with(COMPONENT_WRAP, &style),
        COMPONENT_WRAP_ALIGN_OUT
    );
}

#[test]
fn wrap_always_without_alignment_uses_continuation_indent() {
    let style = style(|s| {
        s.right_margin = 40;
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.new_line_after_lparen_in_record_header = false;
        s.align_multiline_records = false;
    });
    assert_eq!(format_with(COMPONENT_WRAP, &style), COMPONENT_WRAP_CONT_OUT);
}

#[test]
fn wrap_if_long_keeps_components_that_fit_on_one_line() {
    let style = style(|s| {
        s.right_margin = 80;
        s.record_components_wrap = WrapStyle::WrapIfLong;
    });
    assert_eq!(format_with(COMPONENTS_FIT, &style), COMPONENTS_FIT_OUT);
}

#[test]
fn do_not_wrap_keeps_a_single_long_line() {
    let style = style(|s| {
        s.right_margin = 10;
        s.record_components_wrap = WrapStyle::DoNotWrap;
    });
    assert_eq!(format_with(DO_NOT_WRAP, &style), DO_NOT_WRAP_OUT);
}

#[test]
fn wrapped_header_still_formats_full_body() {
    // A wrapped header still formats a full body after it.
    let style = style(|s| {
        s.right_margin = 40;
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.new_line_after_lparen_in_record_header = true;
        s.align_multiline_records = false;
    });
    assert_eq!(
        format_with(WRAP_WITH_MEMBERS, &style),
        WRAP_WITH_MEMBERS_OUT
    );
}
