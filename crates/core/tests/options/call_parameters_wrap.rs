//! CALL_PARAMETERS_WRAP — wrapping of method-call argument lists.
//! Fixtures live under tests/java/call_parameters_wrap/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_CALL: &str = include_str!("../java/call_parameters_wrap/long_call.java");
const LONG_CALL_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/call_parameters_wrap/long_call.out.java");
const LONG_CALL_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/call_parameters_wrap/long_call_do_not_wrap.out.java");
const LONG_CALL_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/call_parameters_wrap/long_call_wrap_always.out.java");
const SHORT_CALL: &str = include_str!("../java/call_parameters_wrap/short_call.java");
const SHORT_CALL_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/call_parameters_wrap/short_call.out.java");

/// A style with a tight margin so the long call overflows it.
fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.call_parameters_wrap = wrap;
    })
}

#[test]
fn wrap_if_long_wraps_the_argument_list() {
    assert_eq!(
        format_with(LONG_CALL, &narrow(WrapStyle::WrapIfLong)),
        LONG_CALL_WRAP_IF_LONG_OUT
    );
}

#[test]
fn do_not_wrap_keeps_the_call_on_one_line() {
    assert_eq!(
        format_with(LONG_CALL, &narrow(WrapStyle::DoNotWrap)),
        LONG_CALL_DO_NOT_WRAP_OUT
    );
}

#[test]
fn wrap_always_wraps_a_call_that_overflows_the_margin() {
    assert_eq!(
        format_with(LONG_CALL, &narrow(WrapStyle::WrapAlways)),
        LONG_CALL_WRAP_ALWAYS_OUT
    );
}

#[test]
fn wrap_always_keeps_short_calls_flat() {
    assert_eq!(
        format_with(SHORT_CALL, &narrow(WrapStyle::WrapAlways)),
        SHORT_CALL_WRAP_ALWAYS_OUT
    );
}
