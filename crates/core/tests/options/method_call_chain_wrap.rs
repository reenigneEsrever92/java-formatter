//! METHOD_CALL_CHAIN_WRAP — wrapping of chained method calls.
//!
//! When a chained invocation overflows the right margin, `WrapIfLong` /
//! `WrapAlways` break the chain into one link per line at the continuation
//! indent; `DoNotWrap` (the default) keeps the whole chain flat.
//!
//! Fixtures live under tests/java/method_call_chain_wrap/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const CHAIN: &str = include_str!("../java/method_call_chain_wrap/chain.java");
const CHAIN_OUT: &str = include_str!("../java/method_call_chain_wrap/chain.out.java");
const CHAIN_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/method_call_chain_wrap/chain_do_not_wrap.out.java");
const CHAIN_SHORT_OUT: &str = include_str!("../java/method_call_chain_wrap/chain_short.out.java");
const NESTED_IN_LINK: &str = include_str!("../java/method_call_chain_wrap/nested_in_link.java");
const NESTED_IN_LINK_OUT: &str =
    include_str!("../java/method_call_chain_wrap/nested_in_link.out.java");
const NESTED_IN_LINK_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/method_call_chain_wrap/nested_in_link_do_not_wrap.out.java");
const NESTED_LIST_OF: &str = include_str!("../java/method_call_chain_wrap/nested_list_of.java");
const NESTED_LIST_OF_OUT: &str =
    include_str!("../java/method_call_chain_wrap/nested_list_of.out.java");
const CHAIN_ARGUMENT: &str = include_str!("../java/method_call_chain_wrap/chain_argument.java");
const CHAIN_ARGUMENT_OUT: &str =
    include_str!("../java/method_call_chain_wrap/chain_argument.out.java");

fn wrap_style(right_margin: u32, wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = right_margin;
        s.method_call_chain_wrap = wrap;
    })
}

#[test]
fn default_keeps_long_chain_flat() {
    // The chain overflows the tight margin but DoNotWrap (the default) must
    // leave it on a single line.
    assert_eq!(
        format_with(CHAIN, &wrap_style(40, WrapStyle::DoNotWrap)),
        CHAIN_DO_NOT_WRAP_OUT
    );
}

#[test]
fn wrap_if_long_breaks_long_chain_per_link() {
    // Each link after the first goes on its own line at the continuation
    // indent (statement level 2 × 4 + continuation 8 = 16 spaces).
    let style = wrap_style(40, WrapStyle::WrapIfLong);
    assert_eq!(format_with(CHAIN, &style), CHAIN_OUT);
}

#[test]
fn wrap_if_long_keeps_short_chains_flat() {
    // When the whole chain fits, WrapIfLong must not break it.
    assert_eq!(
        format_with(CHAIN, &wrap_style(120, WrapStyle::WrapIfLong)),
        CHAIN_SHORT_OUT
    );
}

#[test]
fn wrap_if_long_wraps_a_chain_nested_in_a_link_argument() {
    // The outer chain breaks per link and the chain used as a link's argument
    // breaks too, one indent deeper.
    let style = wrap_style(60, WrapStyle::WrapIfLong);
    assert_eq!(format_with(NESTED_IN_LINK, &style), NESTED_IN_LINK_OUT);
}

#[test]
fn wrap_if_long_wraps_chains_inside_a_multi_element_argument() {
    // `List.of(<builder chain>, <builder chain>)` is a single argument whose
    // own argument list holds two chains; both wrap in place.
    let style = wrap_style(60, WrapStyle::WrapIfLong);
    assert_eq!(format_with(NESTED_LIST_OF, &style), NESTED_LIST_OF_OUT);
}

#[test]
fn wrap_if_long_wraps_a_chain_argument_with_call_parameters_wrap_off() {
    // A multi-argument call whose argument is a long chain: the chain wraps
    // (CALL_PARAMETERS_WRAP is DoNotWrap) instead of the argument list.
    let style = wrap_style(60, WrapStyle::WrapIfLong);
    assert_eq!(format_with(CHAIN_ARGUMENT, &style), CHAIN_ARGUMENT_OUT);
}

#[test]
fn do_not_wrap_keeps_nested_chains_flat() {
    // DoNotWrap (and the absent default) leaves both the outer and the nested
    // chain on one line, over the margin.
    let style = wrap_style(60, WrapStyle::DoNotWrap);
    assert_eq!(
        format_with(NESTED_IN_LINK, &style),
        NESTED_IN_LINK_DO_NOT_WRAP_OUT
    );
}

#[test]
fn reformatting_nested_chain_output_is_a_no_op() {
    let style = wrap_style(60, WrapStyle::WrapIfLong);
    for golden in [NESTED_IN_LINK_OUT, NESTED_LIST_OF_OUT, CHAIN_ARGUMENT_OUT] {
        assert_eq!(format_with(golden, &style), golden);
    }
}
