//! SPACE_WITHIN_DECONSTRUCTION_LIST — put one space just inside the parens of
//! a record pattern that share a line with a component (`case Point( int x )`
//! vs `case Point(int x)`); a paren alone on its own line gets no pad.
//! Fixtures live under tests/java/space_within_deconstruction_list/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const LABEL: &str = include_str!("../java/space_within_deconstruction_list/label.java");
const LABEL_SPACED_OUT: &str =
    include_str!("../java/space_within_deconstruction_list/label_spaced.out.java");
const LABEL_DEFAULT_OUT: &str =
    include_str!("../java/space_within_deconstruction_list/label_default.out.java");
const LABEL_SPACED_SELF: &str =
    include_str!("../java/space_within_deconstruction_list/label_spaced_self.java");
const LABEL_SPACED_SELF_OUT: &str =
    include_str!("../java/space_within_deconstruction_list/label_spaced_self.out.java");
const CASE_WRAP: &str = include_str!("../java/space_within_deconstruction_list/case_wrap.java");
const CASE_WRAP_PAD_HUG_OUT: &str =
    include_str!("../java/space_within_deconstruction_list/case_wrap_pad_hug.out.java");

#[test]
fn pads_the_flat_label_when_enabled() {
    let style = style(|s| s.space_within_deconstruction_list = true);
    assert_eq!(format_with(LABEL, &style), LABEL_SPACED_OUT);
}

#[test]
fn absent_option_keeps_the_flat_label_unpadded() {
    // The absent option defaults to false, so the label stays unchanged.
    assert_eq!(format(LABEL), LABEL_DEFAULT_OUT);
}

#[test]
fn reformatting_the_spaced_label_is_a_no_op() {
    let style = style(|s| s.space_within_deconstruction_list = true);
    assert_eq!(
        format_with(LABEL_SPACED_SELF, &style),
        LABEL_SPACED_SELF_OUT
    );
}

#[test]
fn pads_only_a_paren_that_shares_a_line_with_a_component() {
    // In the wrapped layout each component has its own line, so only the
    // hugging ')' (rparen off) carries the pad.
    let style = style(|s| {
        s.deconstruction_list_wrap = WrapStyle::WrapAlways;
        s.space_within_deconstruction_list = true;
        s.rparen_on_new_line_in_deconstruction_pattern = false;
    });
    assert_eq!(format_with(CASE_WRAP, &style), CASE_WRAP_PAD_HUG_OUT);
}
