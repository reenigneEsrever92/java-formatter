//! SPACE_WITHIN_IF_PARENTHESES — padding inside `if` / `else if` conditions.
//! Fixtures live under tests/java/space_within_if_parentheses/.

use super::common::*;

const IF_COND: &str = include_str!("../java/space_within_if_parentheses/if_cond.java");
const IF_COND_OUT: &str = include_str!("../java/space_within_if_parentheses/if_cond.out.java");
const IF_COND_DEFAULT_OUT: &str =
    include_str!("../java/space_within_if_parentheses/if_cond_default.out.java");

#[test]
fn pads_if_condition_when_on() {
    let s = style(|st| st.space_within_if_parentheses = true);
    assert_eq!(format_with(IF_COND, &s), IF_COND_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(IF_COND), IF_COND_DEFAULT_OUT);
}
