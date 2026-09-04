//! SPACE_BEFORE_SEMICOLON — space before `;` inside a `for` header. Defaults
//! to off (for (int i = 0; i < n; i++)).
//! Fixtures live under tests/java/space_before_semicolon/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_before_semicolon/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_before_semicolon/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_before_semicolon/mixed_default.out.java");

#[test]
fn on_spaces_before_for_header_semicolons() {
    let style = style(|s| s.space_before_semicolon = true);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
