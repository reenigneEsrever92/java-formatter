//! SPACE_BEFORE_DECONSTRUCTION_LIST — put one space between the record type
//! and the pattern list's '(' (`case Point (int x)` vs `case Point(int x)`).
//! Fixtures live under tests/java/space_before_deconstruction_list/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const LABEL: &str = include_str!("../java/space_before_deconstruction_list/label.java");
const LABEL_SPACED_OUT: &str =
    include_str!("../java/space_before_deconstruction_list/label_spaced.out.java");
const LABEL_DEFAULT_OUT: &str =
    include_str!("../java/space_before_deconstruction_list/label_default.out.java");
const LABEL_SPACED_SELF: &str =
    include_str!("../java/space_before_deconstruction_list/label_spaced_self.java");
const LABEL_SPACED_SELF_OUT: &str =
    include_str!("../java/space_before_deconstruction_list/label_spaced_self.out.java");
const CASE_WRAP: &str = include_str!("../java/space_before_deconstruction_list/case_wrap.java");
const CASE_WRAP_SPACED_OUT: &str =
    include_str!("../java/space_before_deconstruction_list/case_wrap_spaced.out.java");

#[test]
fn puts_a_space_before_the_pattern_list_when_enabled() {
    let style = style(|s| s.space_before_deconstruction_list = true);
    assert_eq!(format_with(LABEL, &style), LABEL_SPACED_OUT);
}

#[test]
fn absent_option_keeps_the_compact_label() {
    // The absent option defaults to false, so the label stays unchanged.
    assert_eq!(format(LABEL), LABEL_DEFAULT_OUT);
}

#[test]
fn reformatting_the_spaced_label_is_a_no_op() {
    let style = style(|s| s.space_before_deconstruction_list = true);
    assert_eq!(
        format_with(LABEL_SPACED_SELF, &style),
        LABEL_SPACED_SELF_OUT
    );
}

#[test]
fn shifts_the_open_paren_of_a_wrapped_label() {
    // The space moves the '(' (and with it the aligned components) right one
    // column in the wrapped layout too.
    let style = style(|s| {
        s.deconstruction_list_wrap = WrapStyle::WrapAlways;
        s.space_before_deconstruction_list = true;
    });
    assert_eq!(format_with(CASE_WRAP, &style), CASE_WRAP_SPACED_OUT);
}
