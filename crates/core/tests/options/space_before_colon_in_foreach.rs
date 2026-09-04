//! SPACE_BEFORE_COLON_IN_FOREACH — space before the colon in an enhanced-`for`
//! header. Defaults to on (for (T t : xs)).
//! Fixtures live under tests/java/space_before_colon_in_foreach/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_before_colon_in_foreach/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_before_colon_in_foreach/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_before_colon_in_foreach/mixed_default.out.java");

#[test]
fn off_glues_foreach_colon_to_variable() {
    let style = style(|s| s.space_before_colon_in_foreach = false);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
