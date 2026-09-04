//! SPACE_BEFORE_FOR_PARENTHESES — space between `for` and its header (classic and enhanced). Defaults to on.
//! Fixtures live under tests/java/space_before_for_parentheses/.

use super::common::*;

const FOR_LOOPS: &str = include_str!("../java/space_before_for_parentheses/for_loops.java");
const FOR_LOOPS_OUT: &str = include_str!("../java/space_before_for_parentheses/for_loops.out.java");
const FOR_LOOPS_DEFAULT_OUT: &str =
    include_str!("../java/space_before_for_parentheses/for_loops_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_for_parentheses = false);
    assert_eq!(format_with(FOR_LOOPS, &s), FOR_LOOPS_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(FOR_LOOPS), FOR_LOOPS_DEFAULT_OUT);
}
