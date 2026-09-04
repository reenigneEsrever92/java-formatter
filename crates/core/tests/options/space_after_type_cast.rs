//! SPACE_AFTER_TYPE_CAST — space between (Type) and the cast value. Defaults
//! to on ((int) x).
//! Fixtures live under tests/java/space_after_type_cast/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_after_type_cast/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_after_type_cast/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_after_type_cast/mixed_default.out.java");

#[test]
fn off_glues_cast_value_to_type() {
    let style = style(|s| s.space_after_type_cast = false);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
