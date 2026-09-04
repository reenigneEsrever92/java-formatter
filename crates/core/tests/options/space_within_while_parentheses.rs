//! SPACE_WITHIN_WHILE_PARENTHESES — padding inside `while` conditions and the
//! trailing `while` of a `do … while`.
//! Fixtures live under tests/java/space_within_while_parentheses/.

use super::common::*;

const WHILE_COND: &str = include_str!("../java/space_within_while_parentheses/while_cond.java");
const WHILE_COND_OUT: &str =
    include_str!("../java/space_within_while_parentheses/while_cond.out.java");
const WHILE_COND_DEFAULT_OUT: &str =
    include_str!("../java/space_within_while_parentheses/while_cond_default.out.java");

#[test]
fn pads_while_conditions_when_on() {
    let s = style(|st| st.space_within_while_parentheses = true);
    assert_eq!(format_with(WHILE_COND, &s), WHILE_COND_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(WHILE_COND), WHILE_COND_DEFAULT_OUT);
}
