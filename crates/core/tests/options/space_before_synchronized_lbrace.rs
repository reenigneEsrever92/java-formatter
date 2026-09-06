//! SPACE_BEFORE_SYNCHRONIZED_LBRACE — space before the opening brace of a `synchronized` body. Defaults to on.
//! Fixtures live under tests/java/space_before_synchronized_lbrace/.

use super::common::*;

const SYNC_BODY: &str = include_str!("../java/space_before_synchronized_lbrace/sync_body.java");
const SYNC_BODY_OUT: &str =
    include_str!("../java/space_before_synchronized_lbrace/sync_body.out.java");
const SYNC_BODY_DEFAULT_OUT: &str =
    include_str!("../java/space_before_synchronized_lbrace/sync_body_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_synchronized_lbrace = false);
    assert_eq!(format_with(SYNC_BODY, &s), SYNC_BODY_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(SYNC_BODY), SYNC_BODY_DEFAULT_OUT);
}
