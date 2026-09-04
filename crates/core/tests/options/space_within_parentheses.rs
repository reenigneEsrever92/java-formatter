//! SPACE_WITHIN_PARENTHESES — padding inside plain parentheses.
//! Fixtures live under tests/java/space_within_parentheses/.

use super::common::*;

const PARENS: &str = include_str!("../java/space_within_parentheses/parens.java");
const PARENS_OUT: &str = include_str!("../java/space_within_parentheses/parens.out.java");
const PARENS_DEFAULT_OUT: &str =
    include_str!("../java/space_within_parentheses/parens_default.out.java");

#[test]
fn pads_plain_parentheses_when_on() {
    let s = style(|st| st.space_within_parentheses = true);
    assert_eq!(format_with(PARENS, &s), PARENS_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(PARENS), PARENS_DEFAULT_OUT);
}
