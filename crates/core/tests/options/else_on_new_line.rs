//! ELSE_ON_NEW_LINE — putting the `else` keyword of an if / else-if chain on
//! its own line.
//!
//! With the option on (IntelliJ default off), the `else` keyword that follows
//! a closing `}` starts a fresh line at the statement indent instead of
//! joining the brace line. Absent schemes fall back to the default, keeping
//! the keyword on the brace line.
//!
//! Fixtures live under tests/java/else_on_new_line/.

use super::common::*;

const ELSE_CHAIN: &str = include_str!("../java/else_on_new_line/else_chain.java");
const ELSE_CHAIN_ON_OUT: &str = include_str!("../java/else_on_new_line/else_chain_on.out.java");
const ELSE_CHAIN_DEFAULT_OUT: &str =
    include_str!("../java/else_on_new_line/else_chain_default.out.java");

#[test]
fn else_on_new_line_starts_each_else_keyword_on_its_own_line() {
    let style = style(|s| s.else_on_new_line = true);
    assert_eq!(format_with(ELSE_CHAIN, &style), ELSE_CHAIN_ON_OUT);
}

#[test]
fn absent_option_defaults_to_inline_else_keyword() {
    assert_eq!(format(ELSE_CHAIN), ELSE_CHAIN_DEFAULT_OUT);
}
