//! CALL_PARAMETERS_RPAREN_ON_NEXT_LINE — put the ')' of a wrapped call on its
//! own line.
//! Fixtures live under tests/java/call_parameters_rparen_on_next_line/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const LONG_CALL: &str = include_str!("../java/call_parameters_rparen_on_next_line/long_call.java");
const LONG_CALL_RPAREN_OUT: &str =
    include_str!("../java/call_parameters_rparen_on_next_line/long_call.out.java");
const LONG_CALL_RPAREN_OFF_OUT: &str =
    include_str!("../java/call_parameters_rparen_on_next_line/long_call_rparen_off.out.java");

#[test]
fn rparen_on_next_line_puts_the_rparen_on_its_own_line() {
    let style = style(|s| {
        s.right_margin = 40;
        s.call_parameters_wrap = WrapStyle::WrapIfLong;
        s.call_parameters_rparen_on_next_line = true;
    });
    assert_eq!(format_with(LONG_CALL, &style), LONG_CALL_RPAREN_OUT);
}

#[test]
fn rparen_off_attaches_the_rparen_to_the_last_argument() {
    let style = style(|s| {
        s.right_margin = 40;
        s.call_parameters_wrap = WrapStyle::WrapIfLong;
    });
    assert_eq!(format_with(LONG_CALL, &style), LONG_CALL_RPAREN_OFF_OUT);
}
