//! SPACE_AROUND_SHIFT_OPERATORS — space around shift operators (<<, >>, >>>).
//! Fixtures live under tests/java/space_around_shift_operators/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_around_shift_operators/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_around_shift_operators/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_around_shift_operators/mixed_default.out.java");

#[test]
fn off_tightens_shift_operators() {
    let style = style(|s| s.space_around_shift_operators = false);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
