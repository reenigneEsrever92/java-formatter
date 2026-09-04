//! SPACE_AROUND_UNARY_OPERATOR — space between a unary operator
//! (!, ~, unary +/-, ++, --) and its operand. Defaults to off (space-less).
//! Fixtures live under tests/java/space_around_unary_operator/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_around_unary_operator/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_around_unary_operator/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_around_unary_operator/mixed_default.out.java");

#[test]
fn on_spaces_unary_operators_and_updates() {
    let style = style(|s| s.space_around_unary_operator = true);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_stays_space_less() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
