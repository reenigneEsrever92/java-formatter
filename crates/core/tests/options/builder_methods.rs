//! BUILDER_METHODS — comma-separated method names treated as builder calls.
//!
//! A wrapped chain whose calls all match the list breaks after the receiver,
//! so every `.call()` — including the first — starts its own line at the
//! continuation indent (the generic `METHOD_CALL_CHAIN_WRAP` layout keeps the
//! first call on the receiver's line). Default / absent (empty list) never
//! takes the builder branch, and a chain that fits the margin stays flat.
//!
//! Fixtures live under tests/java/builder_methods/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const CHAIN: &str = include_str!("../java/builder_methods/chain.java");
const CHAIN_OUT: &str = include_str!("../java/builder_methods/chain.out.java");
const CHAIN_PLAIN_OUT: &str = include_str!("../java/builder_methods/chain_plain.out.java");
const CHAIN_FLAT_OUT: &str = include_str!("../java/builder_methods/chain_flat.out.java");

fn builder_style(right_margin: u32, wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = right_margin;
        s.method_call_chain_wrap = wrap;
        s.builder_methods = vec![
            "setName".to_string(),
            "setAge".to_string(),
            "setCity".to_string(),
            "setZip".to_string(),
            "build".to_string(),
        ];
    })
}

#[test]
fn builder_methods_break_after_the_receiver_when_overflowing() {
    // Every `.call()` of an overflowing builder chain goes on its own line,
    // including the first (statement level 2 × 4 + continuation 8 = 16).
    assert_eq!(
        format_with(CHAIN, &builder_style(40, WrapStyle::WrapIfLong)),
        CHAIN_OUT
    );
}

#[test]
fn builder_methods_absent_follows_the_plain_chain_wrap() {
    // Without the name list the same fixture wraps per METHOD_CALL_CHAIN_WRAP,
    // keeping the first call on the base line.
    let style = style(|s| {
        s.right_margin = 40;
        s.method_call_chain_wrap = WrapStyle::WrapIfLong;
    });
    assert_eq!(format_with(CHAIN, &style), CHAIN_PLAIN_OUT);
}

#[test]
fn builder_methods_keep_a_fitting_chain_flat() {
    // When the whole chain fits the margin, WrapIfLong must not break it even
    // with the builder list set.
    assert_eq!(
        format_with(CHAIN, &builder_style(120, WrapStyle::WrapIfLong)),
        CHAIN_FLAT_OUT
    );
}

#[test]
fn builder_methods_absent_default_keeps_the_chain_flat() {
    // The default scheme (empty list, DoNotWrap) formats the chain flat.
    assert_eq!(format(CHAIN), CHAIN_FLAT_OUT);
}

#[test]
fn builder_methods_golden_is_idempotent() {
    // Re-formatting the builder golden reproduces it byte-for-byte.
    assert_eq!(
        format_with(CHAIN_OUT, &builder_style(40, WrapStyle::WrapIfLong)),
        CHAIN_OUT
    );
}
