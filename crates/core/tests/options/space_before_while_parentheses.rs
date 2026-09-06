//! SPACE_BEFORE_WHILE_PARENTHESES — space between `while` and its condition, including the do-while tail. Defaults to on.
//! Fixtures live under tests/java/space_before_while_parentheses/.

use super::common::*;

const WHILE_LOOPS: &str = include_str!("../java/space_before_while_parentheses/while_loops.java");
const WHILE_LOOPS_OUT: &str =
    include_str!("../java/space_before_while_parentheses/while_loops.out.java");
const WHILE_LOOPS_DEFAULT_OUT: &str =
    include_str!("../java/space_before_while_parentheses/while_loops_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_while_parentheses = false);
    assert_eq!(format_with(WHILE_LOOPS, &s), WHILE_LOOPS_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(WHILE_LOOPS), WHILE_LOOPS_DEFAULT_OUT);
}
