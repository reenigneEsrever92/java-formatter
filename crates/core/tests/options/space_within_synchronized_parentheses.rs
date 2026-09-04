//! SPACE_WITHIN_SYNCHRONIZED_PARENTHESES — padding inside `synchronized`
//! lock parentheses.
//! Fixtures live under tests/java/space_within_synchronized_parentheses/.

use super::common::*;

const SYNC_COND: &str =
    include_str!("../java/space_within_synchronized_parentheses/sync_cond.java");
const SYNC_COND_OUT: &str =
    include_str!("../java/space_within_synchronized_parentheses/sync_cond.out.java");
const SYNC_COND_DEFAULT_OUT: &str =
    include_str!("../java/space_within_synchronized_parentheses/sync_cond_default.out.java");

#[test]
fn pads_synchronized_lock_when_on() {
    let s = style(|st| st.space_within_synchronized_parentheses = true);
    assert_eq!(format_with(SYNC_COND, &s), SYNC_COND_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(SYNC_COND), SYNC_COND_DEFAULT_OUT);
}
