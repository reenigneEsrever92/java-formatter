//! MULTI_CATCH_TYPES_WRAP — wrapping of `catch (A | B e)` type lists.
//! Fixtures live under tests/java/multi_catch_types_wrap/.
//!
//! The type list of a multi-catch parameter wraps per the wrap code (`0` do
//! not wrap, `1` wrap if long, `2` wrap always, `5` chop down if long — codes
//! `1` and `5` share the per-type layout, exactly as the record-header and
//! clause-list wraps treat these atomic list elements). A wrapped list keeps
//! the first type on the `catch (` line and starts each following type on its
//! own line, the `|` operator leading the continuation line (the binary
//! operator-placement convention) at spaces to the first type's column
//! (`ALIGN_TYPES_IN_MULTI_CATCH`, default on). Single-type catches never wrap.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const CATCH_LIST: &str = include_str!("../java/multi_catch_types_wrap/catch_list.java");
const CATCH_LIST_DONOTWRAP_OUT: &str =
    include_str!("../java/multi_catch_types_wrap/catch_list_donotwrap.out.java");
const CATCH_LIST_DEFAULT_OUT: &str =
    include_str!("../java/multi_catch_types_wrap/catch_list_default.out.java");
const CATCH_LIST_WRAPIFLONG_OUT: &str =
    include_str!("../java/multi_catch_types_wrap/catch_list_wrapiflong.out.java");
const CATCH_LIST_WRAPALWAYS_OUT: &str =
    include_str!("../java/multi_catch_types_wrap/catch_list_wrapalways.out.java");
const WRAPPED: &str = include_str!("../java/multi_catch_types_wrap/wrapped.java");
const WRAPPED_OUT: &str = include_str!("../java/multi_catch_types_wrap/wrapped.out.java");

/// A style with a tight margin so the long multi-catch overflows.
fn narrow(wrap: WrapStyle) -> java_formatter_core::config::JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.multi_catch_types_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_keeps_the_type_list_on_one_line() {
    // Code `0`: an overflowing multi-catch stays flat, like the other
    // do-not-wrap defaults in the codebase.
    assert_eq!(
        format_with(CATCH_LIST, &narrow(WrapStyle::DoNotWrap)),
        CATCH_LIST_DONOTWRAP_OUT
    );
}

#[test]
fn absent_option_defaults_to_do_not_wrap() {
    // `MULTI_CATCH_TYPES_WRAP` ships `DoNotWrap` (a recorded divergence from
    // the docs-table default `1`, mirroring `RECORD_COMPONENTS_WRAP`), so the
    // default style keeps the single-line catch layout.
    assert_eq!(format(CATCH_LIST), CATCH_LIST_DEFAULT_OUT);
}

#[test]
fn wrap_if_long_wraps_only_the_overflowing_type_list() {
    // Code `1`: the over-margin multi-catch breaks one type per continuation
    // line with the `|` leading the line; the fitting short multi-catch and
    // the single-type catch stay flat.
    assert_eq!(
        format_with(CATCH_LIST, &narrow(WrapStyle::WrapIfLong)),
        CATCH_LIST_WRAPIFLONG_OUT
    );
}

#[test]
fn chop_down_if_long_uses_the_same_per_type_break() {
    // Code `5` shares code `1`'s layout here: the union members are atomic
    // (canonical type text, nothing to chop inside a member), so the goldens
    // record the identical output.
    assert_eq!(
        format_with(CATCH_LIST, &narrow(WrapStyle::ChopDownIfLong)),
        CATCH_LIST_WRAPIFLONG_OUT
    );
}

#[test]
fn wrap_always_breaks_every_multi_type_catch() {
    // Code `2`: even the short multi-catch that fits the margin breaks; a
    // single-type catch still cannot wrap and stays flat.
    assert_eq!(
        format_with(CATCH_LIST, &narrow(WrapStyle::WrapAlways)),
        CATCH_LIST_WRAPALWAYS_OUT
    );
}

#[test]
fn reformatting_the_wrapped_layout_is_a_no_op() {
    // A self-golden: the wrapped output formats to itself under the wrap
    // code that produced it (R6).
    assert_eq!(
        format_with(WRAPPED, &narrow(WrapStyle::WrapIfLong)),
        WRAPPED_OUT
    );
}
