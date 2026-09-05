//! THROWS_LIST_WRAP — wrapping of method / constructor `throws` lists.
//! Fixtures live under tests/java/throws_list_wrap/.
//!
//! Throws-list elements are atomic types that cannot be split further, so
//! WrapIfLong (1) and ChopDownIfLong (5) produce the same layout — the
//! chop-down golden equals the wrap-if-long golden. Keyword placement is
//! governed by THROWS_KEYWORD_WRAP (its own option file).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_THROWS: &str = include_str!("../java/throws_list_wrap/long_throws.java");
const LONG_THROWS_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/throws_list_wrap/long_throws_do_not_wrap.out.java");
const LONG_THROWS_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/throws_list_wrap/long_throws_wrap_if_long.out.java");
const LONG_THROWS_CHOP_DOWN_OUT: &str =
    include_str!("../java/throws_list_wrap/long_throws_chop_down.out.java");
const LONG_THROWS_DEFAULT_OUT: &str =
    include_str!("../java/throws_list_wrap/long_throws_default.out.java");
const SHORT_THROWS: &str = include_str!("../java/throws_list_wrap/short_throws.java");
const SHORT_THROWS_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/throws_list_wrap/short_throws_wrap_always.out.java");
const SELF_WRAPPED: &str = include_str!("../java/throws_list_wrap/self_wrapped.java");
const SELF_WRAPPED_OUT: &str = include_str!("../java/throws_list_wrap/self_wrapped.out.java");

/// A narrow margin so the long fixture's throws clause overflows.
fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.throws_list_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_keeps_the_long_throws_clause_on_one_line() {
    assert_eq!(
        format_with(LONG_THROWS, &narrow(WrapStyle::DoNotWrap)),
        LONG_THROWS_DO_NOT_WRAP_OUT
    );
}

#[test]
fn wrap_if_long_breaks_the_clause_one_exception_per_line() {
    assert_eq!(
        format_with(LONG_THROWS, &narrow(WrapStyle::WrapIfLong)),
        LONG_THROWS_WRAP_IF_LONG_OUT
    );
}

#[test]
fn chop_down_uses_the_same_layout_for_atomic_throws_elements() {
    assert_eq!(
        format_with(LONG_THROWS, &narrow(WrapStyle::ChopDownIfLong)),
        LONG_THROWS_CHOP_DOWN_OUT
    );
}

#[test]
fn wrap_always_breaks_a_clause_that_would_fit() {
    assert_eq!(
        format_with(SHORT_THROWS, &narrow(WrapStyle::WrapAlways)),
        SHORT_THROWS_WRAP_ALWAYS_OUT
    );
}

#[test]
fn default_style_keeps_the_long_clause_on_one_line() {
    // throws_list_wrap defaults to DoNotWrap, so format() (no option set)
    // leaves the over-margin clause on the header line.
    assert_eq!(format(LONG_THROWS), LONG_THROWS_DEFAULT_OUT);
}

#[test]
fn reformatting_wrapped_throws_output_is_a_no_op() {
    // A self-golden: the fixture already matches the wrapped layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(SELF_WRAPPED, &narrow(WrapStyle::WrapIfLong)),
        SELF_WRAPPED_OUT
    );
}
