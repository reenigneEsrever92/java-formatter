//! SPACE_BEFORE_FINALLY_KEYWORD — space between `}` and the `finally` keyword. Defaults to on.
//! Fixtures live under tests/java/space_before_finally_keyword/.

use super::common::*;

const FINALLY_KW: &str = include_str!("../java/space_before_finally_keyword/finally_kw.java");
const FINALLY_KW_OUT: &str = include_str!("../java/space_before_finally_keyword/finally_kw.out.java");
const FINALLY_KW_DEFAULT_OUT: &str =
    include_str!("../java/space_before_finally_keyword/finally_kw_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_finally_keyword = false);
    assert_eq!(format_with(FINALLY_KW, &s), FINALLY_KW_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(FINALLY_KW), FINALLY_KW_DEFAULT_OUT);
}
