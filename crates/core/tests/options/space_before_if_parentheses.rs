//! SPACE_BEFORE_IF_PARENTHESES — space between `if` and its condition. Defaults to on.
//! Fixtures live under tests/java/space_before_if_parentheses/.

use super::common::*;

const IF_CHAIN: &str = include_str!("../java/space_before_if_parentheses/if_chain.java");
const IF_CHAIN_OUT: &str = include_str!("../java/space_before_if_parentheses/if_chain.out.java");
const IF_CHAIN_DEFAULT_OUT: &str =
    include_str!("../java/space_before_if_parentheses/if_chain_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_if_parentheses = false);
    assert_eq!(format_with(IF_CHAIN, &s), IF_CHAIN_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(IF_CHAIN), IF_CHAIN_DEFAULT_OUT);
}
