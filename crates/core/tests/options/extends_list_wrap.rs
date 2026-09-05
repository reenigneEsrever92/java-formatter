//! EXTENDS_LIST_WRAP — wrapping of `extends` / `implements` lists in type
//! declarations (class / interface / enum / record headers). A class's single
//! `extends Base` supertype is not a list and never wraps.
//! Fixtures live under tests/java/extends_list_wrap/.
//!
//! The list elements are atomic types that cannot be split further, so
//! WrapIfLong (1) and ChopDownIfLong (5) produce the same layout — the
//! chop-down golden equals the wrap-if-long golden. Keyword placement is
//! governed by EXTENDS_KEYWORD_WRAP (its own option file).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_HEADERS: &str = include_str!("../java/extends_list_wrap/long_headers.java");
const LONG_HEADERS_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/extends_list_wrap/long_headers_do_not_wrap.out.java");
const LONG_HEADERS_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/extends_list_wrap/long_headers_wrap_if_long.out.java");
const LONG_HEADERS_CHOP_DOWN_OUT: &str =
    include_str!("../java/extends_list_wrap/long_headers_chop_down.out.java");
const LONG_HEADERS_DEFAULT_OUT: &str =
    include_str!("../java/extends_list_wrap/long_headers_default.out.java");
const SHORT_HEADERS: &str = include_str!("../java/extends_list_wrap/short_headers.java");
const SHORT_HEADERS_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/extends_list_wrap/short_headers_wrap_always.out.java");
const SELF_WRAPPED: &str = include_str!("../java/extends_list_wrap/self_wrapped.java");
const SELF_WRAPPED_OUT: &str = include_str!("../java/extends_list_wrap/self_wrapped.out.java");

/// A narrow margin so the long fixture's extends / implements lists overflow.
fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.extends_list_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_keeps_the_long_clauses_on_one_line() {
    assert_eq!(
        format_with(LONG_HEADERS, &narrow(WrapStyle::DoNotWrap)),
        LONG_HEADERS_DO_NOT_WRAP_OUT
    );
}

#[test]
fn wrap_if_long_breaks_class_and_interface_lists_one_type_per_line() {
    assert_eq!(
        format_with(LONG_HEADERS, &narrow(WrapStyle::WrapIfLong)),
        LONG_HEADERS_WRAP_IF_LONG_OUT
    );
}

#[test]
fn chop_down_uses_the_same_layout_for_atomic_list_types() {
    assert_eq!(
        format_with(LONG_HEADERS, &narrow(WrapStyle::ChopDownIfLong)),
        LONG_HEADERS_CHOP_DOWN_OUT
    );
}

#[test]
fn wrap_always_breaks_lists_that_would_fit() {
    // The short fixture's headers all fit at margin 60; wrap-always still
    // breaks each class / interface / enum / record list one type per line.
    assert_eq!(
        format_with(SHORT_HEADERS, &narrow(WrapStyle::WrapAlways)),
        SHORT_HEADERS_WRAP_ALWAYS_OUT
    );
}

#[test]
fn default_style_keeps_the_long_clauses_on_one_line() {
    // extends_list_wrap defaults to DoNotWrap, so format() (no option set)
    // leaves the over-margin clauses on their header lines.
    assert_eq!(format(LONG_HEADERS), LONG_HEADERS_DEFAULT_OUT);
}

#[test]
fn reformatting_wrapped_extends_output_is_a_no_op() {
    // A self-golden: the fixture already matches the wrapped layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(SELF_WRAPPED, &narrow(WrapStyle::WrapIfLong)),
        SELF_WRAPPED_OUT
    );
}
