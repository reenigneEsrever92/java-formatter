//! SPACE_BEFORE_SWITCH_LBRACE — space before the opening brace of a `switch` body. Defaults to on.
//! Fixtures live under tests/java/space_before_switch_lbrace/.

use super::common::*;

const SWITCH_BODY: &str = include_str!("../java/space_before_switch_lbrace/switch_body.java");
const SWITCH_BODY_OUT: &str =
    include_str!("../java/space_before_switch_lbrace/switch_body.out.java");
const SWITCH_BODY_DEFAULT_OUT: &str =
    include_str!("../java/space_before_switch_lbrace/switch_body_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_switch_lbrace = false);
    assert_eq!(format_with(SWITCH_BODY, &s), SWITCH_BODY_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(SWITCH_BODY), SWITCH_BODY_DEFAULT_OUT);
}
