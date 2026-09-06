//! METHOD_PARAMETERS_WRAP — wrapping of method / constructor parameter lists.
//! Fixtures live under tests/java/method_parameters_wrap/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const WRAPPED_PARAMS: &str = include_str!("../java/method_parameters_wrap/wrapped_params.java");
const WRAPPED_PARAMS_OUT: &str =
    include_str!("../java/method_parameters_wrap/wrapped_params.out.java");

#[test]
fn chop_down_if_long_wraps_parameter_lists() {
    // The declaration overflows the margin, so ChopDownIfLong puts each
    // parameter on its own line (the throws clause in the fixture is
    // incidental to this option).
    let style = style(|s| {
        s.right_margin = 60;
        s.method_parameters_wrap = WrapStyle::ChopDownIfLong;
        s.method_parameters_lparen_on_next_line = true;
        s.method_parameters_rparen_on_next_line = true;
    });
    assert_eq!(format_with(WRAPPED_PARAMS, &style), WRAPPED_PARAMS_OUT);
}
