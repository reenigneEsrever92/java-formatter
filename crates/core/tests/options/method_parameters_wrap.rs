//! METHOD_PARAMETERS_WRAP — wrapping of method / constructor parameter lists.
//! Fixtures live under tests/java/method_parameters_wrap/.

use super::common::*;
use java_formatter_core::config::{BraceStyle, JavaStyle, WrapStyle};

const WRAPPED_PARAMS: &str = include_str!("../java/method_parameters_wrap/wrapped_params.java");
const WRAPPED_PARAMS_OUT: &str =
    include_str!("../java/method_parameters_wrap/wrapped_params.out.java");
const PARAMS_FIT_THROWS_OVERFLOWS: &str =
    include_str!("../java/method_parameters_wrap/params_fit_throws_overflows.java");
const PARAMS_FIT_THROWS_OVERFLOWS_OUT: &str =
    include_str!("../java/method_parameters_wrap/params_fit_throws_overflows.out.java");
const PARAMS_FIT_THROWS_OVERFLOWS_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/method_parameters_wrap/params_fit_throws_overflows_do_not_wrap.out.java");
const BRACE_OVERFLOW: &str = include_str!("../java/method_parameters_wrap/brace_overflow.java");
const BRACE_OVERFLOW_OUT: &str =
    include_str!("../java/method_parameters_wrap/brace_overflow.out.java");
const BRACE_OVERFLOW_NEXT_LINE_BRACE_OUT: &str =
    include_str!("../java/method_parameters_wrap/brace_overflow_next_line_brace.out.java");
const HEADER_FITS: &str = include_str!("../java/method_parameters_wrap/header_fits.java");
const HEADER_FITS_OUT: &str = include_str!("../java/method_parameters_wrap/header_fits.out.java");
const ALREADY_WRAPPED: &str = include_str!("../java/method_parameters_wrap/already_wrapped.java");
const ALREADY_WRAPPED_OUT: &str =
    include_str!("../java/method_parameters_wrap/already_wrapped.out.java");

/// `ChopDownIfLong` with the parens on their own lines, at the given margin.
fn chop_down(margin: u32) -> JavaStyle {
    style(|s| {
        s.right_margin = margin;
        s.method_parameters_wrap = WrapStyle::ChopDownIfLong;
        s.method_parameters_lparen_on_next_line = true;
        s.method_parameters_rparen_on_next_line = true;
    })
}

#[test]
fn chop_down_if_long_wraps_parameter_lists() {
    // The declaration overflows the margin, so ChopDownIfLong puts each
    // parameter on its own line (the throws clause in the fixture is
    // incidental to this option).
    assert_eq!(
        format_with(WRAPPED_PARAMS, &chop_down(60)),
        WRAPPED_PARAMS_OUT
    );
}

#[test]
fn a_header_that_overflows_only_after_the_rparen_wraps() {
    // The parameter list ends within the margin; the single `throws` clause
    // pushes the header over it, so the list wraps (R50).
    assert_eq!(
        format_with(PARAMS_FIT_THROWS_OVERFLOWS, &chop_down(60)),
        PARAMS_FIT_THROWS_OVERFLOWS_OUT
    );
}

#[test]
fn the_body_brace_alone_can_push_a_header_over_the_margin() {
    // The parameter list ends exactly at the margin; only the trailing ` {`
    // overflows, and the list still wraps (R50).
    assert_eq!(
        format_with(BRACE_OVERFLOW, &chop_down(85)),
        BRACE_OVERFLOW_OUT
    );
}

#[test]
fn a_next_line_brace_does_not_count_towards_the_header() {
    // METHOD_BRACE_STYLE = NextLine moves the brace onto its own line, so the
    // header is within the margin and the parameter list stays flat (R50).
    let style = style(|s| {
        s.right_margin = 85;
        s.method_parameters_wrap = WrapStyle::ChopDownIfLong;
        s.method_parameters_lparen_on_next_line = true;
        s.method_parameters_rparen_on_next_line = true;
        s.method_brace_style = BraceStyle::NextLine;
    });
    assert_eq!(
        format_with(BRACE_OVERFLOW, &style),
        BRACE_OVERFLOW_NEXT_LINE_BRACE_OUT
    );
}

#[test]
fn a_header_that_fits_is_not_wrapped() {
    // The whole header is within the margin, so nothing wraps (R50).
    assert_eq!(format_with(HEADER_FITS, &chop_down(60)), HEADER_FITS_OUT);
}

#[test]
fn do_not_wrap_keeps_an_over_margin_header() {
    // The option is absent (the built-in DoNotWrap default): the over-margin
    // header is preserved byte-for-byte (R50).
    let style = style(|s| {
        s.right_margin = 60;
        s.method_parameters_lparen_on_next_line = true;
        s.method_parameters_rparen_on_next_line = true;
    });
    assert_eq!(
        format_with(PARAMS_FIT_THROWS_OVERFLOWS, &style),
        PARAMS_FIT_THROWS_OVERFLOWS_DO_NOT_WRAP_OUT
    );
}

#[test]
fn reformatting_a_wrapped_header_is_a_no_op() {
    // A self-golden: the fixture already matches the wrapped layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(ALREADY_WRAPPED, &chop_down(60)),
        ALREADY_WRAPPED_OUT
    );
}
