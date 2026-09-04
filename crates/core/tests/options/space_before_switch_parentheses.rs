//! SPACE_BEFORE_SWITCH_PARENTHESES — space between `switch` and its selector. Defaults to on.
//! Fixtures live under tests/java/space_before_switch_parentheses/.

use super::common::*;

const SWITCH_STMT: &str = include_str!("../java/space_before_switch_parentheses/switch_stmt.java");
const SWITCH_STMT_OUT: &str = include_str!("../java/space_before_switch_parentheses/switch_stmt.out.java");
const SWITCH_STMT_DEFAULT_OUT: &str =
    include_str!("../java/space_before_switch_parentheses/switch_stmt_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_switch_parentheses = false);
    assert_eq!(format_with(SWITCH_STMT, &s), SWITCH_STMT_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(SWITCH_STMT), SWITCH_STMT_DEFAULT_OUT);
}
