//! KEEP_BUILDER_METHODS_INDENTS — keep the indentation of a wrapped builder
//! chain instead of stepping the continuation indent.
//!
//! With `BUILDER_METHODS` naming the calls and the chain wrapping, `true`
//! puts every `.call()` line at the chain's own indentation, while `false`
//! (the default, and absent) steps them a continuation indent — the two
//! layouts differ only in the continuation-line indentation.
//!
//! Fixtures live under tests/java/keep_builder_methods_indents/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const CHAIN: &str = include_str!("../java/keep_builder_methods_indents/chain.java");
const CHAIN_KEEP_OUT: &str =
    include_str!("../java/keep_builder_methods_indents/chain_keep.out.java");
const CHAIN_STEP_OUT: &str =
    include_str!("../java/keep_builder_methods_indents/chain_step.out.java");

fn builder_style(keep: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.method_call_chain_wrap = WrapStyle::WrapIfLong;
        s.builder_methods = vec![
            "setName".to_string(),
            "setAge".to_string(),
            "setCity".to_string(),
            "setZip".to_string(),
            "build".to_string(),
        ];
        s.keep_builder_methods_indents = keep;
    })
}

#[test]
fn keep_builder_methods_indents_true_keeps_the_chain_indent() {
    // The `.call()` lines sit at the chain's own indent (2 × 4 = 8 spaces).
    assert_eq!(format_with(CHAIN, &builder_style(true)), CHAIN_KEEP_OUT);
}

#[test]
fn keep_builder_methods_indents_false_steps_the_continuation_indent() {
    // The `.call()` lines step one continuation indent (2 × 4 + 8 = 16).
    assert_eq!(format_with(CHAIN, &builder_style(false)), CHAIN_STEP_OUT);
}

#[test]
fn keep_builder_methods_indents_absent_steps_the_continuation_indent() {
    // Absent == the default `false`: the continuation indent is stepped.
    let style = style(|s| {
        s.right_margin = 40;
        s.method_call_chain_wrap = WrapStyle::WrapIfLong;
        s.builder_methods = vec![
            "setName".to_string(),
            "setAge".to_string(),
            "setCity".to_string(),
            "setZip".to_string(),
            "build".to_string(),
        ];
    });
    assert_eq!(format_with(CHAIN, &style), CHAIN_STEP_OUT);
}

#[test]
fn keep_builder_methods_indents_golden_is_idempotent() {
    // Re-formatting the keep-indents golden reproduces it byte-for-byte.
    assert_eq!(
        format_with(CHAIN_KEEP_OUT, &builder_style(true)),
        CHAIN_KEEP_OUT
    );
}
