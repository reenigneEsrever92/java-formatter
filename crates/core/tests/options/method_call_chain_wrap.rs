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
const CHAIN_SHORT_OUT: &str =
    include_str!("../java/method_call_chain_wrap/chain_short.out.java");

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
