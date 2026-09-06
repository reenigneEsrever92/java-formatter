//! SPACE_BEFORE_FINALLY_LBRACE — space between `finally` and its body's opening brace. Defaults to on.
//! Fixtures live under tests/java/space_before_finally_lbrace/.

use super::common::*;

const FINALLY_BODY: &str = include_str!("../java/space_before_finally_lbrace/finally_body.java");
const FINALLY_BODY_OUT: &str =
    include_str!("../java/space_before_finally_lbrace/finally_body.out.java");
const FINALLY_BODY_DEFAULT_OUT: &str =
    include_str!("../java/space_before_finally_lbrace/finally_body_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_finally_lbrace = false);
    assert_eq!(format_with(FINALLY_BODY, &s), FINALLY_BODY_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(FINALLY_BODY), FINALLY_BODY_DEFAULT_OUT);
}
