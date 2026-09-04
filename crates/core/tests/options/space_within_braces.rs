//! SPACE_WITHIN_BRACES — padding inside empty code-block / body braces.
//! Fixtures live under tests/java/space_within_braces/.

use super::common::*;

const BRACES: &str = include_str!("../java/space_within_braces/braces.java");
const BRACES_OUT: &str = include_str!("../java/space_within_braces/braces.out.java");
const BRACES_DEFAULT_OUT: &str = include_str!("../java/space_within_braces/braces_default.out.java");

#[test]
fn pads_empty_block_braces_when_on() {
    let s = style(|st| st.space_within_braces = true);
    assert_eq!(format_with(BRACES, &s), BRACES_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(BRACES), BRACES_DEFAULT_OUT);
}
