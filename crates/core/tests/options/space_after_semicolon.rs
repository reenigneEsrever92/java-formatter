//! SPACE_AFTER_SEMICOLON — space after `;` inside a `for` header. Defaults to
//! on (for (int i = 0; i < n; i++)).
//! Fixtures live under tests/java/space_after_semicolon/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_after_semicolon/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_after_semicolon/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_after_semicolon/mixed_default.out.java");

#[test]
fn off_glues_for_header_semicolons() {
    let style = style(|s| s.space_after_semicolon = false);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
