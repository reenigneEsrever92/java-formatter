//! SPACE_AROUND_ASSIGNMENT_OPERATORS — space around assignment operators
//! (=, +=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=, >>>=).
//! Fixtures live under tests/java/space_around_assignment_operators/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_around_assignment_operators/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_around_assignment_operators/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_around_assignment_operators/mixed_default.out.java");

#[test]
fn off_tightens_assignment_operators() {
    let style = style(|s| s.space_around_assignment_operators = false);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
