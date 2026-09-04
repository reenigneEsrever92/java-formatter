//! SPACE_BEFORE_FOR_LBRACE — space before the opening brace of `for` and enhanced-`for` bodies. Defaults to on.
//! Fixtures live under tests/java/space_before_for_lbrace/.

use super::common::*;

const FOR_BODY: &str = include_str!("../java/space_before_for_lbrace/for_body.java");
const FOR_BODY_OUT: &str = include_str!("../java/space_before_for_lbrace/for_body.out.java");
const FOR_BODY_DEFAULT_OUT: &str =
    include_str!("../java/space_before_for_lbrace/for_body_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_for_lbrace = false);
    assert_eq!(format_with(FOR_BODY, &s), FOR_BODY_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(FOR_BODY), FOR_BODY_DEFAULT_OUT);
}
