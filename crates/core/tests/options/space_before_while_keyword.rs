//! SPACE_BEFORE_WHILE_KEYWORD — space between `}` and the trailing `while` of a do-statement. Defaults to on.
//! Fixtures live under tests/java/space_before_while_keyword/.

use super::common::*;

const WHILE_KW: &str = include_str!("../java/space_before_while_keyword/while_kw.java");
const WHILE_KW_OUT: &str = include_str!("../java/space_before_while_keyword/while_kw.out.java");
const WHILE_KW_DEFAULT_OUT: &str =
    include_str!("../java/space_before_while_keyword/while_kw_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_while_keyword = false);
    assert_eq!(format_with(WHILE_KW, &s), WHILE_KW_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(WHILE_KW), WHILE_KW_DEFAULT_OUT);
}
