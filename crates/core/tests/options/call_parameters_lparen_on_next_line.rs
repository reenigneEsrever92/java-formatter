//! CALL_PARAMETERS_LPAREN_ON_NEXT_LINE — break after '(' in wrapped calls.
//! Fixtures live under tests/java/call_parameters_lparen_on_next_line/.
//!
//! The layout effect of the option only shows once the closing paren also sits
//! on its own line (`call_parameters_rparen_on_next_line`); with the attached
//! `)` the formatter emits the same layout for both lparen settings, so the
//! rparen option is pinned on in this file to isolate the lparen behaviour.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_CALL: &str = include_str!("../java/call_parameters_lparen_on_next_line/long_call.java");
const LONG_CALL_LPAREN_OUT: &str =
    include_str!("../java/call_parameters_lparen_on_next_line/long_call.out.java");
const LONG_CALL_LPAREN_OFF_OUT: &str =
    include_str!("../java/call_parameters_lparen_on_next_line/long_call_lparen_off.out.java");

fn wrapped_with(lparen_on_next_line: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.call_parameters_wrap = WrapStyle::WrapIfLong;
        s.call_parameters_lparen_on_next_line = lparen_on_next_line;
        s.call_parameters_rparen_on_next_line = true;
    })
}

#[test]
fn lparen_on_next_line_breaks_after_the_lparen() {
    assert_eq!(
        format_with(LONG_CALL, &wrapped_with(true)),
        LONG_CALL_LPAREN_OUT
    );
}

#[test]
fn lparen_off_keeps_the_first_argument_on_the_lparen_line() {
    assert_eq!(
        format_with(LONG_CALL, &wrapped_with(false)),
        LONG_CALL_LPAREN_OFF_OUT
    );
}
