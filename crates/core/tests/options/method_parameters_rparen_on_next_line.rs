//! METHOD_PARAMETERS_RPAREN_ON_NEXT_LINE — put the ')' of a wrapped method /
//! constructor declaration on its own line.
//! Fixtures live under tests/java/method_parameters_rparen_on_next_line/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const LONG_PARAMS: &str =
    include_str!("../java/method_parameters_rparen_on_next_line/long_params.java");
const LONG_PARAMS_OUT: &str =
    include_str!("../java/method_parameters_rparen_on_next_line/long_params.out.java");
const LONG_PARAMS_RPAREN_OFF_OUT: &str =
    include_str!("../java/method_parameters_rparen_on_next_line/long_params_rparen_off.out.java");

#[test]
fn rparen_goes_on_its_own_line_when_wrapped() {
    let style = style(|s| {
        s.right_margin = 40;
        s.method_parameters_wrap = WrapStyle::ChopDownIfLong;
        s.method_parameters_lparen_on_next_line = true;
        s.method_parameters_rparen_on_next_line = true;
    });
    assert_eq!(format_with(LONG_PARAMS, &style), LONG_PARAMS_OUT);
}

#[test]
fn rparen_stays_attached_to_last_parameter_when_disabled() {
    // The ')' stays glued to the last parameter instead of closing alone.
    let style = style(|s| {
        s.right_margin = 40;
        s.method_parameters_wrap = WrapStyle::ChopDownIfLong;
        s.method_parameters_lparen_on_next_line = true;
        s.method_parameters_rparen_on_next_line = false;
    });
    assert_eq!(format_with(LONG_PARAMS, &style), LONG_PARAMS_RPAREN_OFF_OUT);
}
