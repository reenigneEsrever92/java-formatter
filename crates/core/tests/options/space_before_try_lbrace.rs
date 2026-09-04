//! SPACE_BEFORE_TRY_LBRACE — space before the opening brace of a `try` body. Defaults to on.
//! Fixtures live under tests/java/space_before_try_lbrace/.

use super::common::*;

const TRY_BODY: &str = include_str!("../java/space_before_try_lbrace/try_body.java");
const TRY_BODY_OUT: &str = include_str!("../java/space_before_try_lbrace/try_body.out.java");
const TRY_BODY_DEFAULT_OUT: &str =
    include_str!("../java/space_before_try_lbrace/try_body_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_try_lbrace = false);
    assert_eq!(format_with(TRY_BODY, &s), TRY_BODY_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(TRY_BODY), TRY_BODY_DEFAULT_OUT);
}
