//! RESOURCE_LIST_WRAP — wrapping of try-with-resources resource lists.
//! Fixtures live under tests/java/resource_list_wrap/.
//!
//! The resource elements are atomic and cannot be split further, so
//! WrapIfLong (1) and ChopDownIfLong (5) produce the same layout — the
//! chop-down golden equals the wrap-if-long golden. Paren placement of the
//! wrapped list is governed by RESOURCE_LIST_LPAREN_ON_NEXT_LINE /
//! RESOURCE_LIST_RPAREN_ON_NEXT_LINE (their own option files).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_RESOURCES: &str = include_str!("../java/resource_list_wrap/long_resources.java");
const LONG_RESOURCES_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/resource_list_wrap/long_resources_do_not_wrap.out.java");
const LONG_RESOURCES_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/resource_list_wrap/long_resources_wrap_if_long.out.java");
const LONG_RESOURCES_CHOP_DOWN_OUT: &str =
    include_str!("../java/resource_list_wrap/long_resources_chop_down.out.java");
const SHORT_RESOURCES: &str = include_str!("../java/resource_list_wrap/short_resources.java");
const SHORT_RESOURCES_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/resource_list_wrap/short_resources_wrap_always.out.java");
const MESSY_RESOURCES: &str = include_str!("../java/resource_list_wrap/messy_resources.java");
const MESSY_RESOURCES_DEFAULT_OUT: &str =
    include_str!("../java/resource_list_wrap/messy_resources_default.out.java");
const SELF_WRAPPED: &str = include_str!("../java/resource_list_wrap/self_wrapped.java");
const SELF_WRAPPED_OUT: &str = include_str!("../java/resource_list_wrap/self_wrapped.out.java");

/// A narrow margin so the long fixture's resource list overflows.
fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.resource_list_wrap = wrap;
    })
}

#[test]
fn do_not_wrap_keeps_the_long_resource_list_on_one_line() {
    assert_eq!(
        format_with(LONG_RESOURCES, &narrow(WrapStyle::DoNotWrap)),
        LONG_RESOURCES_DO_NOT_WRAP_OUT
    );
}

#[test]
fn wrap_if_long_breaks_the_list_one_resource_per_line() {
    assert_eq!(
        format_with(LONG_RESOURCES, &narrow(WrapStyle::WrapIfLong)),
        LONG_RESOURCES_WRAP_IF_LONG_OUT
    );
}

#[test]
fn chop_down_uses_the_same_layout_for_atomic_resource_elements() {
    assert_eq!(
        format_with(LONG_RESOURCES, &narrow(WrapStyle::ChopDownIfLong)),
        LONG_RESOURCES_CHOP_DOWN_OUT
    );
}

#[test]
fn wrap_always_breaks_a_list_that_would_fit() {
    assert_eq!(
        format_with(SHORT_RESOURCES, &narrow(WrapStyle::WrapAlways)),
        SHORT_RESOURCES_WRAP_ALWAYS_OUT
    );
}

#[test]
fn default_style_preserves_the_resource_list_verbatim() {
    // resource_list_wrap defaults to DoNotWrap: format() (no option set)
    // leaves the messy single-line resource spec exactly as written — no
    // interior whitespace is touched (R4).
    assert_eq!(format(MESSY_RESOURCES), MESSY_RESOURCES_DEFAULT_OUT);
}

#[test]
fn reformatting_wrapped_resource_output_is_a_no_op() {
    // A self-golden: the fixture already matches the wrapped layout, so
    // formatting it under the same style is byte-identical (R6).
    assert_eq!(
        format_with(SELF_WRAPPED, &narrow(WrapStyle::WrapIfLong)),
        SELF_WRAPPED_OUT
    );
}
