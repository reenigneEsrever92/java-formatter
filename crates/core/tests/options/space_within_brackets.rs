//! SPACE_WITHIN_BRACKETS — padding inside `[ expr ]` array-access brackets.
//! Fixtures live under tests/java/space_within_brackets/.

use super::common::*;

const BRACKETS: &str = include_str!("../java/space_within_brackets/brackets.java");
const BRACKETS_OUT: &str = include_str!("../java/space_within_brackets/brackets.out.java");
const BRACKETS_DEFAULT_OUT: &str =
    include_str!("../java/space_within_brackets/brackets_default.out.java");

#[test]
fn pads_array_access_brackets_when_on() {
    let s = style(|st| st.space_within_brackets = true);
    assert_eq!(format_with(BRACKETS, &s), BRACKETS_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(BRACKETS), BRACKETS_DEFAULT_OUT);
}
