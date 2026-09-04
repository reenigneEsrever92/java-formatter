//! SPACE_BEFORE_SYNCHRONIZED_PARENTHESES — space between `synchronized` and its lock expression. Defaults to on.
//! Fixtures live under tests/java/space_before_synchronized_parentheses/.

use super::common::*;

const SYNC: &str = include_str!("../java/space_before_synchronized_parentheses/sync.java");
const SYNC_OUT: &str = include_str!("../java/space_before_synchronized_parentheses/sync.out.java");
const SYNC_DEFAULT_OUT: &str =
    include_str!("../java/space_before_synchronized_parentheses/sync_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_synchronized_parentheses = false);
    assert_eq!(format_with(SYNC, &s), SYNC_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(SYNC), SYNC_DEFAULT_OUT);
}
