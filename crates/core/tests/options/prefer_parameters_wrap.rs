//! PREFER_PARAMETERS_WRAP — prefer wrapping an overflowing call's argument
//! list over breaking its method-call chain.
//! Fixtures live under tests/java/prefer_parameters_wrap/.
//!
//! The fixture's overflowing call is the tail of a chain, so the two option
//! states diverge: off (and absent, since the default is `false`) breaks the
//! chain and keeps the tail's arguments flat, on wraps the arguments of the
//! tail call instead. Only meaningful when the chain can wrap and the
//! argument list can wrap (both `*_WRAP` options set here).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const OVERFLOWING_CHAIN: &str =
    include_str!("../java/prefer_parameters_wrap/overflowing_chain.java");
const OVERFLOWING_CHAIN_PREFER_OUT: &str =
    include_str!("../java/prefer_parameters_wrap/overflowing_chain_prefer.out.java");
const OVERFLOWING_CHAIN_OUT: &str =
    include_str!("../java/prefer_parameters_wrap/overflowing_chain.out.java");

fn style_with(prefer_parameters_wrap: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 90;
        s.method_call_chain_wrap = WrapStyle::WrapIfLong;
        s.call_parameters_wrap = WrapStyle::WrapIfLong;
        s.prefer_parameters_wrap = prefer_parameters_wrap;
    })
}

#[test]
fn prefer_on_wraps_the_tail_calls_arguments_instead_of_the_chain() {
    assert_eq!(
        format_with(OVERFLOWING_CHAIN, &style_with(true)),
        OVERFLOWING_CHAIN_PREFER_OUT
    );
}

#[test]
fn prefer_off_breaks_the_chain_and_keeps_the_tail_arguments_flat() {
    assert_eq!(
        format_with(OVERFLOWING_CHAIN, &style_with(false)),
        OVERFLOWING_CHAIN_OUT
    );
}

#[test]
fn absent_prefer_defaults_to_off() {
    // prefer_parameters_wrap defaults to false, so a style that never sets it
    // keeps the chain layout byte-identical to the explicit-false golden.
    let style = style(|s| {
        s.right_margin = 90;
        s.method_call_chain_wrap = WrapStyle::WrapIfLong;
        s.call_parameters_wrap = WrapStyle::WrapIfLong;
    });
    assert_eq!(
        format_with(OVERFLOWING_CHAIN, &style),
        OVERFLOWING_CHAIN_OUT
    );
}
