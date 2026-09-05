//! ARRAY_INITIALIZER_WRAP — wrapping of array initializer lists.
//! Fixtures live under tests/java/array_initializer_wrap/.
//!
//! Code `0` (DoNotWrap) keeps the flat form even when long, `1`
//! (WrapIfLong) breaks the initializer one element per line only when it
//! overflows, `2` (WrapAlways) always breaks, and `5` (ChopDownIfLong)
//! shares the wrap-if-long layout. Brace placement is governed by the
//! ARRAY_INITIALIZER_LBRACE/RBRACE_ON_NEXT_LINE options; the default keeps
//! both braces at the end of their lines.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const INIT: &str = include_str!("../java/array_initializer_wrap/init.java");
const INIT_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/array_initializer_wrap/init_do_not_wrap.out.java");
const INIT_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/array_initializer_wrap/init_wrap_if_long.out.java");
const INIT_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/array_initializer_wrap/init_wrap_always.out.java");
const INIT_CHOP_DOWN_OUT: &str =
    include_str!("../java/array_initializer_wrap/init_chop_down.out.java");
const INIT_DEFAULT_OUT: &str = include_str!("../java/array_initializer_wrap/init_default.out.java");
const SELF_WRAPPED: &str = include_str!("../java/array_initializer_wrap/wrapped.java");
const SELF_WRAPPED_OUT: &str = include_str!("../java/array_initializer_wrap/wrapped.out.java");

/// A narrow margin so the long initializer overflows.
fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 50;
        s.array_initializer_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_keeps_the_long_initializer_on_one_line() {
    assert_eq!(
        format_with(INIT, &narrow(WrapStyle::DoNotWrap)),
        INIT_DO_NOT_WRAP_OUT
    );
}

#[test]
fn wrap_if_long_breaks_the_initializer_one_element_per_line() {
    assert_eq!(
        format_with(INIT, &narrow(WrapStyle::WrapIfLong)),
        INIT_WRAP_IF_LONG_OUT
    );
}

#[test]
fn wrap_always_breaks_an_initializer_that_would_fit() {
    assert_eq!(
        format_with(INIT, &narrow(WrapStyle::WrapAlways)),
        INIT_WRAP_ALWAYS_OUT
    );
}

#[test]
fn chop_down_uses_the_wrap_if_long_layout() {
    assert_eq!(
        format_with(INIT, &narrow(WrapStyle::ChopDownIfLong)),
        INIT_CHOP_DOWN_OUT
    );
}

#[test]
fn default_style_keeps_the_long_initializer_on_one_line() {
    // array_initializer_wrap defaults to DoNotWrap, so format() (no option
    // set) leaves the over-margin initializer on one line.
    assert_eq!(format(INIT), INIT_DEFAULT_OUT);
}

#[test]
fn reformatting_wrapped_initializer_output_is_a_no_op() {
    // A self-golden: the fixture already matches the wrapped layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(SELF_WRAPPED, &narrow(WrapStyle::WrapIfLong)),
        SELF_WRAPPED_OUT
    );
}
