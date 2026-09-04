//! SPACE_WITHIN_SWITCH_PARENTHESES — padding inside `switch` conditions.
//! Fixtures live under tests/java/space_within_switch_parentheses/.

use super::common::*;

const SWITCH_COND: &str = include_str!("../java/space_within_switch_parentheses/switch_cond.java");
const SWITCH_COND_OUT: &str =
    include_str!("../java/space_within_switch_parentheses/switch_cond.out.java");
const SWITCH_COND_DEFAULT_OUT: &str =
    include_str!("../java/space_within_switch_parentheses/switch_cond_default.out.java");

#[test]
fn pads_switch_condition_when_on() {
    let s = style(|st| st.space_within_switch_parentheses = true);
    assert_eq!(format_with(SWITCH_COND, &s), SWITCH_COND_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(SWITCH_COND), SWITCH_COND_DEFAULT_OUT);
}
