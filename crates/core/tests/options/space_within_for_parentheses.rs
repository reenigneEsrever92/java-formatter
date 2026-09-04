//! SPACE_WITHIN_FOR_PARENTHESES — padding inside classic and enhanced `for`
//! headers.
//! Fixtures live under tests/java/space_within_for_parentheses/.

use super::common::*;

const FOR_COND: &str = include_str!("../java/space_within_for_parentheses/for_cond.java");
const FOR_COND_OUT: &str = include_str!("../java/space_within_for_parentheses/for_cond.out.java");
const FOR_COND_DEFAULT_OUT: &str =
    include_str!("../java/space_within_for_parentheses/for_cond_default.out.java");

#[test]
fn pads_for_headers_when_on() {
    let s = style(|st| st.space_within_for_parentheses = true);
    assert_eq!(format_with(FOR_COND, &s), FOR_COND_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(FOR_COND), FOR_COND_DEFAULT_OUT);
}
