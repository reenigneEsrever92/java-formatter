//! SPACE_BEFORE_ELSE_KEYWORD — space between `}` and the `else` keyword. Defaults to on.
//! Fixtures live under tests/java/space_before_else_keyword/.

use super::common::*;

const ELSE_KW: &str = include_str!("../java/space_before_else_keyword/else_kw.java");
const ELSE_KW_OUT: &str = include_str!("../java/space_before_else_keyword/else_kw.out.java");
const ELSE_KW_DEFAULT_OUT: &str =
    include_str!("../java/space_before_else_keyword/else_kw_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_else_keyword = false);
    assert_eq!(format_with(ELSE_KW, &s), ELSE_KW_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(ELSE_KW), ELSE_KW_DEFAULT_OUT);
}
