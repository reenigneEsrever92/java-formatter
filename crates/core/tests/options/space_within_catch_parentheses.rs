//! SPACE_WITHIN_CATCH_PARENTHESES — padding inside `catch` parentheses.
//! Fixtures live under tests/java/space_within_catch_parentheses/.

use super::common::*;

const CATCH_COND: &str = include_str!("../java/space_within_catch_parentheses/catch_cond.java");
const CATCH_COND_OUT: &str =
    include_str!("../java/space_within_catch_parentheses/catch_cond.out.java");
const CATCH_COND_DEFAULT_OUT: &str =
    include_str!("../java/space_within_catch_parentheses/catch_cond_default.out.java");

#[test]
fn pads_catch_parentheses_when_on() {
    let s = style(|st| st.space_within_catch_parentheses = true);
    assert_eq!(format_with(CATCH_COND, &s), CATCH_COND_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(CATCH_COND), CATCH_COND_DEFAULT_OUT);
}
