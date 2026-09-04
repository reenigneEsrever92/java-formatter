//! SPACE_BEFORE_COMMA — space before `,`. Defaults to off (f(a, b)).
//! Fixtures live under tests/java/space_before_comma/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_before_comma/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_before_comma/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_before_comma/mixed_default.out.java");

#[test]
fn on_spaces_before_commas() {
    let style = style(|s| s.space_before_comma = true);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
