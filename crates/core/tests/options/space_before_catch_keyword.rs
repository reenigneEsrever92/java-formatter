//! SPACE_BEFORE_CATCH_KEYWORD — space between `}` and the `catch` keyword. Defaults to on.
//! Fixtures live under tests/java/space_before_catch_keyword/.

use super::common::*;

const CATCH_KW: &str = include_str!("../java/space_before_catch_keyword/catch_kw.java");
const CATCH_KW_OUT: &str = include_str!("../java/space_before_catch_keyword/catch_kw.out.java");
const CATCH_KW_DEFAULT_OUT: &str =
    include_str!("../java/space_before_catch_keyword/catch_kw_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_catch_keyword = false);
    assert_eq!(format_with(CATCH_KW, &s), CATCH_KW_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(CATCH_KW), CATCH_KW_DEFAULT_OUT);
}
