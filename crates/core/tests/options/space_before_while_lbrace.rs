//! SPACE_BEFORE_WHILE_LBRACE — space before the opening brace of a `while` body. Defaults to on.
//! Fixtures live under tests/java/space_before_while_lbrace/.

use super::common::*;

const WHILE_BODY: &str = include_str!("../java/space_before_while_lbrace/while_body.java");
const WHILE_BODY_OUT: &str = include_str!("../java/space_before_while_lbrace/while_body.out.java");
const WHILE_BODY_DEFAULT_OUT: &str =
    include_str!("../java/space_before_while_lbrace/while_body_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_while_lbrace = false);
    assert_eq!(format_with(WHILE_BODY, &s), WHILE_BODY_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(WHILE_BODY), WHILE_BODY_DEFAULT_OUT);
}
