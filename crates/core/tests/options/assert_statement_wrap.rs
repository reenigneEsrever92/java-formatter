//! ASSERT_STATEMENT_WRAP — wrapping of `assert` statements.
//! Fixtures live under tests/java/assert_statement_wrap/.
//!
//! Code `0` (DoNotWrap) keeps the one-line form even when long, `1`
//! (WrapIfLong) wraps at the expression / `:` only when the flat statement
//! overflows, `2` (WrapAlways) always wraps, and `5` (ChopDownIfLong)
//! additionally lets an overflowing expression side wrap internally. Colon
//! placement is governed by ASSERT_STATEMENT_COLON_ON_NEXT_LINE (its own
//! option file); the default operator-end layout is asserted here.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const ASSERTS: &str = include_str!("../java/assert_statement_wrap/asserts.java");
const ASSERTS_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/assert_statement_wrap/asserts_do_not_wrap.out.java");
const ASSERTS_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/assert_statement_wrap/asserts_wrap_if_long.out.java");
const ASSERTS_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/assert_statement_wrap/asserts_wrap_always.out.java");
const ASSERTS_CHOP_DOWN_OUT: &str =
    include_str!("../java/assert_statement_wrap/asserts_chop_down.out.java");
const ASSERTS_DEFAULT_OUT: &str =
    include_str!("../java/assert_statement_wrap/asserts_default.out.java");
const SELF_WRAPPED: &str = include_str!("../java/assert_statement_wrap/wrapped.java");
const SELF_WRAPPED_OUT: &str = include_str!("../java/assert_statement_wrap/wrapped.out.java");

/// A narrow margin so the long assert statements overflow.
fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.assert_statement_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_keeps_the_long_assert_on_one_line() {
    assert_eq!(
        format_with(ASSERTS, &narrow(WrapStyle::DoNotWrap)),
        ASSERTS_DO_NOT_WRAP_OUT
    );
}

#[test]
fn wrap_if_long_breaks_the_assert_at_its_colon() {
    assert_eq!(
        format_with(ASSERTS, &narrow(WrapStyle::WrapIfLong)),
        ASSERTS_WRAP_IF_LONG_OUT
    );
}

#[test]
fn wrap_always_breaks_an_assert_that_would_fit() {
    // The short single-part assert wraps at its expression too.
    assert_eq!(
        format_with(ASSERTS, &narrow(WrapStyle::WrapAlways)),
        ASSERTS_WRAP_ALWAYS_OUT
    );
}

#[test]
fn chop_down_keeps_fitting_asserts_flat() {
    assert_eq!(
        format_with(ASSERTS, &narrow(WrapStyle::ChopDownIfLong)),
        ASSERTS_CHOP_DOWN_OUT
    );
}

#[test]
fn default_style_keeps_the_long_assert_on_one_line() {
    // assert_statement_wrap defaults to DoNotWrap, so format() (no option
    // set) leaves the over-margin asserts on one line.
    assert_eq!(format(ASSERTS), ASSERTS_DEFAULT_OUT);
}

#[test]
fn reformatting_wrapped_assert_output_is_a_no_op() {
    // A self-golden: the fixture already matches the wrapped layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(SELF_WRAPPED, &narrow(WrapStyle::WrapIfLong)),
        SELF_WRAPPED_OUT
    );
}
