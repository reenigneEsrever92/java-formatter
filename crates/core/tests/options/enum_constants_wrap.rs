//! ENUM_CONSTANTS_WRAP — wrapping of enum constant lists.
//! Fixtures live under tests/java/enum_constants_wrap/.
//!
//! A constant-only body (no `;` declarations section) collapses to the flat
//! `{A, B}` form: always under `DoNotWrap` (the absent-option default — a
//! list that overflows stays on one line), one constant per line under
//! `WrapAlways`, and flat iff the flat declaration fits the margin under
//! `WrapIfLong` / `ChopDownIfLong` (5 renders like 1 here — constants are
//! echoed verbatim, so there is no in-constant chopping).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_CONSTANTS: &str = include_str!("../java/enum_constants_wrap/long_constants.java");
const LONG_CONSTANTS_0_OUT: &str =
    include_str!("../java/enum_constants_wrap/long_constants_0.out.java");
const LONG_CONSTANTS_1_OUT: &str =
    include_str!("../java/enum_constants_wrap/long_constants_1.out.java");
const LONG_CONSTANTS_2_OUT: &str =
    include_str!("../java/enum_constants_wrap/long_constants_2.out.java");
const LONG_CONSTANTS_5_OUT: &str =
    include_str!("../java/enum_constants_wrap/long_constants_5.out.java");
const LONG_CONSTANTS_DEFAULT_OUT: &str =
    include_str!("../java/enum_constants_wrap/long_constants_default.out.java");
const SHORT_ENUM: &str = include_str!("../java/enum_constants_wrap/short_enum.java");
const SHORT_ENUM_DEFAULT_OUT: &str =
    include_str!("../java/enum_constants_wrap/short_enum_default.out.java");
const SHORT_ENUM_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/enum_constants_wrap/short_enum_wrap_always.out.java");

/// A tight margin so `Season`'s one-line constant list overflows.
fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.enum_constants_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_keeps_the_overflowing_list_on_one_line() {
    assert_eq!(
        format_with(LONG_CONSTANTS, &narrow(WrapStyle::DoNotWrap)),
        LONG_CONSTANTS_0_OUT
    );
}

#[test]
fn wrap_if_long_breaks_an_overflowing_list_one_constant_per_line() {
    assert_eq!(
        format_with(LONG_CONSTANTS, &narrow(WrapStyle::WrapIfLong)),
        LONG_CONSTANTS_1_OUT
    );
}

#[test]
fn chop_down_if_long_breaks_the_long_list_like_wrap_if_long() {
    assert_eq!(
        format_with(LONG_CONSTANTS, &narrow(WrapStyle::ChopDownIfLong)),
        LONG_CONSTANTS_5_OUT
    );
}

#[test]
fn wrap_always_breaks_an_overflowing_list_one_constant_per_line() {
    assert_eq!(
        format_with(LONG_CONSTANTS, &narrow(WrapStyle::WrapAlways)),
        LONG_CONSTANTS_2_OUT
    );
}

#[test]
fn absent_option_defaults_to_do_not_wrap() {
    assert_eq!(format(LONG_CONSTANTS), LONG_CONSTANTS_DEFAULT_OUT);
}

#[test]
fn wrap_if_long_keeps_a_fitting_enum_on_one_line() {
    // Distinguishes code 1 (flat when it fits) from code 2 (breaks anyway).
    assert_eq!(
        format_with(SHORT_ENUM, &narrow(WrapStyle::WrapIfLong)),
        SHORT_ENUM_DEFAULT_OUT
    );
}

#[test]
fn absent_option_keeps_a_fitting_enum_on_one_line() {
    assert_eq!(format(SHORT_ENUM), SHORT_ENUM_DEFAULT_OUT);
}

#[test]
fn wrap_always_breaks_a_fitting_enum_one_constant_per_line() {
    assert_eq!(
        format_with(SHORT_ENUM, &narrow(WrapStyle::WrapAlways)),
        SHORT_ENUM_WRAP_ALWAYS_OUT
    );
}

#[test]
fn reformatting_the_wrap_if_long_output_is_a_no_op() {
    let s = narrow(WrapStyle::WrapIfLong);
    assert_eq!(format_with(LONG_CONSTANTS_1_OUT, &s), LONG_CONSTANTS_1_OUT);
}
