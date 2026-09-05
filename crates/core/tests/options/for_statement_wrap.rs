//! FOR_STATEMENT_WRAP — wrapping of `for` headers (classic and enhanced).
//! Fixtures live under tests/java/for_statement_wrap/.
//!
//! Code `0` (DoNotWrap) keeps the source-verbatim header on one line, `1`
//! (WrapIfLong) breaks the classic header at its semicolons (each non-empty
//! slot on its own continuation line) and the enhanced header at its `:`
//! only when the flat header overflows, `2` (WrapAlways) always breaks, and
//! `5` (ChopDownIfLong) shares the wrap-if-long layout (the slots are
//! atomic verbatim parts). Paren placement is governed by the
//! FOR_STATEMENT_LPAREN/RPAREN_ON_NEXT_LINE options.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_FOR: &str = include_str!("../java/for_statement_wrap/long_for.java");
const LONG_FOR_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/for_statement_wrap/long_for_do_not_wrap.out.java");
const LONG_FOR_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/for_statement_wrap/long_for_wrap_if_long.out.java");
const LONG_FOR_CHOP_DOWN_OUT: &str =
    include_str!("../java/for_statement_wrap/long_for_chop_down.out.java");
const LONG_FOR_DEFAULT_OUT: &str =
    include_str!("../java/for_statement_wrap/long_for_default.out.java");
const SHORT_FOR: &str = include_str!("../java/for_statement_wrap/short_for.java");
const SHORT_FOR_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/for_statement_wrap/short_for_wrap_always.out.java");
const SELF_WRAPPED: &str = include_str!("../java/for_statement_wrap/wrapped.java");
const SELF_WRAPPED_OUT: &str = include_str!("../java/for_statement_wrap/wrapped.out.java");

/// A narrow margin so the long for headers overflow.
fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.for_statement_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_keeps_the_long_headers_on_one_line() {
    assert_eq!(
        format_with(LONG_FOR, &narrow(WrapStyle::DoNotWrap)),
        LONG_FOR_DO_NOT_WRAP_OUT
    );
}

#[test]
fn wrap_if_long_breaks_the_classic_header_at_its_semicolons() {
    assert_eq!(
        format_with(LONG_FOR, &narrow(WrapStyle::WrapIfLong)),
        LONG_FOR_WRAP_IF_LONG_OUT
    );
}

#[test]
fn wrap_always_breaks_a_header_that_would_fit() {
    assert_eq!(
        format_with(SHORT_FOR, &narrow(WrapStyle::WrapAlways)),
        SHORT_FOR_WRAP_ALWAYS_OUT
    );
}

#[test]
fn chop_down_uses_the_wrap_if_long_layout() {
    assert_eq!(
        format_with(LONG_FOR, &narrow(WrapStyle::ChopDownIfLong)),
        LONG_FOR_CHOP_DOWN_OUT
    );
}

#[test]
fn default_style_keeps_the_long_headers_on_one_line() {
    // for_statement_wrap defaults to DoNotWrap, so format() (no option set)
    // leaves the over-margin headers on one line.
    assert_eq!(format(LONG_FOR), LONG_FOR_DEFAULT_OUT);
}

#[test]
fn reformatting_wrapped_for_output_is_a_no_op() {
    // A self-golden: the fixture already matches the wrapped layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(SELF_WRAPPED, &narrow(WrapStyle::WrapIfLong)),
        SELF_WRAPPED_OUT
    );
}
