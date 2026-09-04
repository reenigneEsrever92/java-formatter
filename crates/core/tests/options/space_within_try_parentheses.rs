//! SPACE_WITHIN_TRY_PARENTHESES — padding inside try-with-resources
//! parentheses.
//! Fixtures live under tests/java/space_within_try_parentheses/.

use super::common::*;

const TRY_RES: &str = include_str!("../java/space_within_try_parentheses/try_res.java");
const TRY_RES_OUT: &str = include_str!("../java/space_within_try_parentheses/try_res.out.java");
const TRY_RES_DEFAULT_OUT: &str =
    include_str!("../java/space_within_try_parentheses/try_res_default.out.java");

#[test]
fn pads_try_with_resources_when_on() {
    let s = style(|st| st.space_within_try_parentheses = true);
    assert_eq!(format_with(TRY_RES, &s), TRY_RES_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(TRY_RES), TRY_RES_DEFAULT_OUT);
}
