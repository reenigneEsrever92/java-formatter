//! RPAREN_ON_NEW_LINE_IN_DECONSTRUCTION_PATTERN — the ')' of a wrapped record
//! pattern closes alone at the label indent when on, and hugs the last
//! component when off.
//! Fixtures live under tests/java/rparen_on_new_line_in_deconstruction_pattern/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const CASE_WRAP: &str =
    include_str!("../java/rparen_on_new_line_in_deconstruction_pattern/case_wrap.java");
const CASE_WRAP_RPAREN_ON_OUT: &str = include_str!(
    "../java/rparen_on_new_line_in_deconstruction_pattern/case_wrap_rparen_on.out.java"
);
const CASE_WRAP_RPAREN_OFF_OUT: &str = include_str!(
    "../java/rparen_on_new_line_in_deconstruction_pattern/case_wrap_rparen_off.out.java"
);
const CASE_WRAP_RPAREN_ON_SELF: &str = include_str!(
    "../java/rparen_on_new_line_in_deconstruction_pattern/case_wrap_rparen_on_self.java"
);
const CASE_WRAP_RPAREN_ON_SELF_OUT: &str = include_str!(
    "../java/rparen_on_new_line_in_deconstruction_pattern/case_wrap_rparen_on_self.out.java"
);

fn wrapped(rparen_on: bool) -> JavaStyle {
    style(|s| {
        s.deconstruction_list_wrap = WrapStyle::WrapAlways;
        s.rparen_on_new_line_in_deconstruction_pattern = rparen_on;
    })
}

#[test]
fn rparen_goes_on_its_own_line_when_enabled() {
    // The wrapped label's ')' closes alone at the label indent.
    let style = wrapped(true);
    assert_eq!(format_with(CASE_WRAP, &style), CASE_WRAP_RPAREN_ON_OUT);
}

#[test]
fn rparen_stays_glued_to_the_last_component_when_disabled() {
    let style = wrapped(false);
    assert_eq!(format_with(CASE_WRAP, &style), CASE_WRAP_RPAREN_OFF_OUT);
}

#[test]
fn reformatting_the_rparen_layout_is_a_no_op() {
    let style = wrapped(true);
    assert_eq!(
        format_with(CASE_WRAP_RPAREN_ON_SELF, &style),
        CASE_WRAP_RPAREN_ON_SELF_OUT
    );
}
