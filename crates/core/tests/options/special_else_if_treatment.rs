//! SPECIAL_ELSE_IF_TREATMENT — keeping an `else if` chain fused as one
//! construct instead of nesting `else { if … }`.
//!
//! With the option on (the IntelliJ default), an `else` whose body is another
//! `if` stays fused: `} else if (…) {`. With it off, the chain is rewritten
//! as an explicit `else { if … }` block per level — the braces group a single
//! `if`, so semantics are unchanged. Absent schemes fall back to the default
//! (fused).
//!
//! Fixtures live under tests/java/special_else_if_treatment/.

use super::common::*;

const ELSE_IF_CHAIN: &str = include_str!("../java/special_else_if_treatment/else_if_chain.java");
const ELSE_IF_CHAIN_FUSED_OUT: &str =
    include_str!("../java/special_else_if_treatment/else_if_chain_fused.out.java");
const ELSE_IF_CHAIN_NESTED_OUT: &str =
    include_str!("../java/special_else_if_treatment/else_if_chain_nested.out.java");
const ELSE_IF_CHAIN_DEFAULT_OUT: &str =
    include_str!("../java/special_else_if_treatment/else_if_chain_default.out.java");

#[test]
fn special_else_if_treatment_off_nests_each_else_if_in_an_else_block() {
    let style = style(|s| s.special_else_if_treatment = false);
    assert_eq!(format_with(ELSE_IF_CHAIN, &style), ELSE_IF_CHAIN_NESTED_OUT);
}

#[test]
fn special_else_if_treatment_on_keeps_the_chain_fused() {
    let style = style(|s| s.special_else_if_treatment = true);
    assert_eq!(format_with(ELSE_IF_CHAIN, &style), ELSE_IF_CHAIN_FUSED_OUT);
}

#[test]
fn absent_option_defaults_to_fused_else_if() {
    assert_eq!(format(ELSE_IF_CHAIN), ELSE_IF_CHAIN_DEFAULT_OUT);
}
