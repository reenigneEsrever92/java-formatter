//! RPAREN_ON_NEW_LINE_IN_RECORD_HEADER — put the ')' of a wrapped record
//! header on its own line.
//! Fixtures live under tests/java/rparen_on_new_line_in_record_header/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const COMPONENT_WRAP: &str =
    include_str!("../java/rparen_on_new_line_in_record_header/component_wrap.java");
const COMPONENT_WRAP_RPAREN_ON_OUT: &str =
    include_str!("../java/rparen_on_new_line_in_record_header/component_wrap_rparen_on.out.java");
const COMPONENT_WRAP_RPAREN_OFF_OUT: &str =
    include_str!("../java/rparen_on_new_line_in_record_header/component_wrap_rparen_off.out.java");
const COMPONENT_WRAP_SELF: &str =
    include_str!("../java/rparen_on_new_line_in_record_header/component_wrap_self.java");
const COMPONENT_WRAP_SELF_OUT: &str =
    include_str!("../java/rparen_on_new_line_in_record_header/component_wrap_self.out.java");

fn wrapped(rparen_on: bool) -> JavaStyle {
    style(|s| {
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.align_multiline_records = false;
        s.rparen_on_new_line_in_record_header = rparen_on;
    })
}

#[test]
fn rparen_goes_on_its_own_line_when_enabled() {
    // The wrapped header's ')' closes alone at the record indent.
    let style = wrapped(true);
    assert_eq!(
        format_with(COMPONENT_WRAP, &style),
        COMPONENT_WRAP_RPAREN_ON_OUT
    );
}

#[test]
fn rparen_stays_glued_to_the_last_component_when_disabled() {
    // The shipped lparen-attached layout: ')' glues to the last component.
    let style = wrapped(false);
    assert_eq!(
        format_with(COMPONENT_WRAP, &style),
        COMPONENT_WRAP_RPAREN_OFF_OUT
    );
}

#[test]
fn reformatting_the_rparen_layout_is_a_no_op() {
    let style = wrapped(true);
    assert_eq!(
        format_with(COMPONENT_WRAP_SELF, &style),
        COMPONENT_WRAP_SELF_OUT
    );
}
