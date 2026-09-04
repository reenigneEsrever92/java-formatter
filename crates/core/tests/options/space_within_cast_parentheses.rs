//! SPACE_WITHIN_CAST_PARENTHESES — padding inside `( Type )` cast parentheses.
//! Fixtures live under tests/java/space_within_cast_parentheses/.

use super::common::*;

const CAST: &str = include_str!("../java/space_within_cast_parentheses/cast.java");
const CAST_OUT: &str = include_str!("../java/space_within_cast_parentheses/cast.out.java");
const CAST_DEFAULT_OUT: &str =
    include_str!("../java/space_within_cast_parentheses/cast_default.out.java");

#[test]
fn pads_cast_parentheses_when_on() {
    let s = style(|st| st.space_within_cast_parentheses = true);
    assert_eq!(format_with(CAST, &s), CAST_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(CAST), CAST_DEFAULT_OUT);
}
