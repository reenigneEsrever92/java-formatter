//! SPACE_BEFORE_CATCH_LBRACE — space before the opening brace of a `catch` body. Defaults to on.
//! Fixtures live under tests/java/space_before_catch_lbrace/.

use super::common::*;

const CATCH_BODY: &str = include_str!("../java/space_before_catch_lbrace/catch_body.java");
const CATCH_BODY_OUT: &str = include_str!("../java/space_before_catch_lbrace/catch_body.out.java");
const CATCH_BODY_DEFAULT_OUT: &str =
    include_str!("../java/space_before_catch_lbrace/catch_body_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_catch_lbrace = false);
    assert_eq!(format_with(CATCH_BODY, &s), CATCH_BODY_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(CATCH_BODY), CATCH_BODY_DEFAULT_OUT);
}
