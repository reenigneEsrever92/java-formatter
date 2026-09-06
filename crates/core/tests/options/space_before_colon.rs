//! SPACE_BEFORE_COLON — space before `:` in a ternary expression. Defaults to
//! on (a ? b : c).
//! Fixtures live under tests/java/space_before_colon/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_before_colon/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_before_colon/mixed.out.java");
const MIXED_DEFAULT_OUT: &str = include_str!("../java/space_before_colon/mixed_default.out.java");

#[test]
fn off_glues_colon_to_consequence() {
    let style = style(|s| s.space_before_colon = false);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
