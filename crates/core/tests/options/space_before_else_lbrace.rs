//! SPACE_BEFORE_ELSE_LBRACE — space between `else` and its body's opening brace. Defaults to on.
//! Fixtures live under tests/java/space_before_else_lbrace/.

use super::common::*;

const ELSE_BODY: &str = include_str!("../java/space_before_else_lbrace/else_body.java");
const ELSE_BODY_OUT: &str = include_str!("../java/space_before_else_lbrace/else_body.out.java");
const ELSE_BODY_DEFAULT_OUT: &str =
    include_str!("../java/space_before_else_lbrace/else_body_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_else_lbrace = false);
    assert_eq!(format_with(ELSE_BODY, &s), ELSE_BODY_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(ELSE_BODY), ELSE_BODY_DEFAULT_OUT);
}
