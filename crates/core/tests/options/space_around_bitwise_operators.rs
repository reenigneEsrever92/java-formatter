//! SPACE_AROUND_BITWISE_OPERATORS — space around bitwise operators (&, |, ^).
//! Fixtures live under tests/java/space_around_bitwise_operators/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_around_bitwise_operators/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_around_bitwise_operators/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_around_bitwise_operators/mixed_default.out.java");

#[test]
fn off_tightens_bitwise_operators() {
    let style = style(|s| s.space_around_bitwise_operators = false);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
